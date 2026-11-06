//! Lecture audio.
//!
//! Décodage des trames audio (Opus/AAC) et restitution sur la sortie son
//! du PC. Squelette : interface à venir.

use anyhow::Result;

/// Pipeline audio : trames encodées -> haut-parleurs.
pub struct AudioPipeline {
    _private: (),
}

impl AudioPipeline {
    pub fn new(_codec: &str) -> Result<Self> {
        // TODO: ouvrir le décodeur + le périphérique de sortie audio
        Ok(Self { _private: () })
    }

    pub fn push_packet(&mut self, _data: &[u8]) -> Result<()> {
        // TODO: decode -> buffer -> lecture
        Ok(())
    }
}
