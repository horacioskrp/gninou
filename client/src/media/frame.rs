//! Image décodée en mémoire (#36).
//!
//! [`Frame`] est la représentation RGBA8 d'une image : c'est ce que le décodeur
//! vidéo (#6) produira à terme, et ce que le rendu (fenêtre GUI, capture PNG,
//! enregistrement) consomme. En attendant un flux réel venu d'un appareil, la
//! [`test_pattern`] fournit une **mire animée** pour câbler et valider le rendu.
//! Tout ici est **pur et testable**, sans dépendance graphique.

/// Nombre d'octets par pixel (RGBA8).
pub const BYTES_PER_PIXEL: usize = 4;

/// Image décodée, pixels en RGBA8 ligne par ligne (haut vers le bas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` octets (R, G, B, A).
    pub rgba: Vec<u8>,
}

impl Frame {
    /// Image noire opaque de la taille donnée.
    pub fn black(width: u32, height: u32) -> Frame {
        let len = width as usize * height as usize * BYTES_PER_PIXEL;
        let mut rgba = vec![0u8; len];
        // Alpha opaque.
        for a in rgba.iter_mut().skip(3).step_by(BYTES_PER_PIXEL) {
            *a = 255;
        }
        Frame {
            width,
            height,
            rgba,
        }
    }

    /// Nombre de pixels de l'image.
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Le tampon a-t-il exactement la taille attendue ?
    pub fn is_valid(&self) -> bool {
        self.rgba.len() == self.pixel_count() * BYTES_PER_PIXEL
    }
}

/// Mire de test animée : un dégradé diagonal stable plus une barre verticale
/// claire qui défile avec `tick`. Sert de substitut visible tant que le
/// décodage vidéo réel (#6) n'alimente pas encore le rendu. **Pur.**
///
/// `width`/`height` sont ramenés à au moins 1 pour toujours produire une image
/// valide.
pub fn test_pattern(width: u32, height: u32, tick: u64) -> Frame {
    let w = width.max(1);
    let h = height.max(1);
    let mut frame = Frame::black(w, h);
    let bar = (tick % w as u64) as u32; // colonne de la barre mobile
    for y in 0..h {
        for x in 0..w {
            let idx = (y as usize * w as usize + x as usize) * BYTES_PER_PIXEL;
            // Dégradé diagonal.
            let r = (x * 255 / w) as u8;
            let g = (y * 255 / h) as u8;
            let b = 128u8;
            // Barre mobile (3 px de large) : éclaircit la zone.
            let on_bar = x >= bar && x < bar + 3;
            frame.rgba[idx] = if on_bar { 255 } else { r };
            frame.rgba[idx + 1] = if on_bar { 255 } else { g };
            frame.rgba[idx + 2] = if on_bar { 255 } else { b };
            frame.rgba[idx + 3] = 255;
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noir_opaque_valide() {
        let f = Frame::black(4, 3);
        assert_eq!(f.width, 4);
        assert_eq!(f.height, 3);
        assert_eq!(f.rgba.len(), 4 * 3 * 4);
        assert!(f.is_valid());
        // Premier pixel : noir, alpha opaque.
        assert_eq!(&f.rgba[0..4], &[0, 0, 0, 255]);
    }

    #[test]
    fn pixel_count_ok() {
        let f = Frame::black(16, 9);
        assert_eq!(f.pixel_count(), 144);
        assert!(f.is_valid());
    }

    #[test]
    fn mire_dimensions_et_validite() {
        let f = test_pattern(320, 180, 0);
        assert_eq!(f.width, 320);
        assert_eq!(f.height, 180);
        assert!(f.is_valid());
    }

    #[test]
    fn mire_dimensions_minimales() {
        // Dimensions nulles ramenées à 1x1 : image valide, pas de panique.
        let f = test_pattern(0, 0, 5);
        assert_eq!((f.width, f.height), (1, 1));
        assert!(f.is_valid());
    }

    #[test]
    fn mire_barre_se_deplace() {
        let w = 100;
        // La barre est en colonne (tick % w) : deux ticks distincts diffèrent.
        let a = test_pattern(w, 10, 0);
        let b = test_pattern(w, 10, 50);
        assert_ne!(a.rgba, b.rgba);
    }

    #[test]
    fn mire_barre_a_la_bonne_colonne() {
        let f = test_pattern(100, 1, 10);
        // Colonne 10 doit être blanche (barre), colonne 50 non.
        let col = |x: usize| {
            let i = x * BYTES_PER_PIXEL;
            &f.rgba[i..i + 3]
        };
        assert_eq!(col(10), &[255, 255, 255]);
        assert_ne!(col(50), &[255, 255, 255]);
    }
}
