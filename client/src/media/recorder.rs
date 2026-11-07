//! Enregistrement du flux dans un fichier conteneur (#19).
//!
//! Le **choix du conteneur** (d'après l'extension) et la **vérification de
//! compatibilité** codec ↔ conteneur sont purs et testés. Le multiplexage réel
//! (écriture des paquets dans le fichier) dépendra de FFmpeg (derrière la
//! feature `ffmpeg`, ajout ultérieur) ; ici le `Recorder` tient les compteurs,
//! utile au câblage et aux tests.

use anyhow::{bail, Result};
use std::path::Path;

use crate::media::audio::AudioCodec;
use crate::media::codec::VideoCodec;

/// Conteneur de sortie pour l'enregistrement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// MPEG-4 Part 14 (`.mp4`).
    Mp4,
    /// Matroska (`.mkv`).
    Mkv,
}

impl Container {
    pub const ALL: [Container; 2] = [Container::Mp4, Container::Mkv];

    /// Extension canonique (sans point).
    pub fn extension(self) -> &'static str {
        match self {
            Container::Mp4 => "mp4",
            Container::Mkv => "mkv",
        }
    }

    /// Déduit le conteneur de l'extension d'un chemin de fichier.
    pub fn from_path(path: &Path) -> Result<Container> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        match ext.as_deref() {
            Some("mp4") | Some("m4v") => Ok(Container::Mp4),
            Some("mkv") => Ok(Container::Mkv),
            Some(other) => bail!(
                "extension d'enregistrement non gérée « .{other} » (attendues : {})",
                Self::ALL.map(|c| c.extension()).join(", ")
            ),
            None => bail!("le fichier d'enregistrement doit avoir une extension (.mp4 ou .mkv)"),
        }
    }

    /// Le conteneur accepte-t-il ce codec vidéo ?
    pub fn supports_video(self, codec: VideoCodec) -> bool {
        match self {
            // MP4 et MKV acceptent H.264/H.265/AV1.
            Container::Mp4 | Container::Mkv => {
                matches!(codec, VideoCodec::H264 | VideoCodec::H265 | VideoCodec::Av1)
            }
        }
    }

    /// Le conteneur accepte-t-il ce codec audio ?
    pub fn supports_audio(self, codec: AudioCodec) -> bool {
        match self {
            // MP4 : AAC et Opus ; pas de PCM brut.
            Container::Mp4 => matches!(codec, AudioCodec::Aac | AudioCodec::Opus),
            // MKV accepte tout, y compris le PCM brut.
            Container::Mkv => true,
        }
    }

    /// Vérifie la compatibilité du conteneur avec les codecs choisis.
    /// `audio` vaut `None` si l'audio est désactivé.
    pub fn check_compat(self, video: VideoCodec, audio: Option<AudioCodec>) -> Result<()> {
        if !self.supports_video(video) {
            bail!(
                "le conteneur {} ne gère pas la vidéo {}",
                self.extension(),
                video.as_str()
            );
        }
        if let Some(a) = audio {
            if !self.supports_audio(a) {
                bail!(
                    "le conteneur {} ne gère pas l'audio {} (essayez .mkv, ou --no-audio)",
                    self.extension(),
                    a.as_str()
                );
            }
        }
        Ok(())
    }
}

/// Compteurs d'un enregistrement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub video_packets: u64,
    pub video_bytes: u64,
    pub audio_packets: u64,
    pub audio_bytes: u64,
}

/// Enregistreur : reçoit les paquets encodés et (à terme) les multiplexe
/// dans le fichier de sortie.
pub struct Recorder {
    container: Container,
    path: std::path::PathBuf,
    stats: Stats,
}

impl Recorder {
    /// Prépare un enregistrement vers `path`, après validation du conteneur et
    /// de la compatibilité des codecs.
    pub fn create(
        path: impl AsRef<Path>,
        video: VideoCodec,
        audio: Option<AudioCodec>,
    ) -> Result<Self> {
        let path = path.as_ref();
        let container = Container::from_path(path)?;
        container.check_compat(video, audio)?;
        // TODO (feature ffmpeg) : ouvrir le muxer de sortie (avformat).
        Ok(Self {
            container,
            path: path.to_path_buf(),
            stats: Stats::default(),
        })
    }

    pub fn container(&self) -> Container {
        self.container
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Ajoute un paquet vidéo encodé.
    pub fn push_video(&mut self, data: &[u8]) -> Result<()> {
        self.stats.video_packets += 1;
        self.stats.video_bytes += data.len() as u64;
        // TODO (feature ffmpeg) : écrire le paquet dans le flux vidéo.
        Ok(())
    }

    /// Ajoute un paquet audio encodé.
    pub fn push_audio(&mut self, data: &[u8]) -> Result<()> {
        self.stats.audio_packets += 1;
        self.stats.audio_bytes += data.len() as u64;
        // TODO (feature ffmpeg) : écrire le paquet dans le flux audio.
        Ok(())
    }

    /// Finalise le fichier (écriture de l'index/trailer).
    pub fn finish(self) -> Result<Stats> {
        // TODO (feature ffmpeg) : av_write_trailer + fermeture.
        log::info!(
            "enregistrement terminé : {} ({} paquets vidéo, {} paquets audio)",
            self.path.display(),
            self.stats.video_packets,
            self.stats.audio_packets
        );
        Ok(self.stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn conteneur_par_extension() {
        assert_eq!(
            Container::from_path(Path::new("out.mp4")).unwrap(),
            Container::Mp4
        );
        assert_eq!(
            Container::from_path(Path::new("OUT.MKV")).unwrap(),
            Container::Mkv
        );
        assert_eq!(
            Container::from_path(Path::new("clip.m4v")).unwrap(),
            Container::Mp4
        );
    }

    #[test]
    fn extension_inconnue_ou_absente() {
        assert!(Container::from_path(Path::new("out.avi")).is_err());
        assert!(Container::from_path(Path::new("sansext")).is_err());
    }

    #[test]
    fn mp4_refuse_pcm_mkv_accepte() {
        assert!(!Container::Mp4.supports_audio(AudioCodec::Raw));
        assert!(Container::Mp4.supports_audio(AudioCodec::Aac));
        assert!(Container::Mp4.supports_audio(AudioCodec::Opus));
        assert!(Container::Mkv.supports_audio(AudioCodec::Raw));
    }

    #[test]
    fn video_acceptee_partout() {
        for c in Container::ALL {
            for v in VideoCodec::ALL {
                assert!(c.supports_video(v));
            }
        }
    }

    #[test]
    fn compat_echoue_sur_pcm_en_mp4() {
        assert!(Container::Mp4
            .check_compat(VideoCodec::H264, Some(AudioCodec::Raw))
            .is_err());
        assert!(Container::Mp4
            .check_compat(VideoCodec::H264, Some(AudioCodec::Aac))
            .is_ok());
        assert!(Container::Mp4.check_compat(VideoCodec::H264, None).is_ok());
    }

    #[test]
    fn recorder_create_et_compteurs() {
        let mut r = Recorder::create("sortie.mkv", VideoCodec::Av1, Some(AudioCodec::Raw)).unwrap();
        assert_eq!(r.container(), Container::Mkv);
        r.push_video(&[0u8; 500]).unwrap();
        r.push_video(&[0u8; 300]).unwrap();
        r.push_audio(&[0u8; 80]).unwrap();
        assert_eq!(
            r.stats(),
            Stats {
                video_packets: 2,
                video_bytes: 800,
                audio_packets: 1,
                audio_bytes: 80
            }
        );
    }

    #[test]
    fn recorder_create_rejette_incompatible() {
        assert!(Recorder::create("x.mp4", VideoCodec::H264, Some(AudioCodec::Raw)).is_err());
        assert!(Recorder::create("x.webm", VideoCodec::H264, None).is_err());
    }
}
