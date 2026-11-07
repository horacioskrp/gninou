//! Géométrie de capture : rognage (crop) et rotation.
//!
//! Parsing et validation **purs et testables** des options `--crop` et
//! `--rotation`. L'application réelle se fait côté serveur (projection
//! `SurfaceControl`).

use anyhow::{bail, Result};

/// Rectangle de rognage, en pixels, dans le repère de l'écran capturé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crop {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
}

impl Crop {
    /// Analyse une chaîne `W:H:X:Y` (ex. `1080:2400:0:0`).
    pub fn parse(s: &str) -> Result<Crop> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            bail!("format de crop attendu « W:H:X:Y », reçu « {s} »");
        }
        let mut n = [0u32; 4];
        for (i, p) in parts.iter().enumerate() {
            n[i] = p
                .parse()
                .map_err(|_| anyhow::anyhow!("valeur de crop invalide : « {p} »"))?;
        }
        let crop = Crop { width: n[0], height: n[1], x: n[2], y: n[3] };
        if crop.width == 0 || crop.height == 0 {
            bail!("le crop doit avoir une largeur et une hauteur non nulles");
        }
        Ok(crop)
    }

    /// Vérifie que le rectangle tient dans une définition `fw x fh`.
    pub fn fits(&self, fw: u32, fh: u32) -> bool {
        self.x + self.width <= fw && self.y + self.height <= fh
    }

    /// Représentation `W:H:X:Y` (pour transmission au serveur).
    pub fn to_arg(&self) -> String {
        format!("{}:{}:{}:{}", self.width, self.height, self.x, self.y)
    }
}

/// Spécification d'un nouvel affichage virtuel (#21, Android 10+).
///
/// Taille et densité sont optionnelles : absentes, le serveur retient les
/// valeurs par défaut de l'appareil. Parsing **pur et testable**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NewDisplay {
    /// `(largeur, hauteur)` en pixels, ou `None` (défaut appareil).
    pub size: Option<(u32, u32)>,
    /// Densité en dpi, ou `None` (défaut appareil).
    pub dpi: Option<u32>,
}

impl NewDisplay {
    /// Analyse une spécification `LxH`, `LxH/dpi`, ou vide (défauts).
    /// Ex. : `1920x1080`, `1920x1080/420`, `` (vide).
    pub fn parse(s: &str) -> Result<NewDisplay> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(NewDisplay::default());
        }
        let (size_part, dpi) = match s.split_once('/') {
            Some((sz, d)) => {
                let dpi: u32 = d
                    .trim()
                    .parse()
                    .map_err(|_| anyhow::anyhow!("dpi invalide : « {d} »"))?;
                if dpi == 0 {
                    bail!("le dpi doit être strictement positif");
                }
                (sz, Some(dpi))
            }
            None => (s, None),
        };
        let size_lc = size_part.to_ascii_lowercase();
        let (w, h) = size_lc
            .split_once('x')
            .ok_or_else(|| anyhow::anyhow!("taille attendue « LxH », reçu « {size_part} »"))?;
        let w: u32 = w
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("largeur invalide : « {w} »"))?;
        let h: u32 = h
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("hauteur invalide : « {h} »"))?;
        if w == 0 || h == 0 {
            bail!("l'affichage virtuel doit avoir des dimensions non nulles");
        }
        Ok(NewDisplay { size: Some((w, h)), dpi })
    }

    /// Représentation transmise au serveur (`LxH`, `LxH/dpi`, `/dpi` ou vide).
    pub fn to_arg(&self) -> String {
        match (self.size, self.dpi) {
            (Some((w, h)), Some(d)) => format!("{w}x{h}/{d}"),
            (Some((w, h)), None) => format!("{w}x{h}"),
            (None, Some(d)) => format!("/{d}"),
            (None, None) => String::new(),
        }
    }
}

/// Analyse une rotation en degrés ; seules 0/90/180/270 sont acceptées.
pub fn parse_rotation(s: &str) -> Result<u16> {
    match s.trim() {
        "0" => Ok(0),
        "90" => Ok(90),
        "180" => Ok(180),
        "270" => Ok(270),
        other => bail!("rotation invalide « {other} » (attendu : 0, 90, 180, 270)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_valide() {
        let c = Crop::parse("1080:2400:0:0").unwrap();
        assert_eq!(c, Crop { width: 1080, height: 2400, x: 0, y: 0 });
    }

    #[test]
    fn crop_mauvais_nombre_de_champs() {
        assert!(Crop::parse("1080:2400:0").is_err());
        assert!(Crop::parse("").is_err());
    }

    #[test]
    fn crop_dimensions_nulles_rejetees() {
        assert!(Crop::parse("0:100:0:0").is_err());
    }

    #[test]
    fn crop_fits() {
        let c = Crop::parse("500:500:100:100").unwrap();
        assert!(c.fits(1000, 1000));
        assert!(!c.fits(550, 1000)); // 100+500 > 550
        assert!(!c.fits(1000, 550));
    }

    #[test]
    fn crop_aller_retour_arg() {
        let c = Crop::parse("640:480:10:20").unwrap();
        assert_eq!(c.to_arg(), "640:480:10:20");
    }

    #[test]
    fn rotation_valide_et_invalide() {
        assert_eq!(parse_rotation("90").unwrap(), 90);
        assert_eq!(parse_rotation("0").unwrap(), 0);
        assert!(parse_rotation("45").is_err());
        assert!(parse_rotation("360").is_err());
    }

    #[test]
    fn new_display_vide_defauts() {
        let d = NewDisplay::parse("").unwrap();
        assert_eq!(d, NewDisplay { size: None, dpi: None });
        assert_eq!(d.to_arg(), "");
    }

    #[test]
    fn new_display_taille_seule() {
        let d = NewDisplay::parse("1920x1080").unwrap();
        assert_eq!(d, NewDisplay { size: Some((1920, 1080)), dpi: None });
        assert_eq!(d.to_arg(), "1920x1080");
        // Casse du « x » insensible.
        assert_eq!(NewDisplay::parse("800X600").unwrap().size, Some((800, 600)));
    }

    #[test]
    fn new_display_taille_et_dpi() {
        let d = NewDisplay::parse("1080x2400/420").unwrap();
        assert_eq!(d, NewDisplay { size: Some((1080, 2400)), dpi: Some(420) });
        assert_eq!(d.to_arg(), "1080x2400/420");
    }

    #[test]
    fn new_display_invalide() {
        assert!(NewDisplay::parse("1920").is_err()); // pas de 'x'
        assert!(NewDisplay::parse("0x1080").is_err()); // dimension nulle
        assert!(NewDisplay::parse("1920xABC").is_err()); // hauteur non num
        assert!(NewDisplay::parse("1920x1080/0").is_err()); // dpi nul
        assert!(NewDisplay::parse("1920x1080/xx").is_err()); // dpi non num
    }
}
