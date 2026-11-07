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

/// Extrait la première adresse IPv4 exploitable d'une sortie `ip addr`
/// (`adb shell ip -f inet addr show wlan0`). Ignore `127.0.0.1`.
/// Logique **pure et testable**.
pub fn parse_wlan_ipv4(output: &str) -> Result<String> {
    for line in output.lines() {
        let line = line.trim();
        let rest = match line.strip_prefix("inet ") {
            Some(r) => r,
            None => continue,
        };
        // `inet 192.168.1.42/24 brd ... scope global wlan0`
        let addr = rest.split_whitespace().next().unwrap_or("");
        let ip = addr.split('/').next().unwrap_or("");
        if ip != "127.0.0.1" && ip.parse::<std::net::Ipv4Addr>().is_ok() {
            return Ok(ip.to_string());
        }
    }
    bail!("aucune adresse IPv4 Wi-Fi trouvée (l'appareil est-il connecté au Wi-Fi ?)");
}

/// Bascule un appareil **USB** en Wi-Fi et s'y connecte, sans IP à saisir :
/// lit l'IP Wi-Fi de l'appareil, active le mode TCP/IP (`adb tcpip`), puis
/// `adb connect ip:port`. Renvoie l'adresse réseau `ip:port`.
pub fn switch_to_wifi(serial: &str, port: u16) -> Result<String> {
    // 1. Lire l'IP Wi-Fi de l'appareil (interface wlan0).
    let out = Command::new("adb")
        .args([
            "-s", serial, "shell", "ip", "-f", "inet", "addr", "show", "wlan0",
        ])
        .output()
        .context("échec de la lecture de l'IP Wi-Fi (`adb shell ip addr`)")?;
    anyhow::ensure!(
        out.status.success(),
        "`adb shell ip addr show wlan0` a échoué"
    );
    let ip = parse_wlan_ipv4(&String::from_utf8_lossy(&out.stdout))?;
    log::info!("IP Wi-Fi de l'appareil : {ip}");

    // 2. Basculer le démon adb de l'appareil en TCP/IP.
    let st = Command::new("adb")
        .args(["-s", serial, "tcpip", &port.to_string()])
        .status()
        .context("échec de `adb tcpip`")?;
    anyhow::ensure!(st.success(), "`adb tcpip {port}` a échoué");

    // 3. Se connecter en réseau (réutilise la logique TCP/IP de #16).
    let target = format!("{ip}:{port}");
    let st2 = Command::new("adb")
        .args(["connect", &target])
        .status()
        .context("échec de `adb connect`")?;
    anyhow::ensure!(st2.success(), "`adb connect {target}` a échoué");
    Ok(target)
}

/// Longueur du code d'appairage Wi-Fi (Android 11+).
pub const PAIR_CODE_LEN: usize = 6;

/// Valide un code d'appairage Wi-Fi : exactement 6 chiffres.
/// Logique **pure et testable**.
pub fn validate_pair_code(code: &str) -> Result<String> {
    let code = code.trim();
    if code.len() != PAIR_CODE_LEN || !code.bytes().all(|b| b.is_ascii_digit()) {
        bail!("code d'appairage invalide : « {code} » (6 chiffres attendus)");
    }
    Ok(code.to_string())
}

/// Arguments `adb` pour l'appairage Wi-Fi : `pair <hôte:port> <code>`.
/// Valide l'adresse **et** le code. Logique **pure et testable**.
///
/// L'adresse et le port d'appairage sont ceux affichés par l'appareil dans
/// « Débogage sans fil → Associer l'appareil à l'aide d'un code » ; ce port
/// est distinct de celui utilisé ensuite pour `adb connect`.
pub fn pair_args(addr: &str, code: &str) -> Result<Vec<String>> {
    let target = normalize_addr(addr)?;
    let code = validate_pair_code(code)?;
    Ok(vec!["pair".to_string(), target, code])
}

/// Exécute l'appairage Wi-Fi (Android 11+) via `adb pair`.
pub fn pair(addr: &str, code: &str) -> Result<()> {
    let args = pair_args(addr, code)?;
    log::info!("appairage Wi-Fi vers {}", args[1]);
    let status = Command::new("adb")
        .args(&args)
        .status()
        .context("échec de `adb pair`")?;
    anyhow::ensure!(status.success(), "`adb pair {}` a échoué", args[1]);
    log::info!("appairage réussi ; connectez-vous ensuite avec --tcpip <ip:port>");
    Ok(())
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
    fn wlan_ipv4_extrait() {
        let out = "3: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP>\n    \
                   inet 192.168.1.42/24 brd 192.168.1.255 scope global wlan0\n       \
                   valid_lft forever preferred_lft forever\n";
        assert_eq!(parse_wlan_ipv4(out).unwrap(), "192.168.1.42");
    }

    #[test]
    fn wlan_ipv4_ignore_loopback() {
        let out = "1: lo\n    inet 127.0.0.1/8 scope host lo\n\
                   3: wlan0\n    inet 10.0.0.7/24 scope global wlan0\n";
        assert_eq!(parse_wlan_ipv4(out).unwrap(), "10.0.0.7");
    }

    #[test]
    fn wlan_ipv4_absente_erreur() {
        assert!(parse_wlan_ipv4("3: wlan0\n    (pas d'adresse)\n").is_err());
        assert!(parse_wlan_ipv4("").is_err());
    }

    #[test]
    fn addr_avec_port() {
        assert_eq!(
            parse_addr("192.168.1.42:5555").unwrap(),
            ("192.168.1.42".into(), 5555)
        );
    }

    #[test]
    fn addr_sans_port_defaut() {
        assert_eq!(
            parse_addr("10.0.0.5").unwrap(),
            ("10.0.0.5".into(), DEFAULT_ADB_PORT)
        );
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

    #[test]
    fn code_appairage_valide() {
        assert_eq!(validate_pair_code("123456").unwrap(), "123456");
        assert_eq!(validate_pair_code(" 000000 ").unwrap(), "000000");
    }

    #[test]
    fn code_appairage_invalide() {
        assert!(validate_pair_code("12345").is_err()); // trop court
        assert!(validate_pair_code("1234567").is_err()); // trop long
        assert!(validate_pair_code("12 456").is_err()); // non chiffre
        assert!(validate_pair_code("abcdef").is_err());
        assert!(validate_pair_code("").is_err());
    }

    #[test]
    fn pair_args_complets() {
        assert_eq!(
            pair_args("192.168.1.42:37123", "123456").unwrap(),
            vec!["pair", "192.168.1.42:37123", "123456"],
        );
    }

    #[test]
    fn pair_args_rejette_adresse_ou_code() {
        assert!(pair_args("", "123456").is_err());
        assert!(pair_args("192.168.1.42:37123", "oops").is_err());
    }
}
