//! Vue unifiée des appareils (#30) : agrège les appareils connus d'adb
//! (`adb devices`, USB ou déjà connectés en TCP/IP) et ceux seulement
//! **annoncés** en « Débogage sans fil » via mDNS (#28), en une seule liste.
//!
//! L'**agrégation** (fusion + déduplication) et le **formatage** du tableau
//! sont purs et testés ; la collecte (`adb devices`, `adb mdns services`) est
//! l'I/O validée sur matériel. La commande n'effectue aucun déploiement.

use anyhow::Result;

use crate::device::adb::{self, Device};
use crate::device::mdns::{self, MdnsService, SERVICE_PAIRING};

/// Mode de connexion d'une entrée listée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Appareil branché en USB.
    Usb,
    /// Appareil joignable/annoncé sur le réseau (Wi-Fi).
    Wireless,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Usb => "USB",
            Kind::Wireless => "Wi-Fi",
        }
    }

    /// Icône (emoji) illustrant le mode de connexion, pour la liste GUI.
    pub fn icon(&self) -> &'static str {
        match self {
            Kind::Usb => "🔌",
            Kind::Wireless => "📶",
        }
    }
}

/// Icône (emoji) d'un état lisible d'appareil, pour la liste GUI. **Pur.**
pub fn status_icon(status: &str) -> &'static str {
    let s = status.to_lowercase();
    if s.contains("prêt") {
        "🟢"
    } else if s.contains("non autoris") {
        "🟠"
    } else if s.contains("hors ligne") {
        "🔴"
    } else if s.contains("annoncé") {
        "🔵"
    } else {
        "⚪"
    }
}

/// Une entrée de la liste unifiée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub kind: Kind,
    /// Série (USB) ou adresse `ip:port` (réseau).
    pub id: String,
    /// État lisible (« prêt », « non autorisé », « annoncé (mDNS) »…).
    pub status: String,
}

impl Entry {
    /// Icône d'état (emoji) de l'entrée.
    pub fn status_icon(&self) -> &'static str {
        status_icon(&self.status)
    }

    /// L'appareil est-il prêt à être utilisé (donc à ouvrir en miroir) ?
    /// Les appareils seulement « annoncés » en mDNS ne le sont pas encore.
    pub fn is_ready(&self) -> bool {
        self.status.to_lowercase().contains("prêt")
    }
}

/// Un identifiant adb ressemble-t-il à une cible réseau `host:port` ?
///
/// Un appareil connecté en TCP/IP apparaît dans `adb devices` avec une série
/// de la forme `192.168.1.42:5555`, là où une série USB est alphanumérique et
/// un émulateur s'appelle `emulator-5554`. Logique **pure**.
pub fn looks_like_network(serial: &str) -> bool {
    match serial.rsplit_once(':') {
        Some((host, port)) => {
            !host.is_empty()
                && host.contains('.')
                && !port.is_empty()
                && port.bytes().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
}

/// Construit la liste unifiée à partir des appareils adb et des services mDNS.
///
/// - chaque appareil adb devient une entrée (USB ou Wi-Fi selon sa série) ;
/// - chaque service mDNS **non déjà présent** (même adresse) devient une
///   entrée Wi-Fi « annoncée », en distinguant connexion et appairage.
///
/// Logique **pure et testable**.
pub fn build_entries(devices: &[Device], services: &[MdnsService]) -> Vec<Entry> {
    let mut entries: Vec<Entry> = devices
        .iter()
        .map(|d| Entry {
            kind: if looks_like_network(&d.serial) {
                Kind::Wireless
            } else {
                Kind::Usb
            },
            id: d.serial.clone(),
            status: d.state.label(),
        })
        .collect();

    for s in services {
        // Évite les doublons : un appareil déjà connecté à cette adresse.
        if entries.iter().any(|e| e.id == s.addr) {
            continue;
        }
        let status = if s.service_type == SERVICE_PAIRING {
            "annoncé (appairage, mDNS)"
        } else {
            "annoncé (connexion, mDNS)"
        };
        entries.push(Entry {
            kind: Kind::Wireless,
            id: s.addr.clone(),
            status: status.to_string(),
        });
    }

    entries
}

/// Formate la liste en tableau texte aligné. Logique **pure**.
pub fn format_entries(entries: &[Entry]) -> String {
    if entries.is_empty() {
        return "Aucun appareil détecté (USB ou « Débogage sans fil »).".to_string();
    }

    // Largeur de la colonne « id » (au moins l'en-tête).
    let id_width = entries
        .iter()
        .map(|e| e.id.len())
        .max()
        .unwrap_or(0)
        .max("Appareil".len());

    let mut out = String::new();
    out.push_str(&format!(
        "{:<6}  {:<width$}  {}\n",
        "Type",
        "Appareil",
        "État",
        width = id_width
    ));
    out.push_str(&format!(
        "{}  {}  {}\n",
        "-".repeat(6),
        "-".repeat(id_width),
        "----"
    ));
    for e in entries {
        out.push_str(&format!(
            "{:<6}  {:<width$}  {}\n",
            e.kind.as_str(),
            e.id,
            e.status,
            width = id_width
        ));
    }
    out
}

/// Collecte l'état réel (adb + mDNS) et renvoie les entrées. La découverte
/// mDNS est optionnelle : si elle échoue (adb trop ancien, pas de réseau), on
/// liste au moins les appareils adb.
pub fn gather() -> Result<Vec<Entry>> {
    let devices = adb::list_devices()?;
    let services = mdns::discover_services().unwrap_or_default();
    Ok(build_entries(&devices, &services))
}

/// Affiche la vue unifiée des appareils (I/O). N'effectue aucun déploiement.
pub fn run() -> Result<()> {
    let entries = gather()?;
    print!("{}", format_entries(&entries));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::adb::{Device, DeviceState};
    use crate::device::mdns::{SERVICE_CONNECT, SERVICE_PAIRING};

    fn dev(serial: &str, state: DeviceState) -> Device {
        Device {
            serial: serial.into(),
            state,
        }
    }

    fn svc(service_type: &str, addr: &str) -> MdnsService {
        MdnsService {
            name: "adb-XXXX".into(),
            service_type: service_type.into(),
            addr: addr.into(),
        }
    }

    #[test]
    fn reseau_vs_usb() {
        assert!(looks_like_network("192.168.1.42:5555"));
        assert!(!looks_like_network("ABC123XYZ"));
        assert!(!looks_like_network("emulator-5554"));
        assert!(!looks_like_network("host:5555")); // pas d'IP (pas de point)
        assert!(!looks_like_network("192.168.1.42:")); // port manquant
    }

    #[test]
    fn classe_usb_et_reseau() {
        let devices = vec![
            dev("ABC123", DeviceState::Online),
            dev("192.168.1.42:5555", DeviceState::Online),
        ];
        let e = build_entries(&devices, &[]);
        assert_eq!(e[0].kind, Kind::Usb);
        assert_eq!(e[1].kind, Kind::Wireless);
        assert_eq!(e[0].status, "prêt");
    }

    #[test]
    fn ajoute_services_annonces() {
        let devices = vec![dev("ABC123", DeviceState::Online)];
        let services = vec![
            svc(SERVICE_CONNECT, "10.0.0.9:37123"),
            svc(SERVICE_PAIRING, "10.0.0.9:37000"),
        ];
        let e = build_entries(&devices, &services);
        assert_eq!(e.len(), 3);
        assert_eq!(e[1].id, "10.0.0.9:37123");
        assert!(e[1].status.contains("connexion"));
        assert!(e[2].status.contains("appairage"));
    }

    #[test]
    fn dedup_appareil_deja_connecte() {
        // L'appareil est déjà connecté à 10.0.0.9:5555 ; le service mDNS de
        // même adresse ne doit pas créer de doublon.
        let devices = vec![dev("10.0.0.9:5555", DeviceState::Online)];
        let services = vec![svc(SERVICE_CONNECT, "10.0.0.9:5555")];
        let e = build_entries(&devices, &services);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, Kind::Wireless);
        assert_eq!(e[0].status, "prêt");
    }

    #[test]
    fn format_vide_est_lisible() {
        assert!(format_entries(&[]).contains("Aucun appareil"));
    }

    #[test]
    fn icones_type_et_etat() {
        assert_eq!(Kind::Usb.icon(), "🔌");
        assert_eq!(Kind::Wireless.icon(), "📶");
        assert_eq!(status_icon("prêt"), "🟢");
        assert_eq!(status_icon("non autorisé"), "🟠");
        assert_eq!(status_icon("hors ligne"), "🔴");
        assert_eq!(status_icon("annoncé (connexion, mDNS)"), "🔵");
        assert_eq!(status_icon("truc inconnu"), "⚪");
    }

    #[test]
    fn entry_prete_ou_non() {
        let pret = Entry {
            kind: Kind::Usb,
            id: "ABC".into(),
            status: "prêt".into(),
        };
        let annonce = Entry {
            kind: Kind::Wireless,
            id: "10.0.0.9:37123".into(),
            status: "annoncé (connexion, mDNS)".into(),
        };
        assert!(pret.is_ready());
        assert!(!annonce.is_ready());
        assert_eq!(pret.status_icon(), "🟢");
        assert_eq!(annonce.status_icon(), "🔵");
    }

    #[test]
    fn format_contient_entete_et_lignes() {
        let e = vec![Entry {
            kind: Kind::Usb,
            id: "ABC".into(),
            status: "prêt".into(),
        }];
        let out = format_entries(&e);
        assert!(out.contains("Type"));
        assert!(out.contains("Appareil"));
        assert!(out.contains("USB"));
        assert!(out.contains("ABC"));
        assert!(out.contains("prêt"));
    }
}
