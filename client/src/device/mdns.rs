//! Découverte mDNS des appareils en « débogage sans fil » (#28, Android 11+).
//!
//! S'appuie sur `adb mdns services`, qui annonce les appareils joignables via
//! Zeroconf/Bonjour. L'**analyse de la sortie** et le **filtrage des services
//! connectables** sont purs et testés ; l'exécution d'`adb` est l'I/O validée
//! sur matériel.

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Type de service mDNS pour la **connexion** (débogage sans fil, Android 11+).
pub const SERVICE_CONNECT: &str = "_adb-tls-connect._tcp";
/// Type de service mDNS adb « classique ».
pub const SERVICE_ADB: &str = "_adb._tcp";
/// Type de service mDNS d'**appairage** (à ne pas utiliser pour se connecter).
pub const SERVICE_PAIRING: &str = "_adb-tls-pairing._tcp";

/// Un service mDNS annoncé par `adb mdns services`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdnsService {
    /// Nom d'instance (ex. `adb-ABCDEF-123456`).
    pub name: String,
    /// Type de service (ex. `_adb-tls-connect._tcp`).
    pub service_type: String,
    /// Adresse `ip:port`.
    pub addr: String,
}

impl MdnsService {
    /// Le service permet-il une connexion directe (hors appairage) ?
    pub fn is_connectable(&self) -> bool {
        (self.service_type == SERVICE_CONNECT || self.service_type == SERVICE_ADB)
            && self.addr.contains(':')
    }
}

/// Analyse la sortie de `adb mdns services`. Logique **pure et testable**.
///
/// Format par ligne : `<nom>\t<type>\t<ip:port>` ; l'en-tête
/// « List of discovered mdns services » et les lignes vides sont ignorés.
pub fn parse_mdns_services(output: &str) -> Vec<MdnsService> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("List of discovered"))
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let service_type = parts.next()?.to_string();
            let addr = parts.next()?.to_string();
            Some(MdnsService {
                name,
                service_type,
                addr,
            })
        })
        .collect()
}

/// Ne garde que les services directement connectables.
pub fn connectable(services: &[MdnsService]) -> Vec<&MdnsService> {
    services.iter().filter(|s| s.is_connectable()).collect()
}

/// Ne garde que les services d'**appairage** (`_adb-tls-pairing._tcp`) valides.
pub fn pairing(services: &[MdnsService]) -> Vec<&MdnsService> {
    services
        .iter()
        .filter(|s| s.service_type == SERVICE_PAIRING && s.addr.contains(':'))
        .collect()
}

/// Exécute `adb mdns services` et renvoie tous les services annoncés analysés
/// (I/O). Utilisé par la vue unifiée `--list` (#30).
pub fn discover_services() -> Result<Vec<MdnsService>> {
    query_services()
}

/// Exécute `adb mdns services` et renvoie les services analysés (I/O).
fn query_services() -> Result<Vec<MdnsService>> {
    let out = Command::new("adb")
        .args(["mdns", "services"])
        .output()
        .context("échec de `adb mdns services`")?;
    anyhow::ensure!(out.status.success(), "`adb mdns services` a échoué");
    Ok(parse_mdns_services(&String::from_utf8_lossy(&out.stdout)))
}

/// Découvre les appareils sans fil et renvoie l'adresse `ip:port` du premier
/// service connectable. Erreur si aucun n'est trouvé.
pub fn discover_first() -> Result<String> {
    match discover_connect_addr()? {
        Some(addr) => Ok(addr),
        None => bail!(
            "aucun appareil sans fil découvert (activez « Débogage sans fil » sur l'appareil, \
             même réseau que le PC)"
        ),
    }
}

/// Adresse `ip:port` du premier service **de connexion** annoncé, s'il existe.
/// Renvoie `None` (sans erreur) quand aucun n'est trouvé, pour un repli manuel.
pub fn discover_connect_addr() -> Result<Option<String>> {
    let services = query_services()?;
    Ok(connectable(&services).first().map(|s| {
        log::info!("service mDNS de connexion : {} ({})", s.addr, s.name);
        s.addr.clone()
    }))
}

/// Adresse `ip:port` du premier service d'**appairage** annoncé, s'il existe.
/// Renvoie `None` (sans erreur) quand aucun n'est trouvé, pour un repli manuel.
pub fn discover_pairing_addr() -> Result<Option<String>> {
    let services = query_services()?;
    Ok(pairing(&services).first().map(|s| {
        log::info!("service mDNS d'appairage : {} ({})", s.addr, s.name);
        s.addr.clone()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "List of discovered mdns services\n\
        adb-ABCDEF-123456\t_adb-tls-pairing._tcp\t192.168.1.42:37000\n\
        adb-ABCDEF-123456\t_adb-tls-connect._tcp\t192.168.1.42:37123\n\
        adb-GHIJKL-789\t_adb._tcp\t10.0.0.9:5555\n";

    #[test]
    fn parse_trois_services() {
        let s = parse_mdns_services(SAMPLE);
        assert_eq!(s.len(), 3);
        assert_eq!(s[1].service_type, SERVICE_CONNECT);
        assert_eq!(s[1].addr, "192.168.1.42:37123");
    }

    #[test]
    fn filtre_connectables_exclut_appairage() {
        let s = parse_mdns_services(SAMPLE);
        let c = connectable(&s);
        // Les deux services connect/adb, pas celui d'appairage.
        assert_eq!(c.len(), 2);
        assert!(c.iter().all(|x| x.service_type != SERVICE_PAIRING));
        assert_eq!(c[0].addr, "192.168.1.42:37123");
    }

    #[test]
    fn filtre_appairage_isole_le_service_pairing() {
        let s = parse_mdns_services(SAMPLE);
        let p = pairing(&s);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].service_type, SERVICE_PAIRING);
        assert_eq!(p[0].addr, "192.168.1.42:37000");
    }

    #[test]
    fn parse_vide_ou_entete_seule() {
        assert!(parse_mdns_services("List of discovered mdns services\n").is_empty());
        assert!(parse_mdns_services("").is_empty());
    }

    #[test]
    fn connectable_exige_un_port() {
        let s = MdnsService {
            name: "x".into(),
            service_type: SERVICE_CONNECT.into(),
            addr: "192.168.1.1".into(), // pas de port
        };
        assert!(!s.is_connectable());
    }
}
