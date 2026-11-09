//! View-model **pur** de l'écran d'appairage Wi-Fi (#33).
//!
//! Transpose l'assistant terminal (#29) en formulaire : adresse d'appairage
//! (auto-découverte en mDNS ou saisie), code à 6 chiffres validé en direct,
//! puis adresse de connexion. Aucune dépendance à egui ni à adb ici : seule la
//! validation du code réutilise [`connection::validate_pair_code`]. Le rendu
//! (`super::app`) lit cet état et émet des [`PairMsg`].

use crate::device::connection;

/// Étape de l'appairage guidé.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Phase {
    /// Saisie des informations (adresse + code).
    #[default]
    Input,
    /// Appairage `adb pair` en cours.
    Pairing,
    /// Appairé : prêt à se connecter.
    Paired,
    /// Connecté, avec l'adresse réseau.
    Connected(String),
    /// Échec, avec le message.
    Failed(String),
}

/// État du formulaire d'appairage.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PairingState {
    /// Adresse d'appairage `ip:port` (service `_adb-tls-pairing._tcp`).
    pub pair_addr: String,
    /// Adresse de connexion `ip:port` (service `_adb-tls-connect._tcp`).
    pub connect_addr: String,
    /// Code à 6 chiffres affiché par l'appareil.
    pub code: String,
    pub phase: Phase,
    /// Message d'aide ou d'erreur courant.
    pub hint: String,
}

/// Messages de l'écran d'appairage (actions UI + résultats I/O).
#[derive(Debug, Clone)]
pub enum PairMsg {
    PairAddrChanged(String),
    ConnectAddrChanged(String),
    CodeChanged(String),
    /// Résultat de la découverte mDNS du point d'appairage.
    PairDiscovered(Option<String>),
    /// Résultat de la découverte mDNS du point de connexion.
    ConnectDiscovered(Option<String>),
    PairRequested,
    PairResult(Result<(), String>),
    ConnectRequested,
    ConnectResult(Result<String, String>),
    /// Réinitialise le formulaire.
    Reset,
}

impl PairingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Le code saisi est-il un code d'appairage valide (6 chiffres) ?
    pub fn code_valid(&self) -> bool {
        connection::validate_pair_code(&self.code).is_ok()
    }

    /// Peut-on lancer l'appairage ? (adresse non vide + code valide, en phase
    /// de saisie ou après un échec).
    pub fn can_pair(&self) -> bool {
        !self.pair_addr.trim().is_empty()
            && self.code_valid()
            && matches!(self.phase, Phase::Input | Phase::Failed(_))
    }

    /// Peut-on lancer la connexion ? (appairé + adresse de connexion fournie).
    pub fn can_connect(&self) -> bool {
        self.phase == Phase::Paired && !self.connect_addr.trim().is_empty()
    }

    /// Message d'aide pour la saisie du code (vide → rien, sinon validité).
    fn code_hint(&self) -> String {
        if self.code.trim().is_empty() {
            String::new()
        } else if self.code_valid() {
            "Code valide.".to_string()
        } else {
            "Le code doit comporter exactement 6 chiffres.".to_string()
        }
    }

    /// Applique un message et fait évoluer l'état. **Pur et testable.**
    pub fn reduce(&mut self, msg: PairMsg) {
        match msg {
            PairMsg::PairAddrChanged(v) => self.pair_addr = v,
            PairMsg::ConnectAddrChanged(v) => self.connect_addr = v,
            PairMsg::CodeChanged(v) => {
                self.code = v;
                self.hint = self.code_hint();
            }
            PairMsg::PairDiscovered(found) => match found {
                Some(addr) => {
                    self.pair_addr = addr;
                    self.hint = "Point d'appairage détecté par mDNS.".to_string();
                }
                None => {
                    self.hint = "Aucun point d'appairage mDNS — saisissez l'adresse.".to_string();
                }
            },
            PairMsg::ConnectDiscovered(found) => match found {
                Some(addr) => {
                    self.connect_addr = addr;
                    self.hint = "Point de connexion détecté par mDNS.".to_string();
                }
                None => {
                    self.hint = "Aucun point de connexion mDNS — saisissez l'adresse.".to_string();
                }
            },
            PairMsg::PairRequested => {
                if self.can_pair() {
                    self.phase = Phase::Pairing;
                    self.hint = "Appairage en cours…".to_string();
                } else if !self.code_valid() {
                    self.hint = "Le code doit comporter exactement 6 chiffres.".to_string();
                } else {
                    self.hint = "Renseignez l'adresse d'appairage.".to_string();
                }
            }
            PairMsg::PairResult(Ok(())) => {
                self.phase = Phase::Paired;
                self.hint = "Appairage réussi — connectez-vous à l'appareil.".to_string();
            }
            PairMsg::PairResult(Err(e)) => {
                self.hint = format!("Échec de l'appairage : {e}");
                self.phase = Phase::Failed(e);
            }
            PairMsg::ConnectRequested => {
                if self.can_connect() {
                    self.hint = format!("Connexion à {}…", self.connect_addr.trim());
                } else {
                    self.hint =
                        "Appairez d'abord, puis renseignez l'adresse de connexion.".to_string();
                }
            }
            PairMsg::ConnectResult(Ok(addr)) => {
                self.hint = format!("Connecté : {addr}");
                self.phase = Phase::Connected(addr);
            }
            PairMsg::ConnectResult(Err(e)) => {
                self.hint = format!("Échec de connexion : {e}");
                self.phase = Phase::Failed(e);
            }
            PairMsg::Reset => *self = PairingState::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etat_initial() {
        let s = PairingState::new();
        assert_eq!(s.phase, Phase::Input);
        assert!(!s.can_pair());
        assert!(!s.can_connect());
    }

    #[test]
    fn validation_code() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::CodeChanged("12".into()));
        assert!(!s.code_valid());
        assert!(s.hint.contains("6 chiffres"));
        s.reduce(PairMsg::CodeChanged("123456".into()));
        assert!(s.code_valid());
        assert!(s.hint.contains("valide"));
    }

    #[test]
    fn can_pair_exige_adresse_et_code() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::CodeChanged("123456".into()));
        assert!(!s.can_pair()); // adresse manquante
        s.reduce(PairMsg::PairAddrChanged("192.168.1.42:37000".into()));
        assert!(s.can_pair());
    }

    #[test]
    fn decouverte_remplit_les_adresses() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::PairDiscovered(Some("10.0.0.9:37000".into())));
        assert_eq!(s.pair_addr, "10.0.0.9:37000");
        s.reduce(PairMsg::ConnectDiscovered(Some("10.0.0.9:37123".into())));
        assert_eq!(s.connect_addr, "10.0.0.9:37123");
        s.reduce(PairMsg::PairDiscovered(None));
        assert!(s.hint.contains("Aucun point d'appairage"));
    }

    #[test]
    fn flux_appairage_puis_connexion() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::PairAddrChanged("192.168.1.42:37000".into()));
        s.reduce(PairMsg::CodeChanged("123456".into()));
        s.reduce(PairMsg::PairRequested);
        assert_eq!(s.phase, Phase::Pairing);
        s.reduce(PairMsg::PairResult(Ok(())));
        assert_eq!(s.phase, Phase::Paired);
        assert!(!s.can_connect()); // adresse de connexion encore vide
        s.reduce(PairMsg::ConnectAddrChanged("192.168.1.42:37123".into()));
        assert!(s.can_connect());
        s.reduce(PairMsg::ConnectResult(Ok("192.168.1.42:37123".into())));
        assert_eq!(s.phase, Phase::Connected("192.168.1.42:37123".to_string()));
    }

    #[test]
    fn appairage_refuse_code_invalide() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::PairAddrChanged("192.168.1.42:37000".into()));
        s.reduce(PairMsg::CodeChanged("abc".into()));
        s.reduce(PairMsg::PairRequested);
        assert_eq!(s.phase, Phase::Input); // pas de passage en Pairing
        assert!(s.hint.contains("6 chiffres"));
    }

    #[test]
    fn echec_appairage_puis_reessai_possible() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::PairAddrChanged("192.168.1.42:37000".into()));
        s.reduce(PairMsg::CodeChanged("123456".into()));
        s.reduce(PairMsg::PairRequested);
        s.reduce(PairMsg::PairResult(Err("code refusé".into())));
        assert_eq!(s.phase, Phase::Failed("code refusé".into()));
        assert!(s.can_pair()); // on peut réessayer depuis l'état d'échec
    }

    #[test]
    fn reset_vide_le_formulaire() {
        let mut s = PairingState::new();
        s.reduce(PairMsg::PairAddrChanged("x".into()));
        s.reduce(PairMsg::CodeChanged("123456".into()));
        s.reduce(PairMsg::Reset);
        assert_eq!(s, PairingState::new());
    }
}
