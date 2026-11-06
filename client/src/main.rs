//! gninou — client desktop.
//!
//! Affiche et contrôle un appareil Android (vidéo + audio) connecté en
//! USB, TCP/IP ou Wi-Fi, sans root ni app installée.
//!
//! Organisation en modules :
//!   - `protocol` : définition des trames échangées avec le serveur.
//!   - `device`   : pilotage d'adb, choix du mode de connexion, tunnel.
//!   - `media`    : décodage/affichage vidéo et lecture audio.
//!   - `control`  : capture clavier/souris -> messages de contrôle.

mod cli;
mod control;
mod device;
mod media;
mod protocol;

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

    let connection = args.connection();
    log::info!("mode de connexion : {connection:?}");

    if device::server_bin::is_placeholder() {
        log::warn!(
            "serveur embarqué = placeholder vide (build sans SDK Android). \
             Recompilez avec GNINOU_BUILD_SERVER=1 pour un binaire distribuable."
        );
    }

    // TODO (pipeline de bout en bout) :
    //   1. device::adb::ensure_available()
    //   2. connection.establish()                     // USB / TCP-IP / Wi-Fi
    //   3. device::server_bin::push() + server_bin::start()  // serveur embarqué
    //   4. device::adb::open_tunnel() -> sockets (video / audio / control)
    //   5. media::video + media::audio (décodage/lecture)
    //   6. control (capture entrées -> protocol -> socket)
    log::warn!("pipeline média non implémenté — squelette uniquement");

    Ok(())
}
