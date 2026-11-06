//! Audio : codec, réception et lecture.
//!
//! La **sélection du codec** et les statistiques du flux sont pures et
//! testées. Le décodage et la restitution sonore réels dépendront d'une lib
//! audio (ajout ultérieur, derrière une feature) ; ici le pipeline tient les
//! compteurs, utile au câblage et aux tests.

use anyhow::{bail, Result};

/// Codec audio géré.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    Opus,
    Aac,
    /// PCM brut (non compressé).
    Raw,
}

impl AudioCodec {
    pub const ALL: [AudioCodec; 3] = [AudioCodec::Opus, AudioCodec::Aac, AudioCodec::Raw];

    pub fn as_str(self) -> &'static str {
        match self {
            AudioCodec::Opus => "opus",
            AudioCodec::Aac => "aac",
            AudioCodec::Raw => "raw",
        }
    }

    pub fn parse(id: &str) -> Result<AudioCodec> {
        match id.to_ascii_lowercase().as_str() {
            "opus" => Ok(AudioCodec::Opus),
            "aac" => Ok(AudioCodec::Aac),
            "raw" | "pcm" => Ok(AudioCodec::Raw),
            other => bail!(
                "codec audio inconnu « {other} » (attendus : {})",
                Self::ALL.map(|c| c.as_str()).join(", ")
            ),
        }
    }
}

/// Statistiques du flux audio.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub packets: u64,
    pub bytes: u64,
}

/// Pipeline audio : trames encodées -> (à terme) haut-parleurs.
pub struct AudioPipeline {
    codec: AudioCodec,
    stats: Stats,
}

impl AudioPipeline {
    pub fn new(codec: AudioCodec) -> Self {
        Self { codec, stats: Stats::default() }
    }

    pub fn codec(&self) -> AudioCodec {
        self.codec
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Alimente le pipeline avec une trame audio encodée.
    pub fn push_packet(&mut self, data: &[u8]) -> Result<()> {
        self.stats.packets += 1;
        self.stats.bytes += data.len() as u64;
        // TODO (feature audio) : décoder (Opus/AAC) + restituer sur la sortie son.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_codecs() {
        assert_eq!(AudioCodec::parse("opus").unwrap(), AudioCodec::Opus);
        assert_eq!(AudioCodec::parse("AAC").unwrap(), AudioCodec::Aac);
        assert_eq!(AudioCodec::parse("pcm").unwrap(), AudioCodec::Raw);
        assert!(AudioCodec::parse("mp3").is_err());
    }

    #[test]
    fn aller_retour_as_str() {
        for c in AudioCodec::ALL {
            assert_eq!(AudioCodec::parse(c.as_str()).unwrap(), c);
        }
    }

    #[test]
    fn pipeline_cumule_les_stats() {
        let mut p = AudioPipeline::new(AudioCodec::Opus);
        p.push_packet(&[0u8; 100]).unwrap();
        p.push_packet(&[0u8; 40]).unwrap();
        assert_eq!(p.stats(), Stats { packets: 2, bytes: 140 });
        assert_eq!(p.codec(), AudioCodec::Opus);
    }
}
