//! Serveur embarqué : extraction, push sur l'appareil et lancement.
//!
//! Le binaire du serveur Kotlin est inclus dans l'exécutable client à la
//! compilation (`include_bytes!`). Au démarrage, le client l'écrit dans un
//! fichier temporaire, le pousse via `adb push`, puis l'exécute à distance
//! via `app_process`. Rien n'est installé de façon permanente sur l'appareil.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

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

/// Écrit le jar embarqué dans un fichier temporaire local.
fn extract_to_temp() -> Result<PathBuf> {
    let path = std::env::temp_dir().join("gninou-server.jar");
    std::fs::write(&path, SERVER_JAR)
        .with_context(|| format!("écriture du serveur dans {}", path.display()))?;
    Ok(path)
}

fn adb(serial: Option<&str>) -> Command {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd
}

/// Pousse le serveur embarqué sur l'appareil.
pub fn push(serial: Option<&str>) -> Result<()> {
    if is_placeholder() {
        anyhow::bail!(
            "serveur non embarqué (placeholder) : recompilez avec GNINOU_BUILD_SERVER=1"
        );
    }
    let local = extract_to_temp()?;
    let status = adb(serial)
        .arg("push")
        .arg(&local)
        .arg(DEVICE_PATH)
        .status()
        .context("échec de `adb push`")?;
    anyhow::ensure!(status.success(), "`adb push` a échoué");
    Ok(())
}

/// Démarre le serveur à distance via `app_process` et renvoie le processus.
///
/// `args` sont les options passées au serveur (ex. `max_size=1024`).
pub fn start(serial: Option<&str>, args: &[String]) -> Result<std::process::Child> {
    let remote = format!(
        "CLASSPATH={DEVICE_PATH} app_process / {SERVER_MAIN} {}",
        args.join(" ")
    );
    adb(serial)
        .arg("shell")
        .arg(remote)
        .spawn()
        .context("échec du lancement du serveur via app_process")
}
