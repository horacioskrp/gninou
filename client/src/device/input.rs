//! Injection d'entrées sur l'appareil via `adb shell input …` (#39).
//!
//! Traduit une [`InputAction`](crate::gui::control::InputAction) (en
//! coordonnées appareil) en commande `adb` et l'exécute. La **conversion** des
//! coordonnées et la **construction des arguments** sont pures et testées dans
//! [`crate::gui::control`] ; ici vit l'**I/O** (`adb`).

use anyhow::{ensure, Context, Result};
use std::process::Command;

use crate::gui::control::{adb_input_args, InputAction};

/// Exécute une action d'entrée sur l'appareil `serial` via `adb`.
///
/// `adb -s <serial> shell input tap|swipe|keyevent …`. Bloquant : à appeler
/// depuis un fil dédié pour ne pas figer l'UI.
pub fn inject(serial: &str, action: &InputAction) -> Result<()> {
    let args = adb_input_args(action);
    let out = Command::new("adb")
        .arg("-s")
        .arg(serial)
        .args(&args)
        .output()
        .context("échec de `adb shell input` (adb présent ? appareil prêt ?)")?;
    ensure!(
        out.status.success(),
        "`adb shell input` a échoué (appareil déconnecté ?)"
    );
    Ok(())
}
