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
mod geometry;
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

    // (#11) Validation du codec vidéo demandé avant tout déploiement.
    let codec = media::codec::VideoCodec::parse(&args.video_codec)?;
    log::info!("codec vidéo demandé : {}", codec.as_str());

    // (#12) Validation des réglages (débit) avant tout déploiement.
    let bit_rate = args.bit_rate()?;
    match bit_rate {
        0 => log::info!("réglages vidéo : {}px max, {}fps, débit auto", args.max_size, args.max_fps),
        br => log::info!("réglages vidéo : {}px max, {}fps, {} bps", args.max_size, args.max_fps, br),
    }

    // (#13) Validation du crop et de la rotation.
    if let Some(crop) = args.crop()? {
        log::info!("rognage demandé : {}", crop.to_arg());
    }
    let rotation = args.rotation()?;
    if rotation != 0 {
        log::info!("rotation demandée : {rotation}°");
    }

    // (#21) Affichage ciblé / affichage virtuel (validation précoce).
    if let Some(id) = args.display {
        log::info!("affichage ciblé : #{id}");
    }
    if let Some(nd) = args.new_display()? {
        match nd.size {
            Some((w, h)) => log::info!(
                "nouvel affichage virtuel : {w}x{h}{}",
                nd.dpi.map(|d| format!(" @ {d}dpi")).unwrap_or_default()
            ),
            None => log::info!("nouvel affichage virtuel : dimensions par défaut de l'appareil"),
        }
    }

    // (#14) Audio : validation du codec si l'audio est activé.
    let audio_codec = if args.no_audio {
        log::info!("audio désactivé");
        None
    } else {
        let audio_codec = media::audio::AudioCodec::parse(&args.audio_codec)?;
        log::info!("codec audio demandé : {}", audio_codec.as_str());
        Some(audio_codec)
    };

    // (#19) Enregistrement : valide tôt le conteneur et la compatibilité des
    // codecs (échec immédiat avant tout déploiement si le choix est invalide).
    let recorder = match &args.record {
        Some(path) => {
            let rec = media::recorder::Recorder::create(path, codec, audio_codec)?;
            log::info!(
                "enregistrement activé : {} (conteneur {})",
                rec.path().display(),
                rec.container().extension()
            );
            Some(rec)
        }
        None => None,
    };

    // (#1/#16/#17) Détection de l'appareil (après appairage/connexion réseau).
    device::adb::ensure_available()?;

    // (#17) Appairage Wi-Fi (Android 11+) si demandé, avant la connexion.
    if let Some((addr, code)) = args.pairing() {
        device::connection::pair(addr, code)?;
    }

    connection.establish()?;

    // (#1/#16/#17/#18) Résolution des appareils cibles.
    // En réseau (TCP/IP) la cible est l'adresse connectée ; en USB on peut
    // piloter plusieurs appareils (--serial répété ou --all).
    let targets = match &connection {
        device::Connection::TcpIp(_) => {
            vec![device::adb::detect(connection.serial().as_deref())?]
        }
        device::Connection::Usb { .. } => {
            device::adb::detect_targets(args.target_serials(), args.all)?
        }
    };
    log::info!(
        "{} appareil(s) ciblé(s) : {:?}",
        targets.len(),
        targets.iter().map(|d| &d.serial).collect::<Vec<_>>()
    );

    // (#2/#3/#18) Déploiement du serveur + tunnel, par appareil.
    // Ordre important en mode reverse : push → ouvrir le tunnel (on écoute) →
    // démarrer le serveur (qui se connecte) → accepter la connexion.
    // Chaque appareil reçoit un port local distinct pour éviter les collisions.
    if device::server_bin::is_placeholder() {
        log::warn!(
            "serveur embarqué = placeholder vide (build sans SDK Android) : \
             déploiement sauté. Recompilez avec GNINOU_BUILD_SERVER=1."
        );
    } else {
        let server_args = args.server_args();
        for (i, dev) in targets.iter().enumerate() {
            let serial = Some(dev.serial.as_str());
            let port = device::tunnel::DEFAULT_PORT + i as u16;

            device::server_bin::push(serial)?;
            log::info!("serveur poussé sur {}", dev.serial);

            let tunnel = device::Tunnel::open_reverse(serial, port)?;
            log::info!(
                "[{}] tunnel {:?} ouvert (localabstract:{} ↔ tcp:{})",
                dev.serial,
                tunnel.mode,
                device::tunnel::SOCKET_NAME,
                tunnel.port
            );

            let _server = device::server_bin::start(serial, &server_args)?;
            log::info!("serveur lancé sur {}", dev.serial);

            // #3 : le canal est prêt ; la lecture du handshake arrive avec le protocole.
            let _channel = tunnel.accept()?;
            log::info!("[{}] connexion établie avec le serveur (handshake)", dev.serial);

            // (#22) HID/UHID : créer clavier + souris simulés sur le canal contrôle.
            if args.hid {
                use control::hid;
                let kbd = protocol::ControlMessage::UhidCreate {
                    id: hid::KEYBOARD_ID,
                    report_desc: hid::KEYBOARD_REPORT_DESC.to_vec(),
                };
                let mouse = protocol::ControlMessage::UhidCreate {
                    id: hid::MOUSE_ID,
                    report_desc: hid::MOUSE_REPORT_DESC.to_vec(),
                };
                log::info!(
                    "[{}] HID activé : clavier (#{}, {} o) + souris (#{}, {} o) prêts à créer",
                    dev.serial,
                    hid::KEYBOARD_ID,
                    kbd.encode().len(),
                    hid::MOUSE_ID,
                    mouse.encode().len()
                );
                // TODO (#8-10) : émettre kbd/mouse puis les UhidInput au fil des entrées.
            }

            // (#23) Manette : périphérique HID supplémentaire via le même transport UHID.
            if args.gamepad {
                use control::gamepad;
                let pad = protocol::ControlMessage::UhidCreate {
                    id: gamepad::GAMEPAD_ID,
                    report_desc: gamepad::GAMEPAD_REPORT_DESC.to_vec(),
                };
                log::info!(
                    "[{}] manette activée (#{}, {} o de descripteur) prête à créer",
                    dev.serial,
                    gamepad::GAMEPAD_ID,
                    pad.encode().len()
                );
                // TODO (#8-10) : émettre `pad` puis les UhidInput (GamepadState::report).
            }

            // (#20) Écran éteint : message de contrôle prêt à émettre sur le canal.
            if args.turn_screen_off {
                let msg = protocol::ControlMessage::SetScreenPowerMode {
                    mode: protocol::screen_power::OFF,
                };
                log::info!(
                    "[{}] écran éteint demandé ({} octets de contrôle en attente du canal)",
                    dev.serial,
                    msg.encode().len()
                );
                // TODO (#8-10) : émettre `msg` sur le canal contrôle, puis restaurer
                //   screen_power::NORMAL en fin de session.
            }

            // TODO (suite du pipeline, par appareil) :
            //   #4-7.  media::video + media::audio (décodage/lecture)
            //   #8-10. control (capture entrées -> protocol -> socket)
        }
    }

    // (#19) Finalise le fichier d'enregistrement (écriture du trailer).
    if let Some(rec) = recorder {
        rec.finish()?;
    }

    log::warn!("média et contrôle non implémentés — prochaines fonctionnalités");

    Ok(())
}
