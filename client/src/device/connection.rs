//! Modes de connexion à l'appareil.
//!
//! gninou supporte trois voies, toutes via adb :
//!   - USB      : câble, appareil détecté par son numéro de série.
//!   - TCP/IP   : `adb connect ip:port` (Wi-Fi « classique »).
//!   - Wi-Fi    : appairage sans fil (Android 11+), via `adb pair`.
//!
//! Wi-Fi « classique » et TCP/IP partagent le même transport ; la
//! distinction porte sur l'étape d'appairage initiale.

/// Comment joindre l'appareil.
#[derive(Debug, Clone)]
pub enum Connection {
    /// Connexion filaire USB. `serial` cible un appareil précis si plusieurs.
    Usb { serial: Option<String> },
    /// Connexion réseau directe `ip:port` (Wi-Fi/TCP-IP).
    TcpIp(String),
}

impl Connection {
    /// Prépare la connexion (ex. `adb connect` pour le TCP/IP).
    pub fn establish(&self) -> anyhow::Result<()> {
        match self {
            Connection::Usb { serial } => {
                log::debug!("connexion USB (serial={serial:?})");
                // TODO: vérifier la présence de l'appareil via `adb devices`
                Ok(())
            }
            Connection::TcpIp(addr) => {
                log::debug!("connexion TCP/IP vers {addr}");
                // TODO: `adb connect <addr>`
                Ok(())
            }
        }
    }
}
