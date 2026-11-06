//! Modes de connexion à l'appareil.
//!
//! gninou supporte trois voies, toutes via adb :
//!   - USB      : câble, appareil détecté par son numéro de série.
//!   - TCP/IP   : `adb connect ip:port` (Wi-Fi « classique »).
//!   - Wi-Fi    : appairage sans fil (Android 11+), via `adb pair`.
//!
//! Wi-Fi « classique » et TCP/IP partagent le même transport ; la
//! distinction porte sur l'étape d'appairage initiale.

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Port adb par défaut si l'adresse n'en précise pas.
pub const DEFAULT_ADB_PORT: u16 = 5555;

/// Comment joindre l'appareil.
#[derive(Debug, Clone)]
pub enum Connection {
    /// Connexion filaire USB. `serial` cible un appareil précis si plusieurs.
    Usb { serial: Option<String> },
    /// Connexion réseau directe `ip:port` (Wi-Fi/TCP-IP).
    TcpIp(String),
}

/// Valide et normalise une adresse `hôte[:port]` ; le port vaut
/// [`DEFAULT_ADB_PORT`] s'il est absent. Logique **pure et testable**.
pub fn parse_addr(addr: &str) -> Result<(String, u16)> {
    let addr = addr.trim();
    if addr.is_empty() {
        bail!("adresse vide");
    }
    let (host, port) = match addr.rsplit_once(':') {
        Some((h, p)) => {
            let port: u16 = p
                .parse()
                .map_err(|_| anyhow::anyhow!("port invalide : « {p} »"))?;
            if port == 0 {
                bail!("port invalide : 0");
            }
            (h, port)
        }
        None => (addr, DEFAULT_ADB_PORT),
    };
    if host.is_empty() {
        bail!("hôte vide dans « {addr} »");
    }
    Ok((host.to_string(), port))
}

/// Adresse normalisée `hôte:port`.
pub fn normalize_addr(addr: &str) -> Result<String> {
    let (host, port) = parse_addr(addr)?;
    Ok(format!("{host}:{port}"))
}

impl Connection {
    /// Prépare la connexion (ex. `adb connect` pour le TCP/IP).
    pub fn establish(&self) -> Result<()> {
        match self {
            Connection::Usb { serial } => {
                log::debug!("connexion USB (serial={serial:?})");
                Ok(())
            }
            Connection::TcpIp(addr) => {
                let target = normalize_addr(addr)?;
                log::info!("connexion TCP/IP vers {target}");
                let status = Command::new("adb")
                    .arg("connect")
                    .arg(&target)
                    .status()
                    .context("échec de `adb connect`")?;
                anyhow::ensure!(status.success(), "`adb connect {target}` a échoué");
                Ok(())
            }
        }
    }

    /// Numéro de série adb correspondant (pour cibler l'appareil ensuite).
    pub fn serial(&self) -> Option<String> {
        match self {
            Connection::Usb { serial } => serial.clone(),
            Connection::TcpIp(addr) => normalize_addr(addr).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addr_avec_port() {
        assert_eq!(parse_addr("192.168.1.42:5555").unwrap(), ("192.168.1.42".into(), 5555));
    }

    #[test]
    fn addr_sans_port_defaut() {
        assert_eq!(parse_addr("10.0.0.5").unwrap(), ("10.0.0.5".into(), DEFAULT_ADB_PORT));
    }

    #[test]
    fn addr_port_invalide() {
        assert!(parse_addr("10.0.0.5:abc").is_err());
        assert!(parse_addr("10.0.0.5:0").is_err());
        assert!(parse_addr("10.0.0.5:99999").is_err());
    }

    #[test]
    fn addr_vide_ou_hote_vide() {
        assert!(parse_addr("").is_err());
        assert!(parse_addr(":5555").is_err());
    }

    #[test]
    fn normalisation() {
        assert_eq!(normalize_addr("10.0.0.5").unwrap(), "10.0.0.5:5555");
        assert_eq!(normalize_addr("10.0.0.5:5037").unwrap(), "10.0.0.5:5037");
    }

    #[test]
    fn serial_tcpip_normalise() {
        let c = Connection::TcpIp("10.0.0.5".into());
        assert_eq!(c.serial(), Some("10.0.0.5:5555".into()));
    }
}
