//! Définition de la ligne de commande.

use anyhow::{bail, Result};
use clap::Parser;

use crate::device::Connection;
use crate::geometry::{parse_rotation, Crop};

/// Analyse un débit avec suffixe optionnel `K`/`M` (base 1000).
/// Ex. : `"8M"` -> 8_000_000, `"1500K"` -> 1_500_000, `"4000000"` -> 4_000_000.
/// Logique pure et testable.
pub fn parse_bitrate(s: &str) -> Result<u32> {
    let s = s.trim();
    if s.is_empty() {
        bail!("débit vide");
    }
    let (num, mult) = match s.chars().last().unwrap() {
        'k' | 'K' => (&s[..s.len() - 1], 1_000u64),
        'm' | 'M' => (&s[..s.len() - 1], 1_000_000u64),
        _ => (s, 1u64),
    };
    let base: u64 = num
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("débit invalide : « {s} »"))?;
    let value = base
        .checked_mul(mult)
        .filter(|v| *v <= u32::MAX as u64)
        .ok_or_else(|| anyhow::anyhow!("débit hors limites : « {s} »"))?;
    Ok(value as u32)
}

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

    /// Débit vidéo cible (ex. 8M, 1500K). Vide = automatique selon la définition.
    #[arg(long, value_name = "DÉBIT")]
    pub bit_rate: Option<String>,

    /// Rogner la capture : `W:H:X:Y` (en pixels). Ex. : 1080:2400:0:0
    #[arg(long, value_name = "W:H:X:Y")]
    pub crop: Option<String>,

    /// Rotation de l'affichage : 0, 90, 180 ou 270.
    #[arg(long, default_value = "0")]
    pub rotation: String,

    /// Désactiver l'audio.
    #[arg(long, default_value_t = false)]
    pub no_audio: bool,

    /// Codec audio souhaité (opus, aac, raw).
    #[arg(long, default_value = "opus")]
    pub audio_codec: String,

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

    /// Débit vidéo demandé en bits/s (0 = automatique). Erreur si mal formé.
    pub fn bit_rate(&self) -> Result<u32> {
        match &self.bit_rate {
            Some(s) => parse_bitrate(s),
            None => Ok(0),
        }
    }

    /// Rectangle de rognage demandé, validé. `None` si non demandé.
    pub fn crop(&self) -> Result<Option<Crop>> {
        match &self.crop {
            Some(s) => Ok(Some(Crop::parse(s)?)),
            None => Ok(None),
        }
    }

    /// Rotation demandée en degrés (0/90/180/270). Erreur si invalide.
    pub fn rotation(&self) -> Result<u16> {
        parse_rotation(&self.rotation)
    }

    /// Arguments transmis au serveur embarqué (format `clé=valeur`).
    pub fn server_args(&self) -> Vec<String> {
        let mut v = Vec::new();
        if self.max_size > 0 {
            v.push(format!("max_size={}", self.max_size));
        }
        v.push(format!("max_fps={}", self.max_fps));
        v.push(format!("video_codec={}", self.video_codec));
        if let Ok(br) = self.bit_rate() {
            if br > 0 {
                v.push(format!("bit_rate={br}"));
            }
        }
        if let Ok(Some(crop)) = self.crop() {
            v.push(format!("crop={}", crop.to_arg()));
        }
        if let Ok(rot) = self.rotation() {
            if rot != 0 {
                v.push(format!("rotation={rot}"));
            }
        }
        if self.no_audio {
            v.push("audio=false".to_string());
        } else {
            v.push(format!("audio_codec={}", self.audio_codec));
        }
        if self.no_video {
            v.push("video=false".to_string());
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::parse_bitrate;

    #[test]
    fn bitrate_entier_nu() {
        assert_eq!(parse_bitrate("4000000").unwrap(), 4_000_000);
    }

    #[test]
    fn bitrate_suffixe_k_et_m() {
        assert_eq!(parse_bitrate("1500K").unwrap(), 1_500_000);
        assert_eq!(parse_bitrate("8M").unwrap(), 8_000_000);
        assert_eq!(parse_bitrate("8m").unwrap(), 8_000_000);
    }

    #[test]
    fn bitrate_invalide() {
        assert!(parse_bitrate("").is_err());
        assert!(parse_bitrate("abc").is_err());
        assert!(parse_bitrate("1.5M").is_err());
    }

    #[test]
    fn bitrate_hors_limites() {
        assert!(parse_bitrate("99999M").is_err()); // > u32::MAX
    }
}
