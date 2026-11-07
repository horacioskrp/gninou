//! Source vidéo caméra (#25, Android 12+).
//!
//! Permet de refléter le flux d'une **caméra** de l'appareil au lieu de son
//! écran. La capture réelle (Camera2/`SurfaceControl`) se fait côté serveur ;
//! ce module tient la partie **pure et testable** : choix de la source, de
//! l'orientation (front/back), et de la taille de capture.

use anyhow::{bail, Result};

/// Source du flux vidéo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoSource {
    /// Écran de l'appareil (comportement par défaut).
    #[default]
    Display,
    /// Une caméra de l'appareil.
    Camera,
}

impl VideoSource {
    pub fn as_str(self) -> &'static str {
        match self {
            VideoSource::Display => "display",
            VideoSource::Camera => "camera",
        }
    }

    pub fn parse(id: &str) -> Result<VideoSource> {
        match id.trim().to_ascii_lowercase().as_str() {
            "display" | "screen" => Ok(VideoSource::Display),
            "camera" | "cam" => Ok(VideoSource::Camera),
            other => bail!("source vidéo inconnue « {other} » (attendues : display, camera)"),
        }
    }
}

/// Orientation de la caméra souhaitée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    /// Caméra arrière (dos de l'appareil).
    Back,
    /// Caméra frontale (selfie).
    Front,
    /// Caméra externe (USB).
    External,
}

impl Facing {
    pub fn as_str(self) -> &'static str {
        match self {
            Facing::Back => "back",
            Facing::Front => "front",
            Facing::External => "external",
        }
    }

    pub fn parse(id: &str) -> Result<Facing> {
        match id.trim().to_ascii_lowercase().as_str() {
            "back" | "rear" => Ok(Facing::Back),
            "front" => Ok(Facing::Front),
            "external" | "usb" => Ok(Facing::External),
            other => {
                bail!("orientation caméra inconnue « {other} » (attendues : back, front, external)")
            }
        }
    }
}

/// Analyse une taille `LxH` (ex. `1920x1080`). Logique pure.
pub fn parse_size(s: &str) -> Result<(u32, u32)> {
    let s = s.trim().to_ascii_lowercase();
    let (w, h) = s
        .split_once('x')
        .ok_or_else(|| anyhow::anyhow!("taille attendue « LxH », reçu « {s} »"))?;
    let w: u32 = w
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("largeur invalide : « {w} »"))?;
    let h: u32 = h
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("hauteur invalide : « {h} »"))?;
    if w == 0 || h == 0 {
        bail!("la taille de capture doit être non nulle");
    }
    Ok((w, h))
}

/// Configuration de capture caméra, validée.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CameraConfig {
    /// Identifiant explicite de caméra (prioritaire sur `facing` si présent).
    pub id: Option<String>,
    /// Orientation souhaitée si pas d'identifiant explicite.
    pub facing: Option<Facing>,
    /// Taille de capture `(l, h)`, ou défauts de la caméra.
    pub size: Option<(u32, u32)>,
    /// Images par seconde, ou défaut.
    pub fps: Option<u32>,
}

impl CameraConfig {
    /// Construit une configuration à partir des options brutes (déjà en
    /// « source caméra »). Valide l'orientation et la taille.
    pub fn build(
        id: Option<String>,
        facing: Option<&str>,
        size: Option<&str>,
        fps: Option<u32>,
    ) -> Result<CameraConfig> {
        let facing = match facing {
            Some(f) => Some(Facing::parse(f)?),
            None => None,
        };
        let size = match size {
            Some(s) => Some(parse_size(s)?),
            None => None,
        };
        if let Some(0) = fps {
            bail!("le fps caméra doit être strictement positif");
        }
        Ok(CameraConfig {
            id,
            facing,
            size,
            fps,
        })
    }

    /// Arguments `clé=valeur` à transmettre au serveur pour cette caméra.
    pub fn server_args(&self) -> Vec<String> {
        let mut v = vec![format!("video_source={}", VideoSource::Camera.as_str())];
        if let Some(id) = &self.id {
            v.push(format!("camera_id={id}"));
        }
        if let Some(f) = self.facing {
            v.push(format!("camera_facing={}", f.as_str()));
        }
        if let Some((w, h)) = self.size {
            v.push(format!("camera_size={w}x{h}"));
        }
        if let Some(fps) = self.fps {
            v.push(format!("camera_fps={fps}"));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_parse() {
        assert_eq!(VideoSource::parse("display").unwrap(), VideoSource::Display);
        assert_eq!(VideoSource::parse("CAMERA").unwrap(), VideoSource::Camera);
        assert_eq!(VideoSource::default(), VideoSource::Display);
        assert!(VideoSource::parse("webcam").is_err());
    }

    #[test]
    fn facing_parse() {
        assert_eq!(Facing::parse("front").unwrap(), Facing::Front);
        assert_eq!(Facing::parse("rear").unwrap(), Facing::Back);
        assert_eq!(Facing::parse("USB").unwrap(), Facing::External);
        assert!(Facing::parse("left").is_err());
    }

    #[test]
    fn taille_parse() {
        assert_eq!(parse_size("1920x1080").unwrap(), (1920, 1080));
        assert_eq!(parse_size("640X480").unwrap(), (640, 480));
        assert!(parse_size("1920").is_err());
        assert!(parse_size("0x480").is_err());
    }

    #[test]
    fn config_valide_et_args() {
        let c = CameraConfig::build(None, Some("front"), Some("1280x720"), Some(30)).unwrap();
        assert_eq!(c.facing, Some(Facing::Front));
        assert_eq!(c.size, Some((1280, 720)));
        assert_eq!(
            c.server_args(),
            vec![
                "video_source=camera",
                "camera_facing=front",
                "camera_size=1280x720",
                "camera_fps=30",
            ]
        );
    }

    #[test]
    fn config_par_id() {
        let c = CameraConfig::build(Some("2".into()), None, None, None).unwrap();
        assert_eq!(c.server_args(), vec!["video_source=camera", "camera_id=2"]);
    }

    #[test]
    fn config_rejets() {
        assert!(CameraConfig::build(None, Some("left"), None, None).is_err());
        assert!(CameraConfig::build(None, None, Some("abc"), None).is_err());
        assert!(CameraConfig::build(None, None, None, Some(0)).is_err());
    }
}
