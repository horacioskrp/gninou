//! Définition de la ligne de commande.

use anyhow::{bail, Result};
use clap::Parser;

use crate::device::Connection;
use crate::geometry::{parse_rotation, Crop, NewDisplay};
use crate::media::camera::{CameraConfig, VideoSource};

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

/// Périphériques HID à exposer en mode OTG. Logique **pure et testable**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OtgPlan {
    pub keyboard: bool,
    pub mouse: bool,
    pub gamepad: bool,
}

impl OtgPlan {
    /// Au moins un périphérique est demandé.
    pub fn any(&self) -> bool {
        self.keyboard || self.mouse || self.gamepad
    }
}

/// Valide la cohérence des options en mode OTG (aucun flux média).
/// Logique **pure et testable**.
pub fn validate_otg(
    otg: bool,
    has_record: bool,
    has_display: bool,
    has_new_display: bool,
    has_v4l2: bool,
    no_audio: bool,
) -> Result<()> {
    if !otg {
        return Ok(());
    }
    if has_record {
        bail!("--otg est incompatible avec --record (pas de flux vidéo en OTG)");
    }
    if has_display || has_new_display {
        bail!("--otg est incompatible avec --display / --new-display (pas de miroir en OTG)");
    }
    if has_v4l2 {
        bail!("--otg est incompatible avec --v4l2-sink (pas de flux vidéo en OTG)");
    }
    let _ = no_audio; // l'audio est de toute façon désactivé en OTG
    Ok(())
}

/// Afficher et contrôler un appareil Android depuis le bureau.
#[derive(Parser, Debug)]
#[command(name = "gninou", version, about, long_about = None)]
pub struct Cli {
    /// Numéro de série d'un appareil ciblé (adb -s). Répétable pour piloter
    /// plusieurs appareils : --serial A --serial B. Optionnel.
    #[arg(short, long, value_name = "SERIAL")]
    pub serial: Vec<String>,

    /// Piloter tous les appareils en ligne (incompatible avec --serial / réseau).
    #[arg(long, default_value_t = false, conflicts_with_all = ["serial", "tcpip", "pair"])]
    pub all: bool,

    /// Se connecter en TCP/IP (Wi-Fi) à cette adresse `ip:port`.
    /// Ex. : --tcpip 192.168.1.42:5555
    #[arg(long, value_name = "IP:PORT")]
    pub tcpip: Option<String>,

    /// Appairage sans fil (Android 11+) : adresse `ip:port` du dialogue
    /// « Associer l'appareil à l'aide d'un code ». À combiner avec --pair-code
    /// puis --tcpip pour la connexion. Ex. : --pair 192.168.1.42:37123
    #[arg(long, value_name = "IP:PORT", requires = "pair_code")]
    pub pair: Option<String>,

    /// Code d'appairage à 6 chiffres affiché par l'appareil (avec --pair).
    #[arg(long, value_name = "CODE", requires = "pair")]
    pub pair_code: Option<String>,

    /// Wi-Fi automatique : bascule un appareil branché en USB vers le sans-fil
    /// (lit son IP, active adb tcpip, se connecte) sans saisir d'adresse.
    #[arg(long, default_value_t = false, conflicts_with_all = ["tcpip", "pair", "all"])]
    pub wifi: bool,

    /// Port TCP utilisé par --wifi pour la bascule sans-fil.
    #[arg(long, value_name = "PORT", default_value_t = 5555)]
    pub wifi_port: u16,

    /// Découverte mDNS : se connecter au premier appareil en « Débogage sans
    /// fil » annoncé sur le réseau (Android 11+), sans câble ni IP.
    #[arg(long, default_value_t = false, conflicts_with_all = ["tcpip", "pair", "wifi", "all"])]
    pub wifi_discover: bool,

    /// Assistant d'appairage interactif (Android 11+) : guide pas à pas,
    /// découvre les points d'appairage/connexion en mDNS (repli manuel) et ne
    /// demande que le code à 6 chiffres affiché par l'appareil.
    #[arg(long, default_value_t = false, conflicts_with_all = ["tcpip", "pair", "wifi", "wifi_discover", "all"])]
    pub pair_assistant: bool,

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

    /// Enregistrer le flux dans un fichier (.mp4 ou .mkv) en plus de l'affichage.
    #[arg(long, value_name = "FICHIER")]
    pub record: Option<std::path::PathBuf>,

    /// Éteindre l'écran de l'appareil pendant le miroir (rallumé en fin de session).
    #[arg(long, default_value_t = false)]
    pub turn_screen_off: bool,

    /// Refléter un affichage existant par son identifiant (0 = écran principal).
    #[arg(long, value_name = "ID", conflicts_with = "new_display")]
    pub display: Option<u32>,

    /// Créer un nouvel affichage virtuel (Android 10+). Taille optionnelle
    /// `LxH` ou `LxH/dpi` (ex. 1920x1080/420) ; sinon défauts de l'appareil.
    #[arg(long, value_name = "LxH[/dpi]", num_args = 0..=1, default_missing_value = "")]
    pub new_display: Option<String>,

    /// Simuler un clavier/souris physiques (HID/UHID) au lieu de l'injection logicielle.
    #[arg(long, default_value_t = false)]
    pub hid: bool,

    /// Exposer une manette (gamepad) HID à l'appareil et y mapper le contrôleur du PC.
    #[arg(long, default_value_t = false)]
    pub gamepad: bool,

    /// Mode OTG : contrôle HID pur (clavier/souris/manette), sans miroir ni audio.
    #[arg(long, default_value_t = false)]
    pub otg: bool,

    /// Source vidéo : `display` (écran, défaut) ou `camera` (Android 12+).
    #[arg(long, value_name = "SOURCE", default_value = "display")]
    pub video_source: String,

    /// Identifiant de caméra explicite (prioritaire sur --camera-facing).
    #[arg(long, value_name = "ID")]
    pub camera_id: Option<String>,

    /// Orientation de la caméra : `back`, `front` ou `external`.
    #[arg(long, value_name = "FACING")]
    pub camera_facing: Option<String>,

    /// Taille de capture caméra `LxH` (ex. 1280x720).
    #[arg(long, value_name = "LxH")]
    pub camera_size: Option<String>,

    /// Images par seconde de la caméra.
    #[arg(long, value_name = "FPS")]
    pub camera_fps: Option<u32>,

    /// Exposer le flux comme webcam virtuelle via un périphérique V4L2 (Linux).
    /// Ex. : --v4l2-sink /dev/video2
    #[arg(long, value_name = "/dev/videoN")]
    pub v4l2_sink: Option<String>,

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

    /// Appairage Wi-Fi demandé : `(adresse, code)`. `None` si non demandé.
    /// clap garantit que les deux options vont de pair (`requires`).
    pub fn pairing(&self) -> Option<(&str, &str)> {
        match (&self.pair, &self.pair_code) {
            (Some(addr), Some(code)) => Some((addr.as_str(), code.as_str())),
            _ => None,
        }
    }

    /// Numéros de série explicitement ciblés (peut être vide).
    pub fn target_serials(&self) -> &[String] {
        &self.serial
    }

    /// Détermine le mode de connexion demandé.
    pub fn connection(&self) -> Connection {
        if let Some(addr) = &self.tcpip {
            Connection::TcpIp(addr.clone())
        } else {
            // En USB, le premier série (le cas échéant) sert à cibler la connexion ;
            // la liste complète pilote le multi-appareils (voir target_serials()).
            Connection::Usb {
                serial: self.serial.first().cloned(),
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

    /// Spécification d'affichage virtuel demandée, validée. `None` si non demandé.
    pub fn new_display(&self) -> Result<Option<NewDisplay>> {
        match &self.new_display {
            Some(spec) => Ok(Some(NewDisplay::parse(spec)?)),
            None => Ok(None),
        }
    }

    /// Configuration caméra demandée, validée. `None` si la source est l'écran.
    pub fn camera(&self) -> Result<Option<CameraConfig>> {
        match VideoSource::parse(&self.video_source)? {
            VideoSource::Display => Ok(None),
            VideoSource::Camera => Ok(Some(CameraConfig::build(
                self.camera_id.clone(),
                self.camera_facing.as_deref(),
                self.camera_size.as_deref(),
                self.camera_fps,
            )?)),
        }
    }

    /// Le clavier/souris HID sont-ils actifs ? (explicitement via `--hid`, ou
    /// implicitement en mode OTG).
    pub fn hid_enabled(&self) -> bool {
        self.hid || self.otg
    }

    /// Plan des périphériques HID à créer en mode OTG.
    pub fn otg_plan(&self) -> OtgPlan {
        OtgPlan {
            keyboard: true,
            mouse: true,
            gamepad: self.gamepad,
        }
    }

    /// Périphérique webcam virtuelle demandé, validé. `None` si non demandé.
    pub fn v4l2_sink(&self) -> Result<Option<std::path::PathBuf>> {
        match &self.v4l2_sink {
            Some(dev) => Ok(Some(crate::media::v4l2::parse_device(dev)?)),
            None => Ok(None),
        }
    }

    /// Valide la cohérence des options OTG. Erreur si incompatibles.
    pub fn check_otg(&self) -> Result<()> {
        validate_otg(
            self.otg,
            self.record.is_some(),
            self.display.is_some(),
            self.new_display.is_some(),
            self.v4l2_sink.is_some(),
            self.no_audio,
        )
    }

    /// Arguments transmis au serveur embarqué (format `clé=valeur`).
    pub fn server_args(&self) -> Vec<String> {
        // (#24) En OTG : contrôle seul, ni vidéo ni audio.
        if self.otg {
            return vec!["video=false".to_string(), "audio=false".to_string()];
        }
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
        // (#21) Sélection d'affichage / affichage virtuel.
        if let Some(id) = self.display {
            v.push(format!("display_id={id}"));
        }
        if let Ok(Some(nd)) = self.new_display() {
            v.push(format!("new_display={}", nd.to_arg()));
        }
        // (#25) Source caméra : ajoute video_source=camera + réglages caméra.
        if let Ok(Some(cam)) = self.camera() {
            v.extend(cam.server_args());
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_bitrate, validate_otg};

    #[test]
    fn otg_sans_conflit_ok() {
        // Hors OTG : jamais d'erreur quels que soient les autres drapeaux.
        assert!(validate_otg(false, true, true, true, true, false).is_ok());
        // OTG seul : ok.
        assert!(validate_otg(true, false, false, false, false, true).is_ok());
    }

    #[test]
    fn otg_conflits_rejetes() {
        // OTG + record
        assert!(validate_otg(true, true, false, false, false, false).is_err());
        // OTG + display
        assert!(validate_otg(true, false, true, false, false, false).is_err());
        // OTG + new_display
        assert!(validate_otg(true, false, false, true, false, false).is_err());
        // OTG + v4l2-sink
        assert!(validate_otg(true, false, false, false, true, false).is_err());
    }

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
