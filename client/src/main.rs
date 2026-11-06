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

    // (#1) Détection de l'appareil.
    device::adb::ensure_available()?;
    connection.establish()?;
    let device = device::adb::detect(args.serial.as_deref())?;
    log::info!(
        "appareil sélectionné : {} (état : {:?})",
        device.serial,
        device.state
    );

    // (#2/#3) Déploiement du serveur + tunnel.
    // Ordre important en mode reverse : push → ouvrir le tunnel (on écoute) →
    // démarrer le serveur (qui se connecte) → accepter la connexion.
    if device::server_bin::is_placeholder() {
        log::warn!(
            "serveur embarqué = placeholder vide (build sans SDK Android) : \
             déploiement sauté. Recompilez avec GNINOU_BUILD_SERVER=1."
        );
    } else {
        let serial = Some(device.serial.as_str());

        device::server_bin::push(serial)?;
        log::info!("serveur poussé sur {}", device.serial);

        let tunnel = device::Tunnel::open_reverse(serial, device::tunnel::DEFAULT_PORT)?;
        log::info!(
            "tunnel {:?} ouvert (localabstract:{} ↔ tcp:{})",
            tunnel.mode,
            device::tunnel::SOCKET_NAME,
            tunnel.port
        );

        let _server = device::server_bin::start(serial, &args.server_args())?;
        log::info!("serveur lancé sur {}", device.serial);

        // #3 : le canal est prêt ; la lecture du handshake arrive avec le protocole.
        let _channel = tunnel.accept()?;
        log::info!("connexion établie avec le serveur (handshake)");

        // TODO (suite du pipeline) :
        //   #4-7.  media::video + media::audio (décodage/lecture)
        //   #8-10. control (capture entrées -> protocol -> socket)
    }

    log::warn!("média et contrôle non implémentés — prochaines fonctionnalités");

    Ok(())
}
