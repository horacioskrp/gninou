//! View-model **pur** de l'écran « Options du miroir » (#35).
//!
//! Compose et **valide en direct** les réglages vidéo du miroir — résolution
//! maximale, FPS, codec, débit, rognage et rotation (#11/#12/#13) — en
//! réutilisant les analyseurs purs déjà en place : [`parse_bitrate`],
//! [`Crop::parse`] et [`VideoCodec`]. L'écran ne lance rien lui-même : il
//! produit les arguments transmis au serveur ([`OptionsState::server_args`]) et
//! un aperçu de la commande équivalente ([`OptionsState::cli_preview`]). Comme
//! les autres view-models GUI, tout est pur et testé dans le build par défaut.

use crate::cli::parse_bitrate;
use crate::geometry::Crop;
use crate::media::codec::VideoCodec;

/// Rotations proposées (degrés), dans l'ordre d'affichage.
pub const ROTATIONS: [u16; 4] = [0, 90, 180, 270];

/// FPS par défaut (aligné sur le CLI).
const DEFAULT_FPS: &str = "60";

/// Messages émis par l'UI (éditions/actions).
#[derive(Debug, Clone)]
pub enum OptMsg {
    MaxSizeChanged(String),
    MaxFpsChanged(String),
    CodecSelected(VideoCodec),
    BitRateChanged(String),
    CropChanged(String),
    RotationSelected(u16),
    Reset,
}

/// État de l'écran Options du miroir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsState {
    /// Résolution maximale (bord le plus long) ; « 0 » = illimité.
    pub max_size: String,
    /// Images par seconde maximales.
    pub max_fps: String,
    /// Codec vidéo choisi.
    pub codec: VideoCodec,
    /// Débit cible (ex. « 8M », « 1500K ») ; vide = automatique.
    pub bit_rate: String,
    /// Rognage `W:H:X:Y` ; vide = aucun.
    pub crop: String,
    /// Rotation en degrés (0/90/180/270).
    pub rotation: u16,
}

impl Default for OptionsState {
    fn default() -> Self {
        OptionsState {
            max_size: "0".to_string(),
            max_fps: DEFAULT_FPS.to_string(),
            codec: VideoCodec::H264,
            bit_rate: String::new(),
            crop: String::new(),
            rotation: 0,
        }
    }
}

impl OptionsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Résolution maximale saisie, si c'est un entier valide (0 accepté). **Pur.**
    pub fn max_size_num(&self) -> Option<u32> {
        self.max_size.trim().parse::<u32>().ok()
    }

    /// FPS saisis, s'ils forment un entier ≥ 1. **Pur.**
    pub fn max_fps_num(&self) -> Option<u32> {
        match self.max_fps.trim().parse::<u32>() {
            Ok(n) if n >= 1 => Some(n),
            _ => None,
        }
    }

    /// Le débit saisi est-il acceptable (vide = automatique, sinon bien formé) ?
    pub fn bit_rate_valid(&self) -> bool {
        let s = self.bit_rate.trim();
        s.is_empty() || parse_bitrate(s).is_ok()
    }

    /// Débit en bits/s : 0 si vide ou illisible, sinon la valeur analysée. **Pur.**
    pub fn bit_rate_num(&self) -> u32 {
        let s = self.bit_rate.trim();
        if s.is_empty() {
            0
        } else {
            parse_bitrate(s).unwrap_or(0)
        }
    }

    /// Rectangle de rognage demandé : `Ok(None)` si vide, sinon analysé. **Pur.**
    pub fn crop_parsed(&self) -> anyhow::Result<Option<Crop>> {
        let s = self.crop.trim();
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Crop::parse(s)?))
        }
    }

    /// Le rognage saisi est-il acceptable (vide, ou `W:H:X:Y` valide) ?
    pub fn crop_valid(&self) -> bool {
        self.crop_parsed().is_ok()
    }

    pub fn max_size_valid(&self) -> bool {
        self.max_size_num().is_some()
    }

    pub fn max_fps_valid(&self) -> bool {
        self.max_fps_num().is_some()
    }

    /// Tous les champs sont-ils valides ? (conditionne le bouton d'application).
    pub fn is_valid(&self) -> bool {
        self.max_size_valid() && self.max_fps_valid() && self.bit_rate_valid() && self.crop_valid()
    }

    /// Arguments `clé=valeur` transmis au serveur, dans le même format que le
    /// CLI. Les champs vides/à zéro sont omis ; les champs invalides sont
    /// ignorés (l'UI empêche l'application tant que [`is_valid`] est faux).
    pub fn server_args(&self) -> Vec<String> {
        let mut v = Vec::new();
        if let Some(ms) = self.max_size_num() {
            if ms > 0 {
                v.push(format!("max_size={ms}"));
            }
        }
        if let Some(fps) = self.max_fps_num() {
            v.push(format!("max_fps={fps}"));
        }
        v.push(format!("video_codec={}", self.codec.as_str()));
        let br = self.bit_rate_num();
        if br > 0 {
            v.push(format!("bit_rate={br}"));
        }
        if let Ok(Some(crop)) = self.crop_parsed() {
            v.push(format!("crop={}", crop.to_arg()));
        }
        if self.rotation != 0 {
            v.push(format!("rotation={}", self.rotation));
        }
        v
    }

    /// Aperçu lisible de la commande CLI équivalente aux réglages actuels.
    /// Les champs invalides sont omis de l'aperçu. **Pur.**
    pub fn cli_preview(&self) -> String {
        let mut parts = vec!["gninou".to_string()];
        if let Some(ms) = self.max_size_num() {
            if ms > 0 {
                parts.push(format!("--max-size {ms}"));
            }
        }
        if let Some(fps) = self.max_fps_num() {
            parts.push(format!("--max-fps {fps}"));
        }
        parts.push(format!("--video-codec {}", self.codec.as_str()));
        let bitrate = self.bit_rate.trim();
        if !bitrate.is_empty() && self.bit_rate_valid() {
            parts.push(format!("--bit-rate {bitrate}"));
        }
        let crop = self.crop.trim();
        if !crop.is_empty() && self.crop_valid() {
            parts.push(format!("--crop {crop}"));
        }
        if self.rotation != 0 {
            parts.push(format!("--rotation {}", self.rotation));
        }
        parts.join(" ")
    }

    /// Applique un message et fait évoluer l'état. **Pur et testable.**
    pub fn reduce(&mut self, msg: OptMsg) {
        match msg {
            OptMsg::MaxSizeChanged(s) => self.max_size = s,
            OptMsg::MaxFpsChanged(s) => self.max_fps = s,
            OptMsg::CodecSelected(c) => self.codec = c,
            OptMsg::BitRateChanged(s) => self.bit_rate = s,
            OptMsg::CropChanged(s) => self.crop = s,
            OptMsg::RotationSelected(r) => self.rotation = r,
            OptMsg::Reset => *self = OptionsState::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etat_initial_valide() {
        let s = OptionsState::new();
        assert!(s.is_valid());
        assert_eq!(s.max_size, "0");
        assert_eq!(s.max_fps, "60");
        assert_eq!(s.codec, VideoCodec::H264);
        assert_eq!(s.rotation, 0);
    }

    #[test]
    fn args_par_defaut_minimalistes() {
        // max_size=0 (omis), pas de débit ni crop ni rotation : seuls fps + codec.
        let s = OptionsState::new();
        assert_eq!(s.server_args(), vec!["max_fps=60", "video_codec=h264"]);
    }

    #[test]
    fn max_size_zero_omis_mais_valide() {
        let s = OptionsState::new();
        assert!(s.max_size_valid());
        assert!(!s.server_args().iter().any(|a| a.starts_with("max_size=")));
    }

    #[test]
    fn max_size_renseigne_present() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::MaxSizeChanged("1280".into()));
        assert!(s.server_args().contains(&"max_size=1280".to_string()));
    }

    #[test]
    fn max_size_invalide() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::MaxSizeChanged("abc".into()));
        assert!(!s.max_size_valid());
        assert!(!s.is_valid());
    }

    #[test]
    fn fps_doit_etre_au_moins_un() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::MaxFpsChanged("0".into()));
        assert!(!s.max_fps_valid());
        s.reduce(OptMsg::MaxFpsChanged("30".into()));
        assert_eq!(s.max_fps_num(), Some(30));
    }

    #[test]
    fn codec_dans_les_args_et_apercu() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::CodecSelected(VideoCodec::H265));
        assert!(s.server_args().contains(&"video_codec=h265".to_string()));
        assert!(s.cli_preview().contains("--video-codec h265"));
    }

    #[test]
    fn debit_vide_automatique() {
        let s = OptionsState::new();
        assert!(s.bit_rate_valid()); // vide accepté
        assert_eq!(s.bit_rate_num(), 0);
        assert!(!s.server_args().iter().any(|a| a.starts_with("bit_rate=")));
    }

    #[test]
    fn debit_suffixe_m() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::BitRateChanged("8M".into()));
        assert!(s.bit_rate_valid());
        assert_eq!(s.bit_rate_num(), 8_000_000);
        assert!(s.server_args().contains(&"bit_rate=8000000".to_string()));
        assert!(s.cli_preview().contains("--bit-rate 8M"));
    }

    #[test]
    fn debit_invalide() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::BitRateChanged("1.5M".into()));
        assert!(!s.bit_rate_valid());
        assert!(!s.is_valid());
        // Omis de l'aperçu tant qu'il est invalide.
        assert!(!s.cli_preview().contains("--bit-rate"));
    }

    #[test]
    fn crop_valide_et_invalide() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::CropChanged("1080:2400:0:0".into()));
        assert!(s.crop_valid());
        assert!(s.server_args().contains(&"crop=1080:2400:0:0".to_string()));
        s.reduce(OptMsg::CropChanged("1080:2400".into()));
        assert!(!s.crop_valid());
        assert!(!s.is_valid());
    }

    #[test]
    fn rotation_non_nulle_presente() {
        let mut s = OptionsState::new();
        assert!(!s.server_args().iter().any(|a| a.starts_with("rotation=")));
        s.reduce(OptMsg::RotationSelected(90));
        assert!(s.server_args().contains(&"rotation=90".to_string()));
        assert!(s.cli_preview().contains("--rotation 90"));
    }

    #[test]
    fn apercu_complet() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::MaxSizeChanged("1920".into()));
        s.reduce(OptMsg::MaxFpsChanged("30".into()));
        s.reduce(OptMsg::CodecSelected(VideoCodec::Av1));
        s.reduce(OptMsg::BitRateChanged("6M".into()));
        s.reduce(OptMsg::RotationSelected(270));
        assert_eq!(
            s.cli_preview(),
            "gninou --max-size 1920 --max-fps 30 --video-codec av1 --bit-rate 6M --rotation 270"
        );
    }

    #[test]
    fn reset_revient_a_zero() {
        let mut s = OptionsState::new();
        s.reduce(OptMsg::MaxSizeChanged("800".into()));
        s.reduce(OptMsg::CodecSelected(VideoCodec::H265));
        s.reduce(OptMsg::Reset);
        assert_eq!(s, OptionsState::new());
    }

    #[test]
    fn rotations_proposees() {
        assert_eq!(ROTATIONS, [0, 90, 180, 270]);
    }
}
