//! Assistant d'appairage interactif (#29, Android 11+).
//!
//! Transforme l'appairage sans fil en une courte conversation au terminal :
//! l'utilisateur n'a en pratique qu'à saisir le code à 6 chiffres affiché par
//! l'appareil. Tout le reste (découverte des points d'appairage et de
//! connexion) est automatique via mDNS (#28), avec repli manuel si la
//! découverte échoue (mDNS indisponible, Android < 11, réseaux filtrés…).
//!
//! La **logique de décision** (évaluation d'une saisie de code, choix d'un
//! point d'accès) est pure et testée ; l'**I/O terminal** et les appels `adb`
//! sont une fine couche validée sur matériel.

use std::io::{self, BufRead, Write};

use anyhow::{bail, Result};

use crate::device::{connection, mdns};

/// Nombre de saisies de code autorisées avant abandon.
pub const MAX_CODE_ATTEMPTS: u32 = 3;

/// Résultat de l'évaluation d'une saisie de code (logique **pure**).
#[derive(Debug, PartialEq, Eq)]
pub enum CodeOutcome {
    /// Code valide (6 chiffres) : la valeur normalisée est renvoyée.
    Valid(String),
    /// Code invalide mais il reste des tentatives : message d'explication.
    Retry(String),
    /// Code invalide et plus de tentatives : on abandonne.
    GiveUp,
}

/// Évalue une saisie de code en fonction des tentatives restantes.
///
/// `attempts_left` est le nombre de tentatives **y compris celle-ci** : à 1,
/// un code invalide conduit à `GiveUp` ; au-dessus, à `Retry`.
pub fn eval_code_attempt(input: &str, attempts_left: u32) -> CodeOutcome {
    match connection::validate_pair_code(input) {
        Ok(code) => CodeOutcome::Valid(code),
        Err(e) if attempts_left > 1 => CodeOutcome::Retry(e.to_string()),
        Err(_) => CodeOutcome::GiveUp,
    }
}

/// Choisit un point d'accès : l'adresse découverte si elle existe, sinon la
/// saisie manuelle fournie (normalisée). Logique **pure**.
///
/// Renvoie une erreur si aucune des deux n'est utilisable.
pub fn resolve_endpoint(discovered: Option<String>, manual: Option<&str>) -> Result<String> {
    match (discovered, manual) {
        (Some(addr), _) => Ok(addr),
        (None, Some(m)) => connection::normalize_addr(m),
        (None, None) => bail!("aucune adresse fournie"),
    }
}

/// Affiche une invite et lit une ligne au clavier (I/O terminal).
fn prompt(message: &str) -> Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Lit le code à 6 chiffres au clavier, avec jusqu'à [`MAX_CODE_ATTEMPTS`]
/// essais (I/O terminal + logique pure [`eval_code_attempt`]).
fn ask_code() -> Result<String> {
    for attempt in (1..=MAX_CODE_ATTEMPTS).rev() {
        let input = prompt("  Code d'appairage (6 chiffres) : ")?;
        match eval_code_attempt(&input, attempt) {
            CodeOutcome::Valid(code) => return Ok(code),
            CodeOutcome::Retry(msg) => {
                eprintln!("  ✗ {msg} — il reste {} essai(s).", attempt - 1);
            }
            CodeOutcome::GiveUp => break,
        }
    }
    bail!("trop de codes invalides saisis — appairage abandonné")
}

/// Résout un point d'accès : tente la découverte mDNS, sinon demande l'adresse.
///
/// `discovered` est la fermeture d'I/O qui interroge mDNS pour ce type de
/// service ; `label` et `hint` servent les messages à l'écran.
fn resolve_or_ask(label: &str, hint: &str, discovered: Result<Option<String>>) -> Result<String> {
    match discovered {
        Ok(Some(addr)) => {
            println!("  ✓ {label} détecté automatiquement : {addr}");
            Ok(addr)
        }
        Ok(None) | Err(_) => {
            println!("  (découverte mDNS indisponible — saisie manuelle)");
            let manual = prompt(&format!("  {hint} (ip:port) : "))?;
            resolve_endpoint(None, Some(&manual))
        }
    }
}

/// Lance l'assistant interactif et renvoie l'adresse `ip:port` prête pour la
/// connexion réseau. N'établit pas la connexion lui-même : l'appelant
/// (`main`) s'en charge via `Connection::TcpIp`.
pub fn run() -> Result<String> {
    println!("=== Assistant d'appairage Wi-Fi (Android 11+) ===");
    println!("Sur l'appareil : Paramètres › Options pour développeurs › Débogage sans fil");
    println!("  › « Associer l'appareil à l'aide d'un code d'association ».");
    println!("L'appareil affiche un code à 6 chiffres et une adresse ip:port.\n");

    // 1) Point d'appairage (service `_adb-tls-pairing._tcp`), auto ou manuel.
    let pairing_addr = resolve_or_ask(
        "point d'appairage",
        "Adresse d'appairage affichée par l'appareil",
        mdns::discover_pairing_addr(),
    )?;

    // 2) Code à 6 chiffres (jusqu'à 3 essais), puis `adb pair`.
    let code = ask_code()?;
    connection::pair(&pairing_addr, &code)?;
    println!("  ✓ appairage réussi.\n");

    // 3) Point de connexion (service `_adb-tls-connect._tcp`), auto ou manuel.
    let connect_addr = resolve_or_ask(
        "point de connexion",
        "Adresse de connexion (menu « Débogage sans fil »)",
        mdns::discover_connect_addr(),
    )?;

    println!("  → connexion à {connect_addr}");
    Ok(connect_addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_valide_est_accepte() {
        assert_eq!(
            eval_code_attempt("123456", 3),
            CodeOutcome::Valid("123456".to_string())
        );
    }

    #[test]
    fn code_invalide_avec_essais_restants_redemande() {
        match eval_code_attempt("12", 3) {
            CodeOutcome::Retry(_) => {}
            other => panic!("attendu Retry, obtenu {other:?}"),
        }
    }

    #[test]
    fn code_invalide_au_dernier_essai_abandonne() {
        assert_eq!(eval_code_attempt("abc", 1), CodeOutcome::GiveUp);
    }

    #[test]
    fn endpoint_prefere_la_decouverte() {
        let r = resolve_endpoint(Some("192.168.1.42:5555".into()), Some("10.0.0.1:1")).unwrap();
        assert_eq!(r, "192.168.1.42:5555");
    }

    #[test]
    fn endpoint_repli_manuel_normalise() {
        let r = resolve_endpoint(None, Some("192.168.1.9:5555")).unwrap();
        assert_eq!(r, "192.168.1.9:5555");
    }

    #[test]
    fn endpoint_sans_adresse_echoue() {
        assert!(resolve_endpoint(None, None).is_err());
    }
}
