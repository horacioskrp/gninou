//! Webcam virtuelle V4L2 (#26, Linux).
//!
//! Écrit le flux décodé dans un périphérique de rebouclage vidéo
//! (`v4l2loopback`, ex. `/dev/video2`) : les autres applications du PC
//! (visioconférence, OBS…) voient alors l'appareil Android comme une webcam.
//!
//! La **validation du périphérique** et le **comptage des trames** sont purs et
//! testés. L'écriture réelle (ouverture `/dev/videoN`, `VIDIOC_S_FMT`, écriture
//! des trames) est spécifique à Linux et se branche derrière la feature `v4l2`.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// Indique si la cible de compilation gère la sortie V4L2 (Linux uniquement).
pub const fn is_supported() -> bool {
    cfg!(target_os = "linux")
}

/// Valide un chemin de périphérique de rebouclage `/dev/videoN`.
/// Logique **pure et testable** (indépendante de l'OS hôte).
pub fn parse_device(path: &str) -> Result<PathBuf> {
    let path = path.trim();
    let rest = path
        .strip_prefix("/dev/video")
        .ok_or_else(|| anyhow::anyhow!("périphérique V4L2 attendu « /dev/videoN », reçu « {path} »"))?;
    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_digit()) {
        bail!("numéro de périphérique V4L2 invalide dans « {path} »");
    }
    Ok(PathBuf::from(path))
}

/// Statistiques d'une sortie webcam virtuelle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub frames: u64,
    pub bytes: u64,
}

/// Sortie webcam virtuelle : reçoit les trames décodées et (à terme) les écrit
/// dans le périphérique de rebouclage.
pub struct V4l2Sink {
    device: PathBuf,
    size: (u32, u32),
    stats: Stats,
}

impl V4l2Sink {
    /// Prépare une sortie vers `device`, après validation du chemin et de la
    /// prise en charge par la plateforme.
    pub fn create(device: &str, width: u32, height: u32) -> Result<Self> {
        let device = parse_device(device)?;
        if width == 0 || height == 0 {
            bail!("taille de sortie V4L2 invalide : {width}x{height}");
        }
        if !is_supported() {
            bail!("la sortie webcam V4L2 n'est disponible que sous Linux");
        }
        // TODO (feature v4l2) : ouvrir le périphérique + VIDIOC_S_FMT (format décodé).
        Ok(Self { device, size: (width, height), stats: Stats::default() })
    }

    pub fn device(&self) -> &Path {
        &self.device
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Écrit une trame décodée dans la webcam virtuelle.
    pub fn push_frame(&mut self, data: &[u8]) -> Result<()> {
        self.stats.frames += 1;
        self.stats.bytes += data.len() as u64;
        // TODO (feature v4l2) : write() de la trame sur le descripteur.
        Ok(())
    }

    /// Ferme le périphérique de rebouclage.
    pub fn finish(self) -> Result<Stats> {
        log::info!(
            "webcam virtuelle fermée : {} ({} trames)",
            self.device.display(),
            self.stats.frames
        );
        Ok(self.stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_valide() {
        assert_eq!(parse_device("/dev/video0").unwrap(), PathBuf::from("/dev/video0"));
        assert_eq!(parse_device("/dev/video10").unwrap(), PathBuf::from("/dev/video10"));
    }

    #[test]
    fn device_invalide() {
        assert!(parse_device("/dev/videoX").is_err());
        assert!(parse_device("/dev/video").is_err());
        assert!(parse_device("video0").is_err());
        assert!(parse_device("/dev/audio0").is_err());
    }

    #[test]
    fn create_valide_le_chemin_avant_la_plateforme() {
        // Un chemin invalide échoue quelle que soit la plateforme.
        assert!(V4l2Sink::create("/dev/videoX", 1280, 720).is_err());
        // Une taille nulle est rejetée avant la vérification de plateforme.
        assert!(V4l2Sink::create("/dev/video0", 0, 720).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn create_et_compteurs_linux() {
        let mut s = V4l2Sink::create("/dev/video2", 1280, 720).unwrap();
        assert_eq!(s.size(), (1280, 720));
        s.push_frame(&[0u8; 1000]).unwrap();
        s.push_frame(&[0u8; 500]).unwrap();
        assert_eq!(s.stats(), Stats { frames: 2, bytes: 1500 });
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn create_refuse_hors_linux() {
        // Chemin et taille valides, mais la plateforme ne gère pas V4L2.
        assert!(V4l2Sink::create("/dev/video2", 1280, 720).is_err());
        assert!(!is_supported());
    }
}
