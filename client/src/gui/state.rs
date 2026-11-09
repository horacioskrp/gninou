//! View-model **pur** de l'interface graphique (#32).
//!
//! Tout l'état de la fenêtre et ses transitions vivent ici, sans aucune
//! dépendance à egui : c'est la partie **testable** de la GUI. Le rendu
//! (`super::app`, derrière la feature `gui`) ne fait que lire cet état et
//! renvoyer des [`Msg`] en retour.

use crate::device::listing::Entry;
use crate::gui::pairing::PairingState;
use crate::gui::wifi::WifiState;

/// Écran actuellement affiché.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Liste unifiée des appareils (USB + Wi-Fi).
    Devices,
    /// Assistant d'appairage Wi-Fi (#33).
    Pairing,
    /// Connexion Wi-Fi : bascule USB→sans-fil, mDNS, reconnexion auto (#34).
    Wifi,
}

/// État du chargement de la liste d'appareils.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Load {
    /// Rien n'a encore été chargé.
    Idle,
    /// Recherche en cours.
    Loading,
    /// Liste chargée.
    Loaded,
    /// Échec, avec le message d'erreur.
    Failed(String),
}

/// Messages émis par l'UI (actions utilisateur) ou par l'I/O (résultats).
#[derive(Debug, Clone)]
pub enum Msg {
    /// L'utilisateur demande un rafraîchissement de la liste.
    RefreshRequested,
    /// La liste d'appareils vient d'être chargée.
    DevicesLoaded(Vec<Entry>),
    /// Le chargement a échoué.
    LoadFailed(String),
    /// L'utilisateur sélectionne l'appareil d'indice donné.
    Selected(usize),
    /// L'utilisateur demande la connexion à l'appareil sélectionné.
    ConnectRequested,
    /// Résultat de la connexion : `Ok(adresse)` ou `Err(message)`.
    ConnectResult(Result<String, String>),
    /// Navigue vers l'écran demandé.
    GoTo(Screen),
}

/// État global du view-model GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub screen: Screen,
    pub devices: Vec<Entry>,
    pub load: Load,
    /// Indice de l'appareil sélectionné, s'il y en a un.
    pub selected: Option<usize>,
    /// Message affiché en bas de la fenêtre.
    pub status: String,
    /// État de l'écran d'appairage Wi-Fi (#33).
    pub pairing: PairingState,
    /// État de l'écran Connexion Wi-Fi (#34).
    pub wifi: WifiState,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            screen: Screen::Devices,
            devices: Vec::new(),
            load: Load::Idle,
            selected: None,
            status: "Prêt — cliquez sur « Rafraîchir » pour détecter les appareils.".to_string(),
            pairing: PairingState::new(),
            wifi: WifiState::new(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// L'appareil actuellement sélectionné, s'il existe encore.
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.selected.and_then(|i| self.devices.get(i))
    }

    /// Un chargement est-il en cours ?
    pub fn is_loading(&self) -> bool {
        self.load == Load::Loading
    }

    /// Applique un message et fait évoluer l'état. **Pur et testable.**
    pub fn reduce(&mut self, msg: Msg) {
        match msg {
            Msg::RefreshRequested => {
                self.load = Load::Loading;
                self.status = "Recherche d'appareils…".to_string();
            }
            Msg::DevicesLoaded(list) => {
                self.devices = list;
                self.load = Load::Loaded;
                // Invalide une sélection devenue hors limites.
                if let Some(i) = self.selected {
                    if i >= self.devices.len() {
                        self.selected = None;
                    }
                }
                self.status = match self.devices.len() {
                    0 => "Aucun appareil détecté.".to_string(),
                    1 => "1 appareil détecté.".to_string(),
                    n => format!("{n} appareils détectés."),
                };
            }
            Msg::LoadFailed(e) => {
                self.status = format!("Échec de la détection : {e}");
                self.load = Load::Failed(e);
            }
            Msg::Selected(i) => {
                if i < self.devices.len() {
                    self.selected = Some(i);
                }
            }
            Msg::ConnectRequested => {
                self.status = match self.selected_entry() {
                    Some(e) => format!("Connexion à {}…", e.id),
                    None => "Sélectionnez d'abord un appareil.".to_string(),
                };
            }
            Msg::ConnectResult(Ok(addr)) => {
                self.status = format!("Connecté : {addr}");
            }
            Msg::ConnectResult(Err(e)) => {
                self.status = format!("Échec de connexion : {e}");
            }
            Msg::GoTo(screen) => {
                self.status = match screen {
                    Screen::Devices => "Écran : Appareils.".to_string(),
                    Screen::Pairing => "Écran : Appairage Wi-Fi.".to_string(),
                    Screen::Wifi => "Écran : Connexion Wi-Fi.".to_string(),
                };
                self.screen = screen;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::listing::{Entry, Kind};

    fn entry(id: &str) -> Entry {
        Entry {
            kind: Kind::Wireless,
            id: id.into(),
            status: "prêt".into(),
        }
    }

    #[test]
    fn etat_initial() {
        let s = AppState::new();
        assert_eq!(s.screen, Screen::Devices);
        assert_eq!(s.load, Load::Idle);
        assert!(s.selected.is_none());
        assert!(s.devices.is_empty());
    }

    #[test]
    fn rafraichir_passe_en_chargement() {
        let mut s = AppState::new();
        s.reduce(Msg::RefreshRequested);
        assert!(s.is_loading());
    }

    #[test]
    fn chargement_remplit_et_compte() {
        let mut s = AppState::new();
        s.reduce(Msg::DevicesLoaded(vec![entry("A"), entry("B")]));
        assert_eq!(s.load, Load::Loaded);
        assert_eq!(s.devices.len(), 2);
        assert!(s.status.contains('2'));
    }

    #[test]
    fn selection_hors_limites_invalidee_au_rechargement() {
        let mut s = AppState::new();
        s.reduce(Msg::DevicesLoaded(vec![entry("A"), entry("B")]));
        s.reduce(Msg::Selected(1));
        assert_eq!(s.selected, Some(1));
        // Nouvelle liste plus courte : la sélection doit disparaître.
        s.reduce(Msg::DevicesLoaded(vec![entry("A")]));
        assert_eq!(s.selected, None);
    }

    #[test]
    fn selection_valide_et_invalide() {
        let mut s = AppState::new();
        s.reduce(Msg::DevicesLoaded(vec![entry("A")]));
        s.reduce(Msg::Selected(5)); // hors limites : ignorée
        assert_eq!(s.selected, None);
        s.reduce(Msg::Selected(0));
        assert_eq!(s.selected_entry().unwrap().id, "A");
    }

    #[test]
    fn connexion_sans_selection_invite() {
        let mut s = AppState::new();
        s.reduce(Msg::ConnectRequested);
        assert!(s.status.to_lowercase().contains("sélectionnez"));
    }

    #[test]
    fn connexion_avec_selection_annonce_la_cible() {
        let mut s = AppState::new();
        s.reduce(Msg::DevicesLoaded(vec![entry("10.0.0.9:5555")]));
        s.reduce(Msg::Selected(0));
        s.reduce(Msg::ConnectRequested);
        assert!(s.status.contains("10.0.0.9:5555"));
    }

    #[test]
    fn resultat_connexion() {
        let mut s = AppState::new();
        s.reduce(Msg::ConnectResult(Ok("10.0.0.9:5555".into())));
        assert!(s.status.contains("Connecté"));
        s.reduce(Msg::ConnectResult(Err("injoignable".into())));
        assert!(s.status.contains("Échec"));
    }

    #[test]
    fn navigation_entre_ecrans() {
        let mut s = AppState::new();
        assert_eq!(s.screen, Screen::Devices);
        s.reduce(Msg::GoTo(Screen::Pairing));
        assert_eq!(s.screen, Screen::Pairing);
        s.reduce(Msg::GoTo(Screen::Wifi));
        assert_eq!(s.screen, Screen::Wifi);
        s.reduce(Msg::GoTo(Screen::Devices));
        assert_eq!(s.screen, Screen::Devices);
    }

    #[test]
    fn echec_chargement() {
        let mut s = AppState::new();
        s.reduce(Msg::LoadFailed("adb absent".into()));
        assert_eq!(s.load, Load::Failed("adb absent".into()));
        assert!(s.status.contains("adb absent"));
    }
}
