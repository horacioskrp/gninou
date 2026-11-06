//! Protocole gninou : messages échangés sur le tunnel adb.
//!
//! Trois flux logiques transitent par le tunnel :
//!   - vidéo   : trames encodées (H.264/H.265) device -> client.
//!   - audio   : trames encodées (Opus/AAC) device -> client.
//!   - contrôle: événements client -> device (clavier, souris, presse-papiers).
//!
//! Le format exact reste à figer ; les types ci-dessous sont des esquisses.

use anyhow::{bail, Result};

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

/// Octet de type (tag) ouvrant chaque message de contrôle.
pub mod control_type {
    pub const KEY: u8 = 0;
    pub const POINTER: u8 = 1;
    pub const SCROLL: u8 = 2;
    pub const SET_CLIPBOARD: u8 = 3;
}

/// Événement de contrôle envoyé au device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlMessage {
    /// Touche clavier (keycode, appui/relâche).
    Key { keycode: u32, pressed: bool },
    /// Mouvement/clic souris en coordonnées appareil.
    Pointer { x: i32, y: i32, buttons: u32 },
    /// Molette (h/v = défilement horizontal/vertical).
    Scroll { x: i32, y: i32, h: i32, v: i32 },
    /// Pousser du texte dans le presse-papiers du device.
    SetClipboard { text: String },
}

impl ControlMessage {
    /// Sérialise le message au format binaire du canal contrôle (big-endian).
    ///
    /// Disposition : `[tag:u8][champs…]`. Logique pure et testable.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            ControlMessage::Key { keycode, pressed } => {
                out.push(control_type::KEY);
                out.extend_from_slice(&keycode.to_be_bytes());
                out.push(if *pressed { 1 } else { 0 });
            }
            ControlMessage::Pointer { x, y, buttons } => {
                out.push(control_type::POINTER);
                out.extend_from_slice(&x.to_be_bytes());
                out.extend_from_slice(&y.to_be_bytes());
                out.extend_from_slice(&buttons.to_be_bytes());
            }
            ControlMessage::Scroll { x, y, h, v } => {
                out.push(control_type::SCROLL);
                out.extend_from_slice(&x.to_be_bytes());
                out.extend_from_slice(&y.to_be_bytes());
                out.extend_from_slice(&h.to_be_bytes());
                out.extend_from_slice(&v.to_be_bytes());
            }
            ControlMessage::SetClipboard { text } => {
                out.push(control_type::SET_CLIPBOARD);
                let bytes = text.as_bytes();
                out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                out.extend_from_slice(bytes);
            }
        }
        out
    }
}

/// Octet de type (tag) ouvrant chaque message **appareil → client**.
pub mod device_type {
    pub const CLIPBOARD: u8 = 0;
}

/// Message envoyé par l'appareil vers le client (canal appareil→PC).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceMessage {
    /// Nouveau contenu du presse-papiers de l'appareil (à recopier sur le PC).
    Clipboard { text: String },
}

impl DeviceMessage {
    /// Décode un message appareil→client (miroir de l'encodage serveur).
    pub fn decode(data: &[u8]) -> Result<DeviceMessage> {
        let tag = *data.first().ok_or_else(|| anyhow::anyhow!("message vide"))?;
        match tag {
            device_type::CLIPBOARD => {
                if data.len() < 5 {
                    bail!("message clipboard tronqué");
                }
                let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
                let end = 5 + len;
                if data.len() < end {
                    bail!("longueur de clipboard incohérente");
                }
                let text = String::from_utf8(data[5..end].to_vec())
                    .map_err(|_| anyhow::anyhow!("clipboard non UTF-8"))?;
                Ok(DeviceMessage::Clipboard { text })
            }
            other => bail!("type de message appareil inconnu : {other}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_clipboard() {
        let bytes = [device_type::CLIPBOARD, 0, 0, 0, 2, b'h', b'i'];
        assert_eq!(
            DeviceMessage::decode(&bytes).unwrap(),
            DeviceMessage::Clipboard { text: "hi".into() }
        );
    }

    #[test]
    fn decode_clipboard_tronque_erreur() {
        assert!(DeviceMessage::decode(&[device_type::CLIPBOARD, 0, 0, 0, 5, b'x']).is_err());
    }

    #[test]
    fn decode_type_inconnu_erreur() {
        assert!(DeviceMessage::decode(&[99, 0]).is_err());
    }

    #[test]
    fn encode_pointer() {
        let msg = ControlMessage::Pointer { x: 1, y: 2, buttons: 1 };
        assert_eq!(
            msg.encode(),
            vec![
                control_type::POINTER,
                0, 0, 0, 1, // x
                0, 0, 0, 2, // y
                0, 0, 0, 1, // buttons
            ]
        );
    }

    #[test]
    fn encode_key() {
        let msg = ControlMessage::Key { keycode: 0x41, pressed: true };
        assert_eq!(msg.encode(), vec![control_type::KEY, 0, 0, 0, 0x41, 1]);
    }

    #[test]
    fn encode_scroll_longueur() {
        let msg = ControlMessage::Scroll { x: 0, y: 0, h: 0, v: -1 };
        // tag(1) + 4 i32 (16) = 17 octets
        assert_eq!(msg.encode().len(), 17);
        assert_eq!(msg.encode()[0], control_type::SCROLL);
    }

    #[test]
    fn encode_clipboard() {
        let msg = ControlMessage::SetClipboard { text: "hi".into() };
        assert_eq!(
            msg.encode(),
            vec![control_type::SET_CLIPBOARD, 0, 0, 0, 2, b'h', b'i']
        );
    }
}
