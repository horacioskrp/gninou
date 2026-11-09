//! View-model **pur** du contrôle tactile du miroir (#39).
//!
//! Permet de **naviguer** sur l'appareil depuis la fenêtre de miroir : un clic
//! devient un *tap*, un glisser devient un *swipe*, et des boutons envoient les
//! touches de navigation (Retour/Accueil/Récents). Ce module ne fait **aucune
//! I/O** : il convertit une position de pointeur dans l'image affichée en
//! coordonnées **pixel de l'appareil**, classe un geste (tap vs swipe) et
//! construit les arguments `adb shell input …`. L'injection réelle vit dans
//! [`crate::device::input`]. Pur et testé dans le build par défaut.

/// Déplacement minimal (en pixels **appareil**) au-delà duquel un geste est
/// traité comme un *swipe* plutôt qu'un *tap*.
pub const SWIPE_MIN_PX: i32 = 16;

/// Durée par défaut d'un *swipe*, en millisecondes.
pub const SWIPE_MS: u32 = 120;

/// Touche de navigation Android.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavKey {
    Back,
    Home,
    Recents,
}

impl NavKey {
    /// Code de touche `adb` correspondant.
    pub fn keycode(self) -> &'static str {
        match self {
            NavKey::Back => "KEYCODE_BACK",
            NavKey::Home => "KEYCODE_HOME",
            NavKey::Recents => "KEYCODE_APP_SWITCH",
        }
    }

    /// Libellé court pour l'UI.
    pub fn label(self) -> &'static str {
        match self {
            NavKey::Back => "◀ Retour",
            NavKey::Home => "⬤ Accueil",
            NavKey::Recents => "▢ Récents",
        }
    }
}

/// Action d'entrée à injecter sur l'appareil, en coordonnées **appareil**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    Tap {
        x: i32,
        y: i32,
    },
    Swipe {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        ms: u32,
    },
    Key(NavKey),
}

/// Convertit une position de pointeur **relative au coin haut-gauche de l'image
/// affichée** en coordonnées pixel de l'appareil.
///
/// `disp_w`/`disp_h` sont la taille de l'image à l'écran ; `dev_w`/`dev_h` la
/// résolution réelle de l'appareil. Le résultat est borné à l'écran de
/// l'appareil. Renvoie `None` si l'image ou l'appareil est dégénéré. **Pur.**
pub fn map_to_device(
    rel_x: f32,
    rel_y: f32,
    disp_w: f32,
    disp_h: f32,
    dev_w: u32,
    dev_h: u32,
) -> Option<(i32, i32)> {
    if disp_w <= 0.0 || disp_h <= 0.0 || dev_w == 0 || dev_h == 0 {
        return None;
    }
    let fx = (rel_x / disp_w).clamp(0.0, 1.0);
    let fy = (rel_y / disp_h).clamp(0.0, 1.0);
    let x = (fx * (dev_w - 1) as f32).round() as i32;
    let y = (fy * (dev_h - 1) as f32).round() as i32;
    Some((x, y))
}

/// Un déplacement (en pixels appareil) dépasse-t-il le seuil du *swipe* ?
/// **Pur.**
pub fn is_swipe(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    (x2 - x1).abs() >= SWIPE_MIN_PX || (y2 - y1).abs() >= SWIPE_MIN_PX
}

/// Construit une action à partir d'un geste (début/fin en coordonnées
/// appareil) : *tap* si court, *swipe* sinon. **Pur.**
pub fn gesture_action(start: (i32, i32), end: (i32, i32)) -> InputAction {
    if is_swipe(start.0, start.1, end.0, end.1) {
        InputAction::Swipe {
            x1: start.0,
            y1: start.1,
            x2: end.0,
            y2: end.1,
            ms: SWIPE_MS,
        }
    } else {
        InputAction::Tap { x: end.0, y: end.1 }
    }
}

/// Arguments `adb` (sans `-s <serial>`) pour injecter l'action. **Pur et
/// testable.**
pub fn adb_input_args(action: &InputAction) -> Vec<String> {
    match action {
        InputAction::Tap { x, y } => vec![
            "shell".into(),
            "input".into(),
            "tap".into(),
            x.to_string(),
            y.to_string(),
        ],
        InputAction::Swipe { x1, y1, x2, y2, ms } => vec![
            "shell".into(),
            "input".into(),
            "swipe".into(),
            x1.to_string(),
            y1.to_string(),
            x2.to_string(),
            y2.to_string(),
            ms.to_string(),
        ],
        InputAction::Key(k) => vec![
            "shell".into(),
            "input".into(),
            "keyevent".into(),
            k.keycode().into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mappe_les_coins() {
        // Image 300x600 à l'écran, appareil 1080x2280.
        assert_eq!(
            map_to_device(0.0, 0.0, 300.0, 600.0, 1080, 2280),
            Some((0, 0))
        );
        assert_eq!(
            map_to_device(300.0, 600.0, 300.0, 600.0, 1080, 2280),
            Some((1079, 2279))
        );
        // Centre.
        assert_eq!(
            map_to_device(150.0, 300.0, 300.0, 600.0, 1080, 2280),
            Some((540, 1140))
        );
    }

    #[test]
    fn borne_hors_image() {
        // Pointeur au-delà des bords : ramené dans l'écran de l'appareil.
        assert_eq!(
            map_to_device(-10.0, -10.0, 300.0, 600.0, 1080, 2280),
            Some((0, 0))
        );
        assert_eq!(
            map_to_device(999.0, 999.0, 300.0, 600.0, 1080, 2280),
            Some((1079, 2279))
        );
    }

    #[test]
    fn image_ou_appareil_degenere() {
        assert_eq!(map_to_device(1.0, 1.0, 0.0, 600.0, 1080, 2280), None);
        assert_eq!(map_to_device(1.0, 1.0, 300.0, 600.0, 0, 2280), None);
    }

    #[test]
    fn tap_vs_swipe() {
        assert!(!is_swipe(100, 100, 105, 108)); // petit déplacement -> tap
        assert!(is_swipe(100, 100, 100, 400)); // vertical -> swipe
        assert!(is_swipe(100, 100, 400, 100)); // horizontal -> swipe
    }

    #[test]
    fn geste_produit_tap_ou_swipe() {
        assert_eq!(
            gesture_action((500, 1000), (503, 1004)),
            InputAction::Tap { x: 503, y: 1004 }
        );
        assert_eq!(
            gesture_action((500, 1600), (500, 400)),
            InputAction::Swipe {
                x1: 500,
                y1: 1600,
                x2: 500,
                y2: 400,
                ms: SWIPE_MS
            }
        );
    }

    #[test]
    fn args_adb_tap_swipe_key() {
        assert_eq!(
            adb_input_args(&InputAction::Tap { x: 540, y: 1140 }),
            vec!["shell", "input", "tap", "540", "1140"]
        );
        assert_eq!(
            adb_input_args(&InputAction::Swipe {
                x1: 500,
                y1: 1600,
                x2: 500,
                y2: 400,
                ms: 120
            }),
            vec!["shell", "input", "swipe", "500", "1600", "500", "400", "120"]
        );
        assert_eq!(
            adb_input_args(&InputAction::Key(NavKey::Back)),
            vec!["shell", "input", "keyevent", "KEYCODE_BACK"]
        );
    }

    #[test]
    fn keycodes_navigation() {
        assert_eq!(NavKey::Back.keycode(), "KEYCODE_BACK");
        assert_eq!(NavKey::Home.keycode(), "KEYCODE_HOME");
        assert_eq!(NavKey::Recents.keycode(), "KEYCODE_APP_SWITCH");
    }
}
