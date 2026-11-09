//! Capture d'écran de l'appareil via `adb exec-out screencap -p` (#38).
//!
//! Voie **pragmatique** pour afficher l'écran réel sans le pipeline vidéo
//! complet (#4–#7) : `adb exec-out screencap -p` renvoie un PNG de l'écran
//! courant ; on le décode en [`Frame`] RGBA pour l'affichage. Le débit est
//! modeste (quelques images/seconde), mais on voit vraiment l'appareil. Le
//! miroir H.264 temps réel reste l'étape suivante.
//!
//! Le **décodage** PNG → [`Frame`] est pur et testé ; la **capture** (`adb`)
//! est l'I/O.

use anyhow::{bail, ensure, Context, Result};
use std::process::Command;

use crate::media::frame::{Frame, BYTES_PER_PIXEL};

/// Exécute `adb exec-out screencap -p` et renvoie les octets PNG bruts.
///
/// `exec-out` évite la traduction de fins de ligne qui corromprait le binaire
/// avec un simple `adb shell`.
pub fn capture_png(serial: &str) -> Result<Vec<u8>> {
    let out = Command::new("adb")
        .args(["-s", serial, "exec-out", "screencap", "-p"])
        .output()
        .context("échec de `adb exec-out screencap` (adb présent ? appareil prêt ?)")?;
    ensure!(
        out.status.success(),
        "`adb exec-out screencap` a échoué (appareil déconnecté ?)"
    );
    ensure!(!out.stdout.is_empty(), "capture d'écran vide");
    Ok(out.stdout)
}

/// Décode un PNG (RGBA ou RGB, 8 bits) en [`Frame`] RGBA. **Pur et testable.**
pub fn decode_png(bytes: &[u8]) -> Result<Frame> {
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder
        .read_info()
        .context("PNG de capture illisible (en-tête)")?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .context("décodage du PNG de capture")?;

    ensure!(
        info.bit_depth == png::BitDepth::Eight,
        "profondeur PNG non gérée : {:?} (8 bits attendus)",
        info.bit_depth
    );

    let data = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => {
            let mut v =
                Vec::with_capacity(info.width as usize * info.height as usize * BYTES_PER_PIXEL);
            for px in data.chunks_exact(3) {
                v.extend_from_slice(px);
                v.push(255);
            }
            v
        }
        other => bail!("format de couleur PNG non géré : {other:?} (RGBA/RGB attendu)"),
    };

    let frame = Frame {
        width: info.width,
        height: info.height,
        rgba,
    };
    ensure!(frame.is_valid(), "image décodée de taille incohérente");
    Ok(frame)
}

/// Capture l'écran de l'appareil et renvoie l'image décodée.
pub fn capture(serial: &str) -> Result<Frame> {
    decode_png(&capture_png(serial)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode une petite image pour alimenter le décodeur.
    fn encode(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut bytes, width, height);
            enc.set_color(color);
            enc.set_depth(png::BitDepth::Eight);
            let mut w = enc.write_header().unwrap();
            w.write_image_data(data).unwrap();
        }
        bytes
    }

    #[test]
    fn decode_rgba_identique() {
        let rgba = vec![10, 20, 30, 255, 40, 50, 60, 128];
        let png = encode(2, 1, png::ColorType::Rgba, &rgba);
        let f = decode_png(&png).unwrap();
        assert_eq!((f.width, f.height), (2, 1));
        assert_eq!(f.rgba, rgba);
        assert!(f.is_valid());
    }

    #[test]
    fn decode_rgb_ajoute_alpha_opaque() {
        let rgb = vec![1, 2, 3, 4, 5, 6];
        let png = encode(2, 1, png::ColorType::Rgb, &rgb);
        let f = decode_png(&png).unwrap();
        assert_eq!(f.rgba, vec![1, 2, 3, 255, 4, 5, 6, 255]);
        assert!(f.is_valid());
    }

    #[test]
    fn decode_png_invalide_echoue() {
        assert!(decode_png(b"ceci n'est pas un png").is_err());
        assert!(decode_png(&[]).is_err());
    }
}
