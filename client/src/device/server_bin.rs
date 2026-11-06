//! Serveur embarqué : extraction, push sur l'appareil et lancement.
//!
//! Le binaire du serveur Kotlin est inclus dans l'exécutable client à la
//! compilation (`include_bytes!`). Au démarrage, le client l'écrit dans un
//! fichier temporaire, le pousse via `adb push`, puis l'exécute à distance
//! via `app_process`. Rien n'est installé de façon permanente sur l'appareil.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command};

/// Jar du serveur embarqué (produit par Gradle, placé dans assets/ par build.rs).
static SERVER_JAR: &[u8] = include_bytes!("../../assets/gninou-server.jar");

/// Emplacement du serveur sur l'appareil (dossier accessible sans root).
const DEVICE_PATH: &str = "/data/local/tmp/gninou-server.jar";

/// Classe d'entrée du serveur.
const SERVER_MAIN: &str = "io.karfi.gninou.server.Server";

/// Indique si le jar embarqué est un placeholder vide (build de dev sans SDK).
pub fn is_placeholder() -> bool {
    SERVER_JAR.is_empty()
}

/// Nom du fichier temporaire local recevant le jar avant le push.
fn temp_jar_path() -> PathBuf {
    std::env::temp_dir().join("gninou-server.jar")
}

/// Écrit le jar embarqué dans un fichier temporaire local, renvoie son chemin.
fn extract_to_temp() -> Result<PathBuf> {
    let path = temp_jar_path();
    std::fs::write(&path, SERVER_JAR)
        .with_context(|| format!("écriture du serveur dans {}", path.display()))?;
    Ok(path)
}

/// Construit la commande distante exécutée par `app_process`.
///
/// Logique pure (testable sans appareil). Exemple de résultat :
/// `CLASSPATH=/data/local/tmp/gninou-server.jar app_process / io.karfi.gninou.server.Server max_size=1024`
fn remote_command(args: &[String]) -> String {
    let mut cmd = format!("CLASSPATH={DEVICE_PATH} app_process / {SERVER_MAIN}");
    if !args.is_empty() {
        cmd.push(' ');
        cmd.push_str(&args.join(" "));
    }
    cmd
}

/// Construit une commande `adb`, ciblant un appareil si `serial` est fourni.
fn adb(serial: Option<&str>) -> Command {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd
}

/// Garde-fou : refuse d'agir si le serveur embarqué est un placeholder vide.
fn ensure_embedded() -> Result<()> {
    anyhow::ensure!(
        !is_placeholder(),
        "serveur non embarqué (placeholder vide) : recompilez avec GNINOU_BUILD_SERVER=1"
    );
    Ok(())
}

/// Pousse le serveur embarqué sur l'appareil (`adb push`).
pub fn push(serial: Option<&str>) -> Result<()> {
    ensure_embedded()?;
    let local = extract_to_temp()?;
    let status = adb(serial)
        .arg("push")
        .arg(&local)
        .arg(DEVICE_PATH)
        .status()
        .context("échec de `adb push`")?;
    anyhow::ensure!(status.success(), "`adb push` a échoué (code {:?})", status.code());
    Ok(())
}

/// Démarre le serveur à distance via `app_process` et renvoie le processus.
///
/// `args` sont les options passées au serveur (ex. `max_size=1024`).
pub fn start(serial: Option<&str>, args: &[String]) -> Result<Child> {
    ensure_embedded()?;
    adb(serial)
        .arg("shell")
        .arg(remote_command(args))
        .spawn()
        .context("échec du lancement du serveur via app_process")
}

/// Pousse puis lance le serveur en une étape.
pub fn deploy(serial: Option<&str>, args: &[String]) -> Result<Child> {
    push(serial)?;
    start(serial, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_command_sans_args() {
        assert_eq!(
            remote_command(&[]),
            "CLASSPATH=/data/local/tmp/gninou-server.jar app_process / io.karfi.gninou.server.Server"
        );
    }

    #[test]
    fn remote_command_avec_args() {
        let args = vec!["max_size=1024".to_string(), "audio=false".to_string()];
        assert_eq!(
            remote_command(&args),
            "CLASSPATH=/data/local/tmp/gninou-server.jar app_process / \
             io.karfi.gninou.server.Server max_size=1024 audio=false"
        );
    }

    #[test]
    fn temp_jar_path_nomme_le_jar() {
        assert!(temp_jar_path().ends_with("gninou-server.jar"));
    }

    #[test]
    fn push_refuse_placeholder() {
        // En build de dev le jar est un placeholder vide : push/start doivent refuser.
        if is_placeholder() {
            assert!(push(None).is_err());
            assert!(start(None, &[]).is_err());
        }
    }
}
