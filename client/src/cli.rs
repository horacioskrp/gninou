//! Définition de la ligne de commande.

use clap::Parser;

/// Afficher et contrôler un appareil Android depuis le bureau.
#[derive(Parser, Debug)]
#[command(name = "gninou", version, about, long_about = None)]
pub struct Cli {
    /// Numéro de série de l'appareil ciblé (adb -s). Optionnel.
    #[arg(short, long)]
    pub serial: Option<String>,

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
}
