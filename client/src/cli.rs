//! Définition de la ligne de commande.

use clap::Parser;

use crate::device::Connection;

/// Afficher et contrôler un appareil Android depuis le bureau.
#[derive(Parser, Debug)]
#[command(name = "gninou", version, about, long_about = None)]
pub struct Cli {
    /// Numéro de série de l'appareil ciblé (adb -s). Optionnel.
    #[arg(short, long)]
    pub serial: Option<String>,

    /// Se connecter en TCP/IP (Wi-Fi) à cette adresse `ip:port`.
    /// Ex. : --tcpip 192.168.1.42:5555
    #[arg(long, value_name = "IP:PORT")]
    pub tcpip: Option<String>,

    /// Résolution maximale (bord le plus long, en pixels). 0 = illimité.
    #[arg(short = 'm', long, default_value_t = 0)]
    pub max_size: u32,

    /// Images par seconde maximales.
    #[arg(long, default_value_t = 60)]
    pub max_fps: u32,

    /// Codec vidéo souhaité.
    #[arg(long, default_value = "h264")]
    pub video_codec: String,

    /// Désactiver l'audio.
    #[arg(long, default_value_t = false)]
    pub no_audio: bool,

    /// Désactiver la vidéo (contrôle/audio seuls).
    #[arg(long, default_value_t = false)]
    pub no_video: bool,

    /// Verbosité (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Cli {
    pub fn log_level(&self) -> log::LevelFilter {
        match self.verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    }

    /// Détermine le mode de connexion demandé.
    pub fn connection(&self) -> Connection {
        if let Some(addr) = &self.tcpip {
            Connection::TcpIp(addr.clone())
        } else {
            Connection::Usb {
                serial: self.serial.clone(),
            }
        }
    }
}
