//! Pilotage d'`adb` : disponibilité, détection/sélection d'appareils, tunnel.

use anyhow::{Context, Result};
use std::process::Command;

/// État d'un appareil tel que rapporté par `adb devices`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceState {
    /// Prêt à l'emploi (`device`).
    Online,
    /// Débogage USB non autorisé sur l'appareil (`unauthorized`).
    Unauthorized,
    /// Connecté mais hors ligne (`offline`).
    Offline,
    /// Autre état rapporté par adb.
    Other(String),
}

impl DeviceState {
    fn parse(raw: &str) -> DeviceState {
        match raw {
            "device" => DeviceState::Online,
            "unauthorized" => DeviceState::Unauthorized,
            "offline" => DeviceState::Offline,
            other => DeviceState::Other(other.to_string()),
        }
    }
}

/// Un appareil détecté par adb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub serial: String,
    pub state: DeviceState,
}

impl Device {
    pub fn is_online(&self) -> bool {
        self.state == DeviceState::Online
    }
}

/// Construit une commande `adb`, ciblant un appareil si `serial` est fourni.
fn adb(serial: Option<&str>) -> Command {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd
}

/// Vérifie qu'`adb` est disponible dans le PATH.
pub fn ensure_available() -> Result<()> {
    let output = Command::new("adb")
        .arg("version")
        .output()
        .context("`adb` introuvable dans le PATH")?;

    if !output.status.success() {
        anyhow::bail!("`adb version` a échoué");
    }
    Ok(())
}

/// Liste les appareils connus d'adb.
pub fn list_devices() -> Result<Vec<Device>> {
    let output = adb(None)
        .arg("devices")
        .output()
        .context("échec de `adb devices`")?;

    anyhow::ensure!(output.status.success(), "`adb devices` a échoué");
    Ok(parse_devices(&String::from_utf8_lossy(&output.stdout)))
}

/// Analyse la sortie de `adb devices` (logique pure, testable sans adb).
fn parse_devices(output: &str) -> Vec<Device> {
    output
        .lines()
        .map(str::trim)
        // Ignore l'en-tête, les lignes vides et les lignes de daemon.
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("List of devices")
                && !l.starts_with('*')
        })
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?.to_string();
            let state = parts.next().unwrap_or("");
            Some(Device {
                serial,
                state: DeviceState::parse(state),
            })
        })
        .collect()
}

/// Choisit l'appareil à utiliser parmi ceux détectés.
///
/// - `serial` fourni : renvoie cet appareil s'il est en ligne.
/// - sinon : renvoie l'unique appareil en ligne, ou une erreur si 0 ou plusieurs.
pub fn select_device(devices: &[Device], serial: Option<&str>) -> Result<Device> {
    if let Some(wanted) = serial {
        let dev = devices
            .iter()
            .find(|d| d.serial == wanted)
            .with_context(|| format!("aucun appareil avec le numéro de série « {wanted} »"))?;
        anyhow::ensure!(
            dev.is_online(),
            "l'appareil « {wanted} » n'est pas prêt (état : {:?})",
            dev.state
        );
        return Ok(dev.clone());
    }

    let online: Vec<&Device> = devices.iter().filter(|d| d.is_online()).collect();
    match online.as_slice() {
        [] => {
            // Message plus utile si des appareils existent mais non prêts.
            if devices.is_empty() {
                anyhow::bail!("aucun appareil détecté (branchez l'appareil, activez le débogage USB)");
            }
            anyhow::bail!(
                "aucun appareil prêt ; états détectés : {:?}",
                devices.iter().map(|d| (&d.serial, &d.state)).collect::<Vec<_>>()
            );
        }
        [only] => Ok((*only).clone()),
        many => anyhow::bail!(
            "plusieurs appareils détectés, précisez --serial parmi : {:?}",
            many.iter().map(|d| &d.serial).collect::<Vec<_>>()
        ),
    }
}

/// Sélectionne **plusieurs** appareils cibles (fonctionnalité multi-appareils).
///
/// - `all` vrai : tous les appareils en ligne (erreur si aucun).
/// - sinon, `wanted` non vide : chaque série demandée, qui doit exister et être
///   en ligne ; les doublons sont ignorés en conservant l'ordre de la demande.
/// - sinon (`wanted` vide, `all` faux) : l'unique appareil en ligne, via
///   [`select_device`] (erreur si 0 ou plusieurs).
///
/// Logique **pure et testable**.
pub fn select_devices(devices: &[Device], wanted: &[String], all: bool) -> Result<Vec<Device>> {
    if all {
        let online: Vec<Device> = devices.iter().filter(|d| d.is_online()).cloned().collect();
        anyhow::ensure!(
            !online.is_empty(),
            "aucun appareil prêt à piloter (option --all)"
        );
        return Ok(online);
    }

    if !wanted.is_empty() {
        let mut selected: Vec<Device> = Vec::new();
        for w in wanted {
            if selected.iter().any(|d| &d.serial == w) {
                continue; // doublon explicite : ignoré
            }
            selected.push(select_device(devices, Some(w))?);
        }
        return Ok(selected);
    }

    Ok(vec![select_device(devices, None)?])
}

/// Détecte et sélectionne l'appareil cible en une étape.
pub fn detect(serial: Option<&str>) -> Result<Device> {
    let devices = list_devices()?;
    select_device(&devices, serial)
}

/// Détecte et sélectionne la liste d'appareils cibles en une étape.
pub fn detect_targets(wanted: &[String], all: bool) -> Result<Vec<Device>> {
    let devices = list_devices()?;
    select_devices(&devices, wanted, all)
}

/// Ouvre un tunnel (reverse de préférence, forward en repli). — à venir (#3)
pub fn open_tunnel(_serial: Option<&str>, _port: u16) -> Result<()> {
    // TODO: adb reverse localabstract:gninou tcp:<port>
    todo!("ouverture de tunnel non implémentée")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vide() {
        assert!(parse_devices("List of devices attached\n").is_empty());
    }

    #[test]
    fn parse_un_appareil() {
        let out = "List of devices attached\nABC123\tdevice\n";
        let devs = parse_devices(out);
        assert_eq!(devs.len(), 1);
        assert_eq!(devs[0].serial, "ABC123");
        assert_eq!(devs[0].state, DeviceState::Online);
    }

    #[test]
    fn parse_etats_varies() {
        let out = "List of devices attached\n\
                   ABC\tdevice\n\
                   XYZ\tunauthorized\n\
                   emulator-5554\toffline\n\
                   * daemon started successfully *\n";
        let devs = parse_devices(out);
        assert_eq!(devs.len(), 3);
        assert_eq!(devs[1].state, DeviceState::Unauthorized);
        assert_eq!(devs[2].state, DeviceState::Offline);
    }

    fn online(serial: &str) -> Device {
        Device { serial: serial.into(), state: DeviceState::Online }
    }

    #[test]
    fn select_unique_sans_serial() {
        let devs = vec![online("ONE")];
        assert_eq!(select_device(&devs, None).unwrap().serial, "ONE");
    }

    #[test]
    fn select_aucun_appareil() {
        assert!(select_device(&[], None).is_err());
    }

    #[test]
    fn select_ambigu_sans_serial() {
        let devs = vec![online("A"), online("B")];
        assert!(select_device(&devs, None).is_err());
    }

    #[test]
    fn select_par_serial() {
        let devs = vec![online("A"), online("B")];
        assert_eq!(select_device(&devs, Some("B")).unwrap().serial, "B");
    }

    #[test]
    fn select_serial_introuvable() {
        let devs = vec![online("A")];
        assert!(select_device(&devs, Some("Z")).is_err());
    }

    #[test]
    fn select_serial_non_pret() {
        let devs = vec![Device { serial: "A".into(), state: DeviceState::Unauthorized }];
        assert!(select_device(&devs, Some("A")).is_err());
    }

    // --- Multi-appareils (#18) ---

    fn serials(devs: &[Device]) -> Vec<String> {
        devs.iter().map(|d| d.serial.clone()).collect()
    }

    #[test]
    fn multi_all_en_ligne() {
        let devs = vec![
            online("A"),
            Device { serial: "B".into(), state: DeviceState::Offline },
            online("C"),
        ];
        let got = select_devices(&devs, &[], true).unwrap();
        assert_eq!(serials(&got), vec!["A", "C"]); // l'offline est exclu
    }

    #[test]
    fn multi_all_aucun_pret() {
        let devs = vec![Device { serial: "A".into(), state: DeviceState::Offline }];
        assert!(select_devices(&devs, &[], true).is_err());
    }

    #[test]
    fn multi_liste_explicite_ordre_conserve() {
        let devs = vec![online("A"), online("B"), online("C")];
        let got = select_devices(&devs, &["C".into(), "A".into()], false).unwrap();
        assert_eq!(serials(&got), vec!["C", "A"]);
    }

    #[test]
    fn multi_doublons_ignores() {
        let devs = vec![online("A"), online("B")];
        let got = select_devices(&devs, &["A".into(), "A".into(), "B".into()], false).unwrap();
        assert_eq!(serials(&got), vec!["A", "B"]);
    }

    #[test]
    fn multi_serial_inconnu_echoue() {
        let devs = vec![online("A")];
        assert!(select_devices(&devs, &["A".into(), "Z".into()], false).is_err());
    }

    #[test]
    fn multi_vide_delegue_unique() {
        let devs = vec![online("ONE")];
        let got = select_devices(&devs, &[], false).unwrap();
        assert_eq!(serials(&got), vec!["ONE"]);
    }
}
