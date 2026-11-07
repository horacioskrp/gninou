//! Découpage d'un flux H.264 « Annex B » en unités NAL.
//!
//! Logique **pure et testable**, indépendante de FFmpeg : elle repère les
//! codes de départ (`00 00 01` ou `00 00 00 01`) et sépare les unités NAL,
//! identifie leur type et détecte les images clés (IDR).

/// Types NAL utiles (H.264).
pub mod nal {
    /// Image non-IDR (P/B).
    pub const NON_IDR: u8 = 1;
    /// Image IDR (image clé).
    pub const IDR: u8 = 5;
    /// Sequence Parameter Set.
    pub const SPS: u8 = 7;
    /// Picture Parameter Set.
    pub const PPS: u8 = 8;
}

/// Longueur du code de départ à la position `i`, s'il y en a un.
fn start_code_len(s: &[u8], i: usize) -> Option<usize> {
    if i + 4 <= s.len() && s[i] == 0 && s[i + 1] == 0 && s[i + 2] == 0 && s[i + 3] == 1 {
        Some(4)
    } else if i + 3 <= s.len() && s[i] == 0 && s[i + 1] == 0 && s[i + 2] == 1 {
        Some(3)
    } else {
        None
    }
}

/// Sépare un flux Annex B en unités NAL (sans les codes de départ).
pub fn split_nal_units(stream: &[u8]) -> Vec<&[u8]> {
    let n = stream.len();
    let mut starts: Vec<(usize, usize)> = Vec::new(); // (position, longueur du code)
    let mut i = 0;
    while i < n {
        if let Some(cl) = start_code_len(stream, i) {
            starts.push((i, cl));
            i += cl;
        } else {
            i += 1;
        }
    }

    let mut units = Vec::new();
    for k in 0..starts.len() {
        let (pos, cl) = starts[k];
        let data_start = pos + cl;
        let data_end = if k + 1 < starts.len() {
            starts[k + 1].0
        } else {
            n
        };
        if data_start < data_end {
            units.push(&stream[data_start..data_end]);
        }
    }
    units
}

/// Type NAL (5 bits de poids faible du premier octet).
pub fn nal_type(unit: &[u8]) -> Option<u8> {
    unit.first().map(|b| b & 0x1F)
}

/// Vrai si l'unité est une image clé (IDR).
pub fn is_keyframe(unit: &[u8]) -> bool {
    nal_type(unit) == Some(nal::IDR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flux_vide() {
        assert!(split_nal_units(&[]).is_empty());
    }

    #[test]
    fn code_depart_4_octets() {
        let s = [0, 0, 0, 1, 0x67, 0xAA];
        let units = split_nal_units(&s);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0], &[0x67, 0xAA]);
        assert_eq!(nal_type(units[0]), Some(nal::SPS));
    }

    #[test]
    fn plusieurs_unites_codes_mixtes() {
        // SPS (4 octets) + PPS (3 octets) + IDR (4 octets)
        let s = [
            0, 0, 0, 1, 0x67, // SPS
            0, 0, 1, 0x68, // PPS
            0, 0, 0, 1, 0x65, 0x11, // IDR
        ];
        let units = split_nal_units(&s);
        assert_eq!(units.len(), 3);
        assert_eq!(nal_type(units[0]), Some(nal::SPS));
        assert_eq!(nal_type(units[1]), Some(nal::PPS));
        assert_eq!(nal_type(units[2]), Some(nal::IDR));
        assert!(is_keyframe(units[2]));
        assert!(!is_keyframe(units[0]));
    }

    #[test]
    fn image_non_idr() {
        let s = [0, 0, 1, 0x41, 0x9A];
        let units = split_nal_units(&s);
        assert_eq!(nal_type(units[0]), Some(nal::NON_IDR));
        assert!(!is_keyframe(units[0]));
    }
}
