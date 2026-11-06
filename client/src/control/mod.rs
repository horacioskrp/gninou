//! Contrôle : capture des entrées locales -> messages pour le device.
//!
//! Traduit les événements clavier/souris de la fenêtre en
//! `protocol::ControlMessage` envoyés au serveur via le tunnel.

use anyhow::Result;

use crate::protocol::ControlMessage;

/// Collecteur d'événements d'entrée.
pub struct Controller {
    _private: (),
}

impl Controller {
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Transforme un événement local en message de contrôle (si pertinent).
    pub fn map_event(&self /*, event: WindowEvent */) -> Result<Option<ControlMessage>> {
        // TODO: mapper touches/souris/molette depuis l'événement de la fenêtre
        Ok(None)
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
