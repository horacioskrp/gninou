//! Clavier : correspondance touches locales -> keycodes Android.
//!
//! Logique **pure et testable**. La capture réelle des événements clavier
//! (via le backend d'affichage) produira des [`Key`] que l'on convertit en
//! keycodes Android, puis en [`ControlMessage::Key`].

use crate::protocol::ControlMessage;

/// Touche logique, indépendante du backend d'entrée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Lettre `a`..`z` (minuscule ASCII).
    Letter(char),
    /// Chiffre `0`..`9`.
    Digit(u8),
    Enter,
    Backspace,
    Space,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    Back,
}

/// Keycodes Android (`android.view.KeyEvent.KEYCODE_*`) utiles.
mod android {
    pub const A: u32 = 29; // KEYCODE_A ; les lettres suivent A..Z = 29..54
    pub const DIGIT0: u32 = 7; // KEYCODE_0 ; 0..9 = 7..16
    pub const ENTER: u32 = 66;
    pub const DEL: u32 = 67; // retour arrière
    pub const SPACE: u32 = 62;
    pub const TAB: u32 = 61;
    pub const ESCAPE: u32 = 111;
    pub const DPAD_UP: u32 = 19;
    pub const DPAD_DOWN: u32 = 20;
    pub const DPAD_LEFT: u32 = 21;
    pub const DPAD_RIGHT: u32 = 22;
    pub const HOME: u32 = 3;
    pub const BACK: u32 = 4;
}

impl Key {
    /// Convertit la touche en keycode Android, si elle est connue.
    pub fn android_keycode(self) -> Option<u32> {
        let code = match self {
            Key::Letter(c) if c.is_ascii_lowercase() => {
                android::A + (c as u32 - 'a' as u32)
            }
            Key::Letter(_) => return None,
            Key::Digit(d) if d <= 9 => android::DIGIT0 + d as u32,
            Key::Digit(_) => return None,
            Key::Enter => android::ENTER,
            Key::Backspace => android::DEL,
            Key::Space => android::SPACE,
            Key::Tab => android::TAB,
            Key::Escape => android::ESCAPE,
            Key::Up => android::DPAD_UP,
            Key::Down => android::DPAD_DOWN,
            Key::Left => android::DPAD_LEFT,
            Key::Right => android::DPAD_RIGHT,
            Key::Home => android::HOME,
            Key::Back => android::BACK,
        };
        Some(code)
    }
}

/// Construit le message de contrôle pour un appui/relâche de touche.
/// `None` si la touche n'a pas de correspondance Android.
pub fn key_event(key: Key, pressed: bool) -> Option<ControlMessage> {
    let keycode = key.android_keycode()?;
    Some(ControlMessage::Key { keycode, pressed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lettres() {
        assert_eq!(Key::Letter('a').android_keycode(), Some(29));
        assert_eq!(Key::Letter('z').android_keycode(), Some(54));
    }

    #[test]
    fn lettre_non_minuscule_rejetee() {
        assert_eq!(Key::Letter('A').android_keycode(), None);
        assert_eq!(Key::Letter('é').android_keycode(), None);
    }

    #[test]
    fn chiffres() {
        assert_eq!(Key::Digit(0).android_keycode(), Some(7));
        assert_eq!(Key::Digit(9).android_keycode(), Some(16));
        assert_eq!(Key::Digit(10).android_keycode(), None);
    }

    #[test]
    fn touches_speciales() {
        assert_eq!(Key::Enter.android_keycode(), Some(66));
        assert_eq!(Key::Backspace.android_keycode(), Some(67));
        assert_eq!(Key::Space.android_keycode(), Some(62));
        assert_eq!(Key::Back.android_keycode(), Some(4));
    }

    #[test]
    fn construction_message() {
        assert_eq!(
            key_event(Key::Enter, true),
            Some(ControlMessage::Key { keycode: 66, pressed: true })
        );
        assert_eq!(
            key_event(Key::Letter('b'), false),
            Some(ControlMessage::Key { keycode: 30, pressed: false })
        );
    }
}
