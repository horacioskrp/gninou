//! View-model **pur** de l'écran « Connexion Wi-Fi » (#34).
//!
//! Rassemble dans un seul écran les commodités Wi-Fi déjà en place : la bascule
//! USB → sans-fil (#27), la découverte mDNS d'un appareil annonçant sa connexion
//! (#28) et la reconnexion automatique à délai exponentiel (#31). Comme les
//! autres view-models GUI, tout l'état et ses transitions vivent ici, sans
//! dépendance à egui : c'est la partie **testable** de l'écran. Le rendu
//! (`super::app`, derrière la feature `gui`) lit cet état, déclenche les I/O
//! (`adb tcpip`/`adb connect`, mDNS) et renvoie des [`WifiMsg`].

use crate::device::connection::{self, DEFAULT_ADB_PORT};
use crate::device::reconnect::Backoff;

/// Nombre de tentatives par défaut si le champ est vide ou illisible.
const DEFAULT_ATTEMPTS: u32 = 10;

/// Phase de l'écran Connexion Wi-Fi.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Phase {
    /// Au repos, prêt à agir.
    #[default]
    Idle,
    /// Bascule USB → Wi-Fi en cours.
    Switching,
    /// Découverte mDNS en cours.
    Discovering,
    /// Connecté à l'adresse réseau indiquée.
    Connected(String),
    /// Échec, avec le message d'erreur.
    Failed(String),
}

/// Messages émis par l'UI (actions/éditions) ou par l'I/O (résultats).
#[derive(Debug, Clone)]
pub enum WifiMsg {
    /// Édition du numéro de série de l'appareil USB.
    SerialChanged(String),
    /// Édition du port TCP/IP cible.
    PortChanged(String),
    /// Activation/désactivation de la reconnexion automatique.
    ReconnectToggled(bool),
    /// Édition du nombre de tentatives de reconnexion.
    AttemptsChanged(String),
    /// L'utilisateur demande la bascule USB → Wi-Fi.
    SwitchRequested,
    /// Résultat de la bascule : `Ok(adresse)` ou `Err(message)`.
    SwitchResult(Result<String, String>),
    /// L'utilisateur demande une découverte mDNS.
    DiscoverRequested,
    /// Résultat de la découverte : `Some(adresse)` annoncée, ou `None`.
    Discovered(Option<String>),
    /// L'utilisateur demande la connexion à l'adresse découverte.
    ConnectDiscoveredRequested,
    /// Résultat de la connexion : `Ok(adresse)` ou `Err(message)`.
    ConnectResult(Result<String, String>),
    /// Réinitialise l'écran.
    Reset,
}

/// État de l'écran Connexion Wi-Fi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiState {
    /// Numéro de série de l'appareil USB à basculer en Wi-Fi.
    pub serial: String,
    /// Port TCP/IP cible (texte éditable ; défaut [`DEFAULT_ADB_PORT`]).
    pub port: String,
    /// Adresse de connexion découverte en mDNS, s'il y en a une.
    pub discovered: Option<String>,
    /// Reconnexion automatique activée ?
    pub reconnect: bool,
    /// Nombre de tentatives de reconnexion (texte éditable).
    pub attempts: String,
    pub phase: Phase,
    /// Message d'aide/retour affiché sous le formulaire.
    pub hint: String,
}

impl Default for WifiState {
    fn default() -> Self {
        WifiState {
            serial: String::new(),
            port: DEFAULT_ADB_PORT.to_string(),
            discovered: None,
            reconnect: false,
            attempts: DEFAULT_ATTEMPTS.to_string(),
            phase: Phase::Idle,
            hint: String::new(),
        }
    }
}

impl WifiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Port TCP/IP saisi, s'il est valide (1–65535). Logique **pure**.
    pub fn port_num(&self) -> Option<u16> {
        match self.port.trim().parse::<u16>() {
            Ok(p) if p != 0 => Some(p),
            _ => None,
        }
    }

    /// Nombre de tentatives saisi, ramené à 1 au minimum ; retombe sur
    /// [`DEFAULT_ATTEMPTS`] si le champ est vide ou illisible. **Pur**.
    pub fn attempts_num(&self) -> u32 {
        match self.attempts.trim().parse::<u32>() {
            Ok(0) => 1,
            Ok(n) => n,
            Err(_) => DEFAULT_ATTEMPTS,
        }
    }

    /// Nombre de tentatives réellement appliqué : le nombre saisi si la
    /// reconnexion auto est active, sinon un seul essai. **Pur**.
    pub fn effective_attempts(&self) -> u32 {
        if self.reconnect {
            self.attempts_num()
        } else {
            1
        }
    }

    /// Politique de reprise à appliquer aux I/O réseau de cet écran. **Pure**.
    pub fn backoff(&self) -> Backoff {
        let d = Backoff::default();
        Backoff::new(d.base_ms, d.max_ms, self.effective_attempts())
    }

    /// Une opération d'I/O est-elle en cours ?
    pub fn is_busy(&self) -> bool {
        matches!(self.phase, Phase::Switching | Phase::Discovering)
    }

    /// La bascule USB → Wi-Fi est-elle possible (série + port valides) ?
    pub fn can_switch(&self) -> bool {
        !self.serial.trim().is_empty() && self.port_num().is_some() && !self.is_busy()
    }

    /// Peut-on se connecter à l'adresse découverte en mDNS ?
    pub fn can_connect_discovered(&self) -> bool {
        self.discovered.is_some() && !self.is_busy()
    }

    /// Applique un message et fait évoluer l'état. **Pur et testable.**
    pub fn reduce(&mut self, msg: WifiMsg) {
        match msg {
            WifiMsg::SerialChanged(s) => self.serial = s,
            WifiMsg::PortChanged(s) => self.port = s,
            WifiMsg::ReconnectToggled(b) => self.reconnect = b,
            WifiMsg::AttemptsChanged(s) => self.attempts = s,
            WifiMsg::SwitchRequested => {
                if self.can_switch() {
                    self.phase = Phase::Switching;
                    self.hint = "Bascule en Wi-Fi… (lecture de l'IP, adb tcpip, connexion)".into();
                } else if self.serial.trim().is_empty() {
                    self.phase = Phase::Failed("série manquante".into());
                    self.hint = "Indiquez le numéro de série de l'appareil USB.".into();
                } else {
                    self.phase = Phase::Failed("port invalide".into());
                    self.hint = "Port invalide : attendez un entier entre 1 et 65535.".into();
                }
            }
            WifiMsg::SwitchResult(Ok(addr)) => {
                self.discovered = Some(addr.clone());
                self.phase = Phase::Connected(addr.clone());
                self.hint = format!("Basculé en Wi-Fi et connecté : {addr}");
            }
            WifiMsg::SwitchResult(Err(e)) => {
                self.phase = Phase::Failed(e.clone());
                self.hint = format!("Échec de la bascule Wi-Fi : {e}");
            }
            WifiMsg::DiscoverRequested => {
                self.phase = Phase::Discovering;
                self.hint = "Recherche d'un appareil annoncé en mDNS…".into();
            }
            WifiMsg::Discovered(Some(addr)) => {
                self.discovered = Some(addr.clone());
                self.phase = Phase::Idle;
                self.hint = format!("Appareil détecté : {addr} — cliquez « Connecter ».");
            }
            WifiMsg::Discovered(None) => {
                self.discovered = None;
                self.phase = Phase::Idle;
                self.hint =
                    "Aucun appareil annoncé en mDNS (« Débogage sans fil » actif ?).".into();
            }
            WifiMsg::ConnectDiscoveredRequested => {
                self.hint = match &self.discovered {
                    Some(addr) => format!("Connexion à {addr}…"),
                    None => "Détectez d'abord un appareil en mDNS.".into(),
                };
            }
            WifiMsg::ConnectResult(Ok(addr)) => {
                self.phase = Phase::Connected(addr.clone());
                self.hint = format!("Connecté : {addr}");
            }
            WifiMsg::ConnectResult(Err(e)) => {
                self.phase = Phase::Failed(e.clone());
                self.hint = format!("Échec de connexion : {e}");
            }
            WifiMsg::Reset => *self = WifiState::new(),
        }
    }
}

/// Normalise l'adresse découverte, si elle est exploitable (réutilise la
/// validation `ip:port` de #16). Rend l'écran robuste à un libellé mDNS
/// inattendu. **Pur.**
pub fn usable_addr(addr: &str) -> Option<String> {
    connection::normalize_addr(addr).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etat_initial() {
        let s = WifiState::new();
        assert_eq!(s.phase, Phase::Idle);
        assert_eq!(s.port, "5555");
        assert!(s.serial.is_empty());
        assert!(s.discovered.is_none());
        assert!(!s.reconnect);
    }

    #[test]
    fn port_valide_et_invalide() {
        let mut s = WifiState::new();
        assert_eq!(s.port_num(), Some(5555));
        s.reduce(WifiMsg::PortChanged("40000".into()));
        assert_eq!(s.port_num(), Some(40000));
        s.reduce(WifiMsg::PortChanged("0".into()));
        assert_eq!(s.port_num(), None);
        s.reduce(WifiMsg::PortChanged("abc".into()));
        assert_eq!(s.port_num(), None);
        s.reduce(WifiMsg::PortChanged("70000".into())); // > u16
        assert_eq!(s.port_num(), None);
    }

    #[test]
    fn attempts_tolerant() {
        let mut s = WifiState::new();
        assert_eq!(s.attempts_num(), 10);
        s.reduce(WifiMsg::AttemptsChanged("3".into()));
        assert_eq!(s.attempts_num(), 3);
        s.reduce(WifiMsg::AttemptsChanged("0".into()));
        assert_eq!(s.attempts_num(), 1); // ramené à 1 au minimum
        s.reduce(WifiMsg::AttemptsChanged("".into()));
        assert_eq!(s.attempts_num(), 10); // défaut
        s.reduce(WifiMsg::AttemptsChanged("oops".into()));
        assert_eq!(s.attempts_num(), 10);
    }

    #[test]
    fn backoff_depend_de_la_reconnexion() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::AttemptsChanged("5".into()));
        // Reconnexion désactivée : un seul essai.
        assert_eq!(s.effective_attempts(), 1);
        assert_eq!(s.backoff().max_attempts, 1);
        // Activée : le nombre saisi.
        s.reduce(WifiMsg::ReconnectToggled(true));
        assert_eq!(s.effective_attempts(), 5);
        assert_eq!(s.backoff().max_attempts, 5);
    }

    #[test]
    fn can_switch_exige_serie_et_port() {
        let mut s = WifiState::new();
        assert!(!s.can_switch()); // pas de série
        s.reduce(WifiMsg::SerialChanged("ABC123".into()));
        assert!(s.can_switch());
        s.reduce(WifiMsg::PortChanged("0".into()));
        assert!(!s.can_switch()); // port invalide
    }

    #[test]
    fn switch_refuse_sans_serie() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::SwitchRequested);
        assert!(matches!(s.phase, Phase::Failed(_)));
        assert!(s.hint.to_lowercase().contains("série"));
    }

    #[test]
    fn switch_refuse_port_invalide() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::SerialChanged("ABC123".into()));
        s.reduce(WifiMsg::PortChanged("0".into()));
        s.reduce(WifiMsg::SwitchRequested);
        assert!(matches!(s.phase, Phase::Failed(_)));
        assert!(s.hint.to_lowercase().contains("port"));
    }

    #[test]
    fn switch_valide_passe_en_bascule_puis_connecte() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::SerialChanged("ABC123".into()));
        s.reduce(WifiMsg::SwitchRequested);
        assert_eq!(s.phase, Phase::Switching);
        s.reduce(WifiMsg::SwitchResult(Ok("192.168.1.42:5555".into())));
        assert_eq!(s.phase, Phase::Connected("192.168.1.42:5555".into()));
        // L'adresse obtenue devient l'adresse de connexion découverte.
        assert_eq!(s.discovered.as_deref(), Some("192.168.1.42:5555"));
        assert!(s.hint.contains("192.168.1.42:5555"));
    }

    #[test]
    fn switch_echec_affiche_erreur() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::SerialChanged("ABC123".into()));
        s.reduce(WifiMsg::SwitchRequested);
        s.reduce(WifiMsg::SwitchResult(Err("adb tcpip a échoué".into())));
        assert!(matches!(s.phase, Phase::Failed(_)));
        assert!(s.hint.contains("adb tcpip"));
    }

    #[test]
    fn decouverte_remplit_ou_vide() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::DiscoverRequested);
        assert_eq!(s.phase, Phase::Discovering);
        s.reduce(WifiMsg::Discovered(Some("10.0.0.9:39001".into())));
        assert!(s.can_connect_discovered());
        assert_eq!(s.phase, Phase::Idle);
        s.reduce(WifiMsg::Discovered(None));
        assert!(!s.can_connect_discovered());
        assert!(s.hint.to_lowercase().contains("mdns"));
    }

    #[test]
    fn connexion_decouverte_sans_cible_invite() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::ConnectDiscoveredRequested);
        assert!(s.hint.to_lowercase().contains("détectez"));
    }

    #[test]
    fn resultat_connexion() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::Discovered(Some("10.0.0.9:39001".into())));
        s.reduce(WifiMsg::ConnectResult(Ok("10.0.0.9:39001".into())));
        assert_eq!(s.phase, Phase::Connected("10.0.0.9:39001".into()));
        s.reduce(WifiMsg::ConnectResult(Err("injoignable".into())));
        assert!(matches!(s.phase, Phase::Failed(_)));
        assert!(s.hint.contains("Échec"));
    }

    #[test]
    fn reset_revient_a_zero() {
        let mut s = WifiState::new();
        s.reduce(WifiMsg::SerialChanged("ABC".into()));
        s.reduce(WifiMsg::Discovered(Some("10.0.0.9:39001".into())));
        s.reduce(WifiMsg::ReconnectToggled(true));
        s.reduce(WifiMsg::Reset);
        assert_eq!(s, WifiState::new());
    }

    #[test]
    fn adresse_exploitable() {
        assert_eq!(
            usable_addr("10.0.0.9:39001").as_deref(),
            Some("10.0.0.9:39001")
        );
        assert_eq!(usable_addr("10.0.0.9").as_deref(), Some("10.0.0.9:5555"));
        assert!(usable_addr("").is_none());
    }
}
