//! HID/UHID : périphériques clavier/souris physiques **simulés** (#22).
//!
//! Plutôt que d'injecter des événements logiciels, gninou peut créer de vrais
//! périphériques HID côté appareil (via `/dev/uhid`) : l'appareil les voit
//! comme un clavier/une souris USB branchés, ce qui maximise la compatibilité
//! (jeux, champs protégés…).
//!
//! Ce module tient la partie **pure et testable** : descripteurs de rapport
//! (report descriptors) et construction des rapports d'entrée (input reports).
//! La création réelle des périphériques `/dev/uhid` se fait côté serveur.

use crate::control::keyboard::Key;

/// Identifiants logiques des périphériques HID créés par défaut.
pub const KEYBOARD_ID: u16 = 1;
pub const MOUSE_ID: u16 = 2;

/// Descripteur de rapport d'un clavier HID « boot protocol » standard.
/// Rapport d'entrée : `[modificateurs:u8][réservé:u8][6 × keycode:u8]` (8 octets).
pub const KEYBOARD_REPORT_DESC: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x06, // Usage (Keyboard)
    0xA1, 0x01, // Collection (Application)
    0x05, 0x07, //   Usage Page (Key Codes)
    0x19, 0xE0, //   Usage Minimum (224)
    0x29, 0xE7, //   Usage Maximum (231)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x08, //   Report Count (8)
    0x81, 0x02, //   Input (Data, Variable, Absolute) ; octet de modificateurs
    0x95, 0x01, //   Report Count (1)
    0x75, 0x08, //   Report Size (8)
    0x81, 0x01, //   Input (Constant) ; octet réservé
    0x95, 0x06, //   Report Count (6)
    0x75, 0x08, //   Report Size (8)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x65, //   Logical Maximum (101)
    0x05, 0x07, //   Usage Page (Key Codes)
    0x19, 0x00, //   Usage Minimum (0)
    0x29, 0x65, //   Usage Maximum (101)
    0x81, 0x00, //   Input (Data, Array)
    0xC0, // End Collection
];

/// Descripteur de rapport d'une souris relative (3 boutons + X/Y + molette).
/// Rapport d'entrée : `[boutons:u8][dx:i8][dy:i8][molette:i8]` (4 octets).
pub const MOUSE_REPORT_DESC: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x02, // Usage (Mouse)
    0xA1, 0x01, // Collection (Application)
    0x09, 0x01, //   Usage (Pointer)
    0xA1, 0x00, //   Collection (Physical)
    0x05, 0x09, //     Usage Page (Buttons)
    0x19, 0x01, //     Usage Minimum (1)
    0x29, 0x03, //     Usage Maximum (3)
    0x15, 0x00, //     Logical Minimum (0)
    0x25, 0x01, //     Logical Maximum (1)
    0x95, 0x03, //     Report Count (3)
    0x75, 0x01, //     Report Size (1)
    0x81, 0x02, //     Input (Data, Variable, Absolute) ; 3 boutons
    0x95, 0x01, //     Report Count (1)
    0x75, 0x05, //     Report Size (5)
    0x81, 0x01, //     Input (Constant) ; remplissage
    0x05, 0x01, //     Usage Page (Generic Desktop)
    0x09, 0x30, //     Usage (X)
    0x09, 0x31, //     Usage (Y)
    0x09, 0x38, //     Usage (Wheel)
    0x15, 0x81, //     Logical Minimum (-127)
    0x25, 0x7F, //     Logical Maximum (127)
    0x75, 0x08, //     Report Size (8)
    0x95, 0x03, //     Report Count (3)
    0x81, 0x06, //     Input (Data, Variable, Relative) ; X, Y, molette
    0xC0, //   End Collection
    0xC0, // End Collection
];

/// Nombre maximal de touches simultanées dans un rapport clavier boot protocol.
pub const KEYBOARD_MAX_KEYS: usize = 6;

/// Construit un rapport d'entrée clavier (8 octets).
///
/// `modifiers` = masque des modificateurs (Ctrl/Shift/Alt/GUI), `keys` les
/// usages HID des touches enfoncées (au plus [`KEYBOARD_MAX_KEYS`] ; les
/// suivantes sont ignorées). Logique **pure et testable**.
pub fn keyboard_report(modifiers: u8, keys: &[u8]) -> [u8; 8] {
    let mut report = [0u8; 8];
    report[0] = modifiers;
    // report[1] réservé = 0
    for (slot, &usage) in report[2..].iter_mut().zip(keys.iter()) {
        *slot = usage;
    }
    report
}

/// Construit un rapport d'entrée souris relative (4 octets).
/// `buttons` : bits 0=gauche, 1=droit, 2=milieu. `dx`/`dy`/`wheel` : déplacements.
pub fn mouse_report(buttons: u8, dx: i8, dy: i8, wheel: i8) -> [u8; 4] {
    [buttons & 0x07, dx as u8, dy as u8, wheel as u8]
}

/// Usage HID (page clavier) correspondant à une touche logique, si connue.
pub fn hid_usage(key: Key) -> Option<u8> {
    let usage = match key {
        Key::Letter(c) if c.is_ascii_lowercase() => 0x04 + (c as u8 - b'a'),
        Key::Letter(_) => return None,
        // 1..9 = 0x1E..0x26 ; 0 = 0x27
        Key::Digit(0) => 0x27,
        Key::Digit(d) if d <= 9 => 0x1E + (d - 1),
        Key::Digit(_) => return None,
        Key::Enter => 0x28,
        Key::Escape => 0x29,
        Key::Backspace => 0x2A,
        Key::Tab => 0x2B,
        Key::Space => 0x2C,
        Key::Right => 0x4F,
        Key::Left => 0x50,
        Key::Down => 0x51,
        Key::Up => 0x52,
        Key::Home => 0x4A,
        // Les touches de navigation Android (Back, Home) n'ont pas d'équivalent
        // sur un clavier HID physique : on repasse par l'injection logicielle.
        Key::Back => return None,
    };
    Some(usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descripteurs_bien_termines() {
        // Toute collection HID se termine par End Collection (0xC0).
        assert_eq!(*KEYBOARD_REPORT_DESC.last().unwrap(), 0xC0);
        assert_eq!(*MOUSE_REPORT_DESC.last().unwrap(), 0xC0);
        // Les deux débutent par Usage Page (Generic Desktop).
        assert_eq!(&KEYBOARD_REPORT_DESC[0..2], &[0x05, 0x01]);
        assert_eq!(&MOUSE_REPORT_DESC[0..2], &[0x05, 0x01]);
    }

    #[test]
    fn rapport_clavier_une_touche() {
        // 'a' = usage 0x04, sans modificateur.
        let r = keyboard_report(0, &[0x04]);
        assert_eq!(r, [0, 0, 0x04, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn rapport_clavier_modificateur_et_plusieurs_touches() {
        let r = keyboard_report(0x02, &[0x04, 0x05, 0x06]); // Shift + a,b,c
        assert_eq!(r, [0x02, 0, 0x04, 0x05, 0x06, 0, 0, 0]);
    }

    #[test]
    fn rapport_clavier_tronque_a_six_touches() {
        let keys = [0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]; // 7 touches
        let r = keyboard_report(0, &keys);
        // Les 6 premières seulement.
        assert_eq!(&r[2..], &[0x04, 0x05, 0x06, 0x07, 0x08, 0x09]);
    }

    #[test]
    fn rapport_souris() {
        assert_eq!(mouse_report(0b001, 5, -3, 1), [0x01, 5, 0xFD, 0x01]);
        // Les bits au-delà des 3 boutons sont masqués.
        assert_eq!(mouse_report(0xFF, 0, 0, 0)[0], 0x07);
    }

    #[test]
    fn usages_hid() {
        assert_eq!(hid_usage(Key::Letter('a')), Some(0x04));
        assert_eq!(hid_usage(Key::Letter('z')), Some(0x1D));
        assert_eq!(hid_usage(Key::Digit(1)), Some(0x1E));
        assert_eq!(hid_usage(Key::Digit(9)), Some(0x26));
        assert_eq!(hid_usage(Key::Digit(0)), Some(0x27));
        assert_eq!(hid_usage(Key::Enter), Some(0x28));
        assert_eq!(hid_usage(Key::Space), Some(0x2C));
        assert_eq!(hid_usage(Key::Up), Some(0x52));
        // Pas d'équivalent clavier HID pour la touche Back Android.
        assert_eq!(hid_usage(Key::Back), None);
    }
}
