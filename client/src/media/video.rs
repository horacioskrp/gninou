//! Décodage et affichage vidéo.
//!
//! Décodage prévu via FFmpeg (crate `ffmpeg-next`), rendu via SDL3
//! (ou `wgpu`/`winit` à terme). Squelette : interface à venir.

use anyhow::Result;

/// Pipeline vidéo : trames encodées -> frames affichées.
pub struct VideoPipeline {
    _private: (),
}

impl VideoPipeline {
    /// Initialise le décodeur et la fenêtre d'affichage.
    pub fn new(_codec: &str) -> Result<Self> {
        // TODO: ouvrir le décodeur ffmpeg + créer la fenêtre SDL
        Ok(Self { _private: () })
    }

    /// Décode une trame encodée et l'affiche.
    pub fn push_packet(&mut self, _data: &[u8]) -> Result<()> {
        // TODO: decode -> upload texture -> présenter
        Ok(())
    }
}
