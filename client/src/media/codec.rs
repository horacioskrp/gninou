//! Codecs vidéo gérés côté client (validation du choix utilisateur).
//!
//! La correspondance avec les types MIME et la négociation réelle de
//! disponibilité se font côté serveur ; ici on valide/normalise l'identifiant
//! passé en ligne de commande. Logique **pure et testable**.

use anyhow::{bail, Result};

/// Codec vidéo demandé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
}

impl VideoCodec {
    /// Tous les codecs gérés, dans l'ordre de préférence par défaut.
    pub const ALL: [VideoCodec; 3] = [VideoCodec::H264, VideoCodec::H265, VideoCodec::Av1];

    /// Identifiant stable (celui transmis au serveur).
    pub fn as_str(self) -> &'static str {
        match self {
            VideoCodec::H264 => "h264",
            VideoCodec::H265 => "h265",
            VideoCodec::Av1 => "av1",
        }
    }

    /// Analyse un identifiant (insensible à la casse) ; erreur si inconnu.
    pub fn parse(id: &str) -> Result<VideoCodec> {
        match id.to_ascii_lowercase().as_str() {
            "h264" | "avc" => Ok(VideoCodec::H264),
            "h265" | "hevc" => Ok(VideoCodec::H265),
            "av1" => Ok(VideoCodec::Av1),
            other => bail!(
                "codec vidéo inconnu « {other} » (attendus : {})",
                Self::ALL.map(|c| c.as_str()).join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connus() {
        assert_eq!(VideoCodec::parse("h264").unwrap(), VideoCodec::H264);
        assert_eq!(VideoCodec::parse("H265").unwrap(), VideoCodec::H265);
        assert_eq!(VideoCodec::parse("hevc").unwrap(), VideoCodec::H265);
        assert_eq!(VideoCodec::parse("av1").unwrap(), VideoCodec::Av1);
    }

    #[test]
    fn parse_inconnu_erreur() {
        assert!(VideoCodec::parse("vp9").is_err());
    }

    #[test]
    fn aller_retour_as_str() {
        for c in VideoCodec::ALL {
            assert_eq!(VideoCodec::parse(c.as_str()).unwrap(), c);
        }
    }
}
