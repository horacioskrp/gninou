//! Protocole gninou : messages échangés sur le tunnel adb.
//!
//! Trois flux logiques transitent par le tunnel :
//!   - vidéo   : trames encodées (H.264/H.265) device -> client.
//!   - audio   : trames encodées (Opus/AAC) device -> client.
//!   - contrôle: événements client -> device (clavier, souris, presse-papiers).
//!
//! Le format exact reste à figer ; les types ci-dessous sont des esquisses.

/// En-tête commun préfixant chaque trame multimédia.
#[derive(Debug, Clone)]
pub struct MediaHeader {
    /// Horodatage de présentation (µs).
    pub pts_us: u64,
    /// Taille utile en octets.
    pub len: u32,
    /// Trame clé (keyframe) pour la vidéo.
    pub key_frame: bool,
}

/// Événement de contrôle envoyé au device.
#[derive(Debug, Clone)]
pub enum ControlMessage {
    /// Touche clavier (keycode, appui/relâche).
    Key { keycode: u32, pressed: bool },
    /// Mouvement/clic souris en coordonnées écran.
    Pointer { x: i32, y: i32, buttons: u32 },
    /// Molette.
    Scroll { x: i32, y: i32, h: i32, v: i32 },
    /// Pousser du texte dans le presse-papiers du device.
    SetClipboard { text: String },
}
