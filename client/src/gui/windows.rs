//! View-model **pur** des fenêtres de miroir dédiées (#37).
//!
//! Depuis la liste des appareils, on ouvre le miroir (et, à terme, le contrôle)
//! d'un appareil dans **sa propre fenêtre**, à la manière d'Android Studio.
//! Plusieurs appareils peuvent donc être affichés simultanément, chacun avec
//! son propre état d'affichage ([`MirrorState`]). Ce module gère l'ensemble des
//! fenêtres ouvertes (ouverture dédupliquée, fermeture, accès) ; le rendu egui
//! et l'ouverture réelle des fenêtres (viewports) vivent dans `super::app`.
//! Tout ici est pur et testé dans le build par défaut.

use super::mirror::MirrorState;

/// Une fenêtre de miroir ouverte pour un appareil donné.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirrorWindow {
    /// Identifiant de l'appareil (série USB ou `ip:port`).
    pub device_id: String,
    /// État d'affichage propre à cette fenêtre (zoom, enregistrement…).
    pub mirror: MirrorState,
}

impl MirrorWindow {
    fn new(device_id: String) -> Self {
        MirrorWindow {
            device_id,
            mirror: MirrorState::new(),
        }
    }
}

/// Ensemble des fenêtres de miroir ouvertes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Windows {
    pub open: Vec<MirrorWindow>,
}

impl Windows {
    pub fn new() -> Self {
        Self::default()
    }

    /// Une fenêtre est-elle déjà ouverte pour cet appareil ?
    pub fn is_open(&self, device_id: &str) -> bool {
        self.open.iter().any(|w| w.device_id == device_id)
    }

    /// Ouvre une fenêtre pour l'appareil (sans doublon). Renvoie `true` si une
    /// **nouvelle** fenêtre a été créée, `false` si elle existait déjà.
    pub fn open(&mut self, device_id: &str) -> bool {
        if self.is_open(device_id) {
            return false;
        }
        self.open.push(MirrorWindow::new(device_id.to_string()));
        true
    }

    /// Ferme la fenêtre de cet appareil (sans effet si elle n'existe pas).
    pub fn close(&mut self, device_id: &str) {
        self.open.retain(|w| w.device_id != device_id);
    }

    /// Accès à la fenêtre d'un appareil, si elle est ouverte.
    pub fn get(&self, device_id: &str) -> Option<&MirrorWindow> {
        self.open.iter().find(|w| w.device_id == device_id)
    }

    /// Accès mutable à la fenêtre d'un appareil, si elle est ouverte.
    pub fn get_mut(&mut self, device_id: &str) -> Option<&mut MirrorWindow> {
        self.open.iter_mut().find(|w| w.device_id == device_id)
    }

    /// Identifiants des appareils ayant une fenêtre ouverte (ordre d'ouverture).
    pub fn ids(&self) -> Vec<String> {
        self.open.iter().map(|w| w.device_id.clone()).collect()
    }

    /// Nombre de fenêtres ouvertes.
    pub fn count(&self) -> usize {
        self.open.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ouvre_sans_doublon() {
        let mut w = Windows::new();
        assert!(w.open("ABC"));
        assert!(!w.open("ABC")); // déjà ouverte
        assert!(w.open("10.0.0.9:5555"));
        assert_eq!(w.count(), 2);
        assert!(w.is_open("ABC"));
    }

    #[test]
    fn ferme() {
        let mut w = Windows::new();
        w.open("ABC");
        w.open("XYZ");
        w.close("ABC");
        assert!(!w.is_open("ABC"));
        assert!(w.is_open("XYZ"));
        assert_eq!(w.count(), 1);
        // Fermer un inconnu ne fait rien.
        w.close("ZZZ");
        assert_eq!(w.count(), 1);
    }

    #[test]
    fn acces_mutable_et_etat_propre() {
        let mut w = Windows::new();
        w.open("ABC");
        w.open("XYZ");
        // Chaque fenêtre a son propre état d'affichage.
        w.get_mut("ABC").unwrap().mirror.running = false;
        assert!(!w.get_mut("ABC").unwrap().mirror.running);
        assert!(w.get_mut("XYZ").unwrap().mirror.running);
        assert!(w.get_mut("absent").is_none());
    }

    #[test]
    fn ids_dans_l_ordre() {
        let mut w = Windows::new();
        w.open("A");
        w.open("B");
        assert_eq!(w.ids(), vec!["A".to_string(), "B".to_string()]);
    }
}
