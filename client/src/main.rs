//! gninou — client desktop.
//!
//! Squelette initial : parse les arguments, prépare le pilotage d'`adb`
//! et le futur pipeline de décodage/affichage. La logique média n'est
//! pas encore implémentée.

mod adb;
mod cli;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.log_level())
        .init();

    log::info!("gninou {} — démarrage", env!("CARGO_PKG_VERSION"));
    log::debug!("arguments : {args:?}");

    if let Some(serial) = &args.serial {
        log::info!("appareil ciblé : {serial}");
    } else {
        log::info!("aucun appareil précisé — premier appareil détecté");
    }

    // TODO: 1. adb::ensure_available()
    //       2. adb::push_server()
    //       3. adb::open_tunnel()
    //       4. démarrer le serveur via app_process
    //       5. décoder (ffmpeg) + afficher (sdl)
    log::warn!("pipeline média non implémenté — squelette uniquement");

    Ok(())
}
