//! Contrôle : capture des entrées locales -> messages pour le device.
//!
//! Traduit les événements clavier/souris de la fenêtre en
//! [`ControlMessage`] envoyés au serveur via le canal contrôle. La capture
//! réelle des événements dépend du backend d'affichage (feature `display`) ;
//! la **traduction des coordonnées** et le **mapping des boutons** sont purs
//! et testés ici.

use crate::media::display::Rect;
use crate::protocol::ControlMessage;

/// Boutons souris (masque de bits), aligné sur les conventions usuelles.
pub mod button {
    pub const PRIMARY: u32 = 1 << 0; // gauche
    pub const SECONDARY: u32 = 1 << 1; // droit
    pub const MIDDLE: u32 = 1 << 2;
}

/// Convertit une position fenêtre `(win_x, win_y)` en coordonnées **appareil**.
///
/// `rect` est la zone où la vidéo est dessinée (cf. `display::fit_rect`),
/// `frame_w/h` la définition de la vidéo. Renvoie `None` si le point est
/// hors de la zone vidéo (bandes du letterbox). Logique pure et testable.
pub fn map_to_device(
    win_x: i32,
    win_y: i32,
    rect: Rect,
    frame_w: u32,
    frame_h: u32,
) -> Option<(i32, i32)> {
    let (rx, ry, rw, rh) = rect;
    if rw == 0 || rh == 0 || frame_w == 0 || frame_h == 0 {
        return None;
    }
    // Position relative à la zone vidéo.
    let dx = win_x - rx;
    let dy = win_y - ry;
    if dx < 0 || dy < 0 || dx >= rw as i32 || dy >= rh as i32 {
        return None; // hors image (bande noire)
    }
    // Mise à l'échelle vers la définition de l'appareil.
    let x = (dx as i64 * frame_w as i64 / rw as i64) as i32;
    let y = (dy as i64 * frame_h as i64 / rh as i64) as i32;
    Some((x, y))
}

/// Construit un message `Pointer` pour un clic/déplacement souris, en projetant
/// la position fenêtre vers l'appareil. `None` si hors zone vidéo.
pub fn pointer(
    win_x: i32,
    win_y: i32,
    buttons: u32,
    rect: Rect,
    frame_w: u32,
    frame_h: u32,
) -> Option<ControlMessage> {
    let (x, y) = map_to_device(win_x, win_y, rect, frame_w, frame_h)?;
    Some(ControlMessage::Pointer { x, y, buttons })
}

/// Construit un message `Scroll` à la position donnée.
pub fn scroll(
    win_x: i32,
    win_y: i32,
    h: i32,
    v: i32,
    rect: Rect,
    frame_w: u32,
    frame_h: u32,
) -> Option<ControlMessage> {
    let (x, y) = map_to_device(win_x, win_y, rect, frame_w, frame_h)?;
    Some(ControlMessage::Scroll { x, y, h, v })
}

/// Collecteur d'événements d'entrée (capture réelle via le backend d'affichage).
pub struct Controller {
    _private: (),
}

impl Controller {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Zone vidéo de 1000x1000 à l'origine, image 1000x1000 -> mapping 1:1.
    const RECT: Rect = (0, 0, 1000, 1000);

    #[test]
    fn mapping_identite() {
        assert_eq!(map_to_device(500, 250, RECT, 1000, 1000), Some((500, 250)));
    }

    #[test]
    fn mise_a_l_echelle() {
        // Zone 500x500, image 1000x1000 -> facteur x2.
        let rect = (0, 0, 500, 500);
        assert_eq!(map_to_device(100, 100, rect, 1000, 1000), Some((200, 200)));
    }

    #[test]
    fn offset_letterbox() {
        // Zone décalée de (50,0) : un clic à 50 retombe sur x=0 appareil.
        let rect = (50, 0, 1000, 1000);
        assert_eq!(map_to_device(50, 0, rect, 1000, 1000), Some((0, 0)));
    }

    #[test]
    fn hors_zone_renvoie_none() {
        let rect = (50, 0, 100, 100);
        assert_eq!(map_to_device(10, 10, rect, 1000, 1000), None); // avant la zone
        assert_eq!(map_to_device(200, 10, rect, 1000, 1000), None); // après la zone
    }

    #[test]
    fn pointer_produit_coordonnees_appareil() {
        let msg = pointer(250, 250, button::PRIMARY, (0, 0, 500, 500), 1000, 1000);
        assert_eq!(
            msg,
            Some(ControlMessage::Pointer { x: 500, y: 500, buttons: button::PRIMARY })
        );
    }

    #[test]
    fn scroll_hors_zone_none() {
        assert!(scroll(9999, 9999, 0, -1, RECT, 1000, 1000).is_none());
    }
}
