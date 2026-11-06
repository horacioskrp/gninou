//! Décodage vidéo.
//!
//! Le flux H.264/H.265 reçu du tunnel est découpé en unités NAL
//! ([`super::h264`]) puis décodé. Le décodage réel s'appuie sur FFmpeg et
//! n'est compilé qu'avec la feature `ffmpeg` (sinon le build reste léger,
//! sans dépendance native). Sans la feature, le `Decoder` tient les
//! statistiques du flux — utile au câblage et aux tests.

use anyhow::Result;

use super::h264;

/// Statistiques d'un flux décodé.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub nal_units: u64,
    pub keyframes: u64,
}

/// Décodeur vidéo.
pub struct Decoder {
    codec: String,
    stats: Stats,
    #[cfg(feature = "ffmpeg")]
    backend: ffmpeg_backend::FfmpegDecoder,
}

impl Decoder {
    pub fn new(codec: &str) -> Result<Self> {
        Ok(Self {
            codec: codec.to_string(),
            stats: Stats::default(),
            #[cfg(feature = "ffmpeg")]
            backend: ffmpeg_backend::FfmpegDecoder::new(codec)?,
        })
    }

    pub fn codec(&self) -> &str {
        &self.codec
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Alimente le décodeur avec un paquet du flux (une ou plusieurs unités NAL).
    pub fn push_packet(&mut self, data: &[u8]) -> Result<()> {
        for unit in h264::split_nal_units(data) {
            self.stats.nal_units += 1;
            if h264::is_keyframe(unit) {
                self.stats.keyframes += 1;
            }
        }

        #[cfg(feature = "ffmpeg")]
        self.backend.decode(data)?;

        Ok(())
    }
}

#[cfg(feature = "ffmpeg")]
mod ffmpeg_backend {
    use anyhow::Result;

    /// Décodeur matériel/logiciel basé sur FFmpeg (`ffmpeg-next`).
    pub struct FfmpegDecoder {
        // TODO: ffmpeg_next::decoder::Video + contexte, échange de paquets/frames.
    }

    impl FfmpegDecoder {
        pub fn new(_codec: &str) -> Result<Self> {
            // TODO: ffmpeg_next::init(), ouvrir le décodeur correspondant au codec.
            Ok(Self {})
        }

        pub fn decode(&mut self, _data: &[u8]) -> Result<()> {
            // TODO: envoyer le paquet, récupérer les frames, les transmettre au rendu.
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compte_unites_et_keyframes() {
        let mut dec = Decoder::new("h264").unwrap();
        // SPS + PPS + IDR
        let flux = [
            0, 0, 0, 1, 0x67, //
            0, 0, 1, 0x68, //
            0, 0, 0, 1, 0x65, 0x11,
        ];
        dec.push_packet(&flux).unwrap();
        assert_eq!(dec.stats().nal_units, 3);
        assert_eq!(dec.stats().keyframes, 1);
    }

    #[test]
    fn cumul_sur_plusieurs_paquets() {
        let mut dec = Decoder::new("h264").unwrap();
        dec.push_packet(&[0, 0, 1, 0x65]).unwrap(); // IDR
        dec.push_packet(&[0, 0, 1, 0x41]).unwrap(); // non-IDR
        assert_eq!(dec.stats().nal_units, 2);
        assert_eq!(dec.stats().keyframes, 1);
    }
}
