//! Tunnel adb entre le client et le serveur.
//!
//! Deux modes, tous deux via adb :
//!   - **reverse** (préféré) : le client écoute sur un port TCP local et adb
//!     mappe `localabstract:gninou` (côté device) vers ce port. Le serveur
//!     se connecte au socket local abstrait, le client `accept()`.
//!   - **forward** (repli) : adb mappe un port TCP local vers le socket abstrait
//!     du device ; le client se connecte à ce port.
//!
//! Sur ce tunnel transiteront trois canaux logiques : vidéo, audio, contrôle.

use anyhow::{Context, Result};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::process::Command;

/// Nom du socket local abstrait côté device.
pub const SOCKET_NAME: &str = "gninou";

/// Port TCP local par défaut côté client.
pub const DEFAULT_PORT: u16 = 27183;

/// Mode de tunnel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Reverse,
    Forward,
}

// --- Construction des arguments adb (logique pure, testable) ---

fn reverse_args(name: &str, port: u16) -> Vec<String> {
    vec![
        "reverse".into(),
        format!("localabstract:{name}"),
        format!("tcp:{port}"),
    ]
}

fn remove_reverse_args(name: &str) -> Vec<String> {
    vec![
        "reverse".into(),
        "--remove".into(),
        format!("localabstract:{name}"),
    ]
}

fn forward_args(port: u16, name: &str) -> Vec<String> {
    vec![
        "forward".into(),
        format!("tcp:{port}"),
        format!("localabstract:{name}"),
    ]
}

fn remove_forward_args(port: u16) -> Vec<String> {
    vec!["forward".into(), "--remove".into(), format!("tcp:{port}")]
}

fn adb(serial: Option<&str>) -> Command {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd
}

fn run(serial: Option<&str>, args: &[String], what: &str) -> Result<()> {
    let status = adb(serial)
        .args(args)
        .status()
        .with_context(|| format!("échec de `adb {what}`"))?;
    anyhow::ensure!(
        status.success(),
        "`adb {what}` a échoué (code {:?})",
        status.code()
    );
    Ok(())
}

/// Tunnel ouvert. En mode reverse, `listener` écoute les connexions du serveur.
pub struct Tunnel {
    pub mode: Mode,
    pub port: u16,
    name: String,
    serial: Option<String>,
    listener: Option<TcpListener>,
}

impl Tunnel {
    /// Ouvre un tunnel en mode **reverse** : écoute d'abord localement, puis
    /// installe la redirection adb (ordre important pour éviter une course).
    pub fn open_reverse(serial: Option<&str>, port: u16) -> Result<Tunnel> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .with_context(|| format!("écoute impossible sur 127.0.0.1:{port}"))?;
        run(serial, &reverse_args(SOCKET_NAME, port), "reverse")?;
        Ok(Tunnel {
            mode: Mode::Reverse,
            port,
            name: SOCKET_NAME.to_string(),
            serial: serial.map(str::to_string),
            listener: Some(listener),
        })
    }

    /// Ouvre un tunnel en mode **forward** (repli).
    pub fn open_forward(serial: Option<&str>, port: u16) -> Result<Tunnel> {
        run(serial, &forward_args(port, SOCKET_NAME), "forward")?;
        Ok(Tunnel {
            mode: Mode::Forward,
            port,
            name: SOCKET_NAME.to_string(),
            serial: serial.map(str::to_string),
            listener: None,
        })
    }

    /// Attend/établit une connexion (un canal) sur le tunnel.
    pub fn accept(&self) -> Result<TcpStream> {
        match self.mode {
            Mode::Reverse => {
                let listener = self
                    .listener
                    .as_ref()
                    .context("tunnel reverse sans listener")?;
                let (stream, _) = listener.accept().context("accept() a échoué")?;
                Ok(stream)
            }
            Mode::Forward => TcpStream::connect((Ipv4Addr::LOCALHOST, self.port))
                .with_context(|| format!("connexion impossible au port {}", self.port)),
        }
    }

    /// Retire la redirection adb. Appelé automatiquement à la destruction.
    pub fn close(&mut self) -> Result<()> {
        let args = match self.mode {
            Mode::Reverse => remove_reverse_args(&self.name),
            Mode::Forward => remove_forward_args(self.port),
        };
        run(self.serial.as_deref(), &args, "remove")
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        // Nettoyage best-effort : on ignore l'erreur à la destruction.
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_reverse() {
        assert_eq!(
            reverse_args("gninou", 27183),
            vec!["reverse", "localabstract:gninou", "tcp:27183"]
        );
    }

    #[test]
    fn args_reverse_remove() {
        assert_eq!(
            remove_reverse_args("gninou"),
            vec!["reverse", "--remove", "localabstract:gninou"]
        );
    }

    #[test]
    fn args_forward() {
        assert_eq!(
            forward_args(27183, "gninou"),
            vec!["forward", "tcp:27183", "localabstract:gninou"]
        );
    }

    #[test]
    fn args_forward_remove() {
        assert_eq!(
            remove_forward_args(27183),
            vec!["forward", "--remove", "tcp:27183"]
        );
    }

    #[test]
    fn listener_bind_sur_port_libre() {
        // Port 0 = l'OS choisit un port libre : vérifie qu'on sait écouter.
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        assert!(listener.local_addr().unwrap().port() > 0);
    }
}
