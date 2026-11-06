//! Pilotage d'`adb` : détection, push du serveur, ouverture des tunnels.
//!
//! Squelette : signatures et TODO. L'implémentation réelle viendra ensuite.

use anyhow::{Context, Result};
use std::process::Command;

/// Vérifie qu'`adb` est disponible dans le PATH.
pub fn ensure_available() -> Result<()> {
    let output = Command::new("adb")
        .arg("version")
        .output()
        .context("`adb` introuvable dans le PATH")?;

    if !output.status.success() {
        anyhow::bail!("`adb version` a échoué");
    }
    Ok(())
}

/// Pousse le binaire du serveur gninou sur l'appareil.
pub fn push_server(_serial: Option<&str>) -> Result<()> {
    // TODO: adb [-s serial] push server.jar /data/local/tmp/gninou-server.jar
    todo!("push du serveur non implémenté")
}

/// Ouvre un tunnel (reverse de préférence, forward en repli).
pub fn open_tunnel(_serial: Option<&str>, _port: u16) -> Result<()> {
    // TODO: adb reverse localabstract:gninou tcp:<port>
    todo!("ouverture de tunnel non implémentée")
}
