//! Manette (gamepad) simulée via HID/UHID (#23).
//!
//! Réutilise le transport UHID de [`crate::control::hid`] : la manette est un
//! périphérique HID de plus (descripteur + rapports d'entrée). Ce module tient
//! la partie **pure et testable** : descripteur de rapport et construction du
//! rapport manette (16 boutons, croix directionnelle, 2 sticks, 2 gâchettes).

/// Identifiant logique du périphérique manette (après [`hid::KEYBOARD_ID`] et
/// [`hid::MOUSE_ID`]).
///
/// [`hid::KEYBOARD_ID`]: crate::control::hid::KEYBOARD_ID
/// [`hid::MOUSE_ID`]: crate::control::hid::MOUSE_ID
pub const GAMEPAD_ID: u16 = 3;

/// Bits des boutons dans le rapport (octets 0-1, little-endian).
pub mod button {
    pub const SOUTH: u16 = 1 << 0; // A / Croix
    pub const EAST: u16 = 1 << 1; // B / Rond
    pub const WEST: u16 = 1 << 2; // X / Carré
    pub const NORTH: u16 = 1 << 3; // Y / Triangle
    pub const L1: u16 = 1 << 4;
    pub const R1: u16 = 1 << 5;
    pub const L2: u16 = 1 << 6;
    pub const R2: u16 = 1 << 7;
    pub const SELECT: u16 = 1 << 8;
    pub const START: u16 = 1 << 9;
    pub const L3: u16 = 1 << 10; // clic stick gauche
    pub const R3: u16 = 1 << 11; // clic stick droit
    pub const MODE: u16 = 1 << 12; // bouton central (guide)
}

/// Direction de la croix directionnelle (hat switch). `NEUTRAL` = relâchée.
pub mod hat {
    pub const NORTH: u8 = 0;
    pub const NORTH_EAST: u8 = 1;
    pub const EAST: u8 = 2;
    pub const SOUTH_EAST: u8 = 3;
    pub const SOUTH: u8 = 4;
    pub const SOUTH_WEST: u8 = 5;
    pub const WEST: u8 = 6;
    pub const NORTH_WEST: u8 = 7;
    /// État nul (aucune direction), codé sur 4 bits.
    pub const NEUTRAL: u8 = 0x0F;
}

/// Valeur centrale (repos) d'un axe analogique 8 bits.
pub const AXIS_CENTER: u8 = 128;

/// Descripteur de rapport d'une manette standard.
///
/// Rapport d'entrée (9 octets) :
/// `[boutons:u16 LE][hat:4b + pad:4b][X][Y][Z][Rz][Rx(LT)][Ry(RT)]`.
pub const GAMEPAD_REPORT_DESC: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Game Pad)
    0xA1, 0x01, // Collection (Application)
    // 16 boutons
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (1)
    0x29, 0x10, //   Usage Maximum (16)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x10, //   Report Count (16)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    // Croix directionnelle (hat switch)
    0x05, 0x01, //   Usage Page (Generic Desktop)
    0x09, 0x39, //   Usage (Hat switch)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x07, //   Logical Maximum (7)
    0x35, 0x00, //   Physical Minimum (0)
    0x46, 0x3B, 0x01, //   Physical Maximum (315)
    0x65, 0x14, //   Unit (degrés)
    0x75, 0x04, //   Report Size (4)
    0x95, 0x01, //   Report Count (1)
    0x81, 0x42, //   Input (Data, Variable, Absolute, Null State)
    // Remplissage 4 bits
    0x75, 0x04, //   Report Size (4)
    0x95, 0x01, //   Report Count (1)
    0x81, 0x01, //   Input (Constant)
    // Sticks : X, Y (gauche), Z, Rz (droit)
    0x05, 0x01, //   Usage Page (Generic Desktop)
    0x09, 0x30, //   Usage (X)
    0x09, 0x31, //   Usage (Y)
    0x09, 0x32, //   Usage (Z)
    0x09, 0x35, //   Usage (Rz)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x04, //   Report Count (4)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    // Gâchettes : Rx (LT), Ry (RT)
    0x09, 0x33, //   Usage (Rx)
    0x09, 0x34, //   Usage (Ry)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    0xC0, // End Collection
];

/// Taille du rapport d'entrée manette, en octets.
pub const GAMEPAD_REPORT_LEN: usize = 9;

/// État complet d'une manette, converti en rapport d'entrée HID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GamepadState {
    /// Masque des boutons (voir [`button`]).
    pub buttons: u16,
    /// Direction de la croix (voir [`hat`]).
    pub hat: u8,
    /// Stick gauche X/Y (0..=255, centre [`AXIS_CENTER`]).
    pub left: (u8, u8),
    /// Stick droit X/Y.
    pub right: (u8, u8),
    /// Gâchettes gauche/droite (0..=255).
    pub triggers: (u8, u8),
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: 0,
            hat: hat::NEUTRAL,
            left: (AXIS_CENTER, AXIS_CENTER),
            right: (AXIS_CENTER, AXIS_CENTER),
            triggers: (0, 0),
        }
    }
}

impl GamepadState {
    /// Sérialise l'état en rapport d'entrée HID (9 octets). Logique pure.
    pub fn report(&self) -> [u8; GAMEPAD_REPORT_LEN] {
        [
            (self.buttons & 0xFF) as u8,
            (self.buttons >> 8) as u8,
            self.hat & 0x0F,
            self.left.0,
            self.left.1,
            self.right.0,
            self.right.1,
            self.triggers.0,
            self.triggers.1,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descripteur_bien_forme() {
        assert_eq!(&GAMEPAD_REPORT_DESC[0..4], &[0x05, 0x01, 0x09, 0x05]);
        assert_eq!(*GAMEPAD_REPORT_DESC.last().unwrap(), 0xC0);
    }

    #[test]
    fn etat_repos() {
        let r = GamepadState::default().report();
        assert_eq!(
            r,
            [
                0,
                0,
                hat::NEUTRAL,
                AXIS_CENTER,
                AXIS_CENTER,
                AXIS_CENTER,
                AXIS_CENTER,
                0,
                0
            ]
        );
        assert_eq!(r.len(), GAMEPAD_REPORT_LEN);
    }

    #[test]
    fn boutons_little_endian() {
        // SOUTH (bit 0) + START (bit 9) -> 0x0201 -> [0x01, 0x02]
        let st = GamepadState {
            buttons: button::SOUTH | button::START,
            ..Default::default()
        };
        let r = st.report();
        assert_eq!(r[0], 0x01);
        assert_eq!(r[1], 0x02);
    }

    #[test]
    fn hat_et_axes() {
        let st = GamepadState {
            hat: hat::EAST,
            left: (0, 255),
            right: (10, 20),
            triggers: (255, 64),
            ..Default::default()
        };
        let r = st.report();
        assert_eq!(r[2], hat::EAST);
        assert_eq!(&r[3..7], &[0, 255, 10, 20]);
        assert_eq!(&r[7..9], &[255, 64]);
    }

    #[test]
    fn hat_masque_sur_4_bits() {
        // Une valeur hat > 0x0F est tronquée aux 4 bits bas.
        let st = GamepadState {
            hat: 0xF5,
            ..Default::default()
        };
        assert_eq!(st.report()[2], 0x05);
    }
}
