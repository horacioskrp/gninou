//! Rendu egui/eframe de l'interface graphique (#32, #33).
//!
//! Fine couche de **vue** au-dessus des view-models purs ([`super::state`],
//! [`super::pairing`]) : elle lit l'état, dessine l'écran courant (Appareils ou
//! Appairage Wi-Fi) et traduit les clics en messages. La collecte
//! (`adb devices` + mDNS), l'appairage (`adb pair`) et la connexion
//! (`adb connect`) sont faits de façon synchrone pour ces premiers incréments
//! (🟨 : passage en tâche de fond à prévoir).

use eframe::egui;

use super::options::ROTATIONS;
use super::pairing::{PairMsg, Phase};
use super::state::{AppState, Load, Msg, Screen};
use super::wifi::{self, WifiMsg};
use crate::device::connection::{self, Connection};
use crate::device::listing::{self, Kind};
use crate::device::{mdns, reconnect};
use crate::media::codec::VideoCodec;

/// Application GUI gninou.
pub struct GninouApp {
    state: AppState,
}

impl GninouApp {
    fn new() -> Self {
        let mut app = GninouApp {
            state: AppState::new(),
        };
        // Première détection au démarrage pour afficher tout de suite la liste.
        app.refresh();
        app
    }

    /// (Re)charge la liste des appareils (I/O synchrone).
    fn refresh(&mut self) {
        self.state.reduce(Msg::RefreshRequested);
        match listing::gather() {
            Ok(list) => self.state.reduce(Msg::DevicesLoaded(list)),
            Err(e) => self.state.reduce(Msg::LoadFailed(e.to_string())),
        }
    }

    /// Connecte l'appareil sélectionné de l'écran Appareils (réseau seulement).
    fn connect_selected(&mut self) {
        self.state.reduce(Msg::ConnectRequested);
        let entry = match self.state.selected_entry() {
            Some(e) => e.clone(),
            None => return, // le reduce a déjà posé le message d'invite
        };

        if entry.kind != Kind::Wireless {
            self.state.reduce(Msg::ConnectResult(Err(
                "connexion directe réservée aux appareils Wi-Fi (USB : déjà relié)".into(),
            )));
            return;
        }

        let result = Connection::TcpIp(entry.id.clone())
            .establish()
            .map(|_| entry.id.clone())
            .map_err(|e| e.to_string());
        self.state.reduce(Msg::ConnectResult(result));
    }

    /// Découvre en mDNS les points d'appairage et de connexion (I/O).
    fn discover_pairing(&mut self) {
        let pair = mdns::discover_pairing_addr().ok().flatten();
        self.state.pairing.reduce(PairMsg::PairDiscovered(pair));
        let connect = mdns::discover_connect_addr().ok().flatten();
        self.state
            .pairing
            .reduce(PairMsg::ConnectDiscovered(connect));
    }

    /// Lance `adb pair` si le formulaire est valide (I/O).
    fn do_pair(&mut self) {
        self.state.pairing.reduce(PairMsg::PairRequested);
        if self.state.pairing.phase != Phase::Pairing {
            return; // refusé (adresse ou code invalide) : le hint explique
        }
        let addr = self.state.pairing.pair_addr.trim().to_string();
        let code = self.state.pairing.code.trim().to_string();
        let res = connection::pair(&addr, &code).map_err(|e| e.to_string());
        self.state.pairing.reduce(PairMsg::PairResult(res));
    }

    /// Lance `adb connect` vers l'adresse de connexion appairée (I/O).
    fn do_connect_pairing(&mut self) {
        self.state.pairing.reduce(PairMsg::ConnectRequested);
        if !self.state.pairing.can_connect() {
            return;
        }
        let addr = self.state.pairing.connect_addr.trim().to_string();
        let res = Connection::TcpIp(addr.clone())
            .establish()
            .map(|_| addr)
            .map_err(|e| e.to_string());
        self.state.pairing.reduce(PairMsg::ConnectResult(res));
    }

    /// Pré-remplit la série Wi-Fi avec le premier appareil USB détecté, si le
    /// champ est vide — petit confort à l'ouverture de l'écran (réutilise #30).
    fn prefill_wifi_serial(&mut self) {
        if !self.state.wifi.serial.trim().is_empty() {
            return;
        }
        if let Some(usb) = self
            .state
            .devices
            .iter()
            .find(|e| e.kind == Kind::Usb)
            .map(|e| e.id.clone())
        {
            self.state.wifi.reduce(WifiMsg::SerialChanged(usb));
        }
    }

    /// Bascule l'appareil USB en Wi-Fi (#27), avec reprise si demandée (#31).
    fn switch_wifi(&mut self) {
        self.state.wifi.reduce(WifiMsg::SwitchRequested);
        if self.state.wifi.phase != wifi::Phase::Switching {
            return; // refusé (série/port) : le hint explique
        }
        let serial = self.state.wifi.serial.trim().to_string();
        // can_switch() (vérifié via la phase) garantit un port valide.
        let port = self
            .state
            .wifi
            .port_num()
            .unwrap_or(connection::DEFAULT_ADB_PORT);
        let backoff = self.state.wifi.backoff();
        let res = reconnect::retry(backoff, || connection::switch_to_wifi(&serial, port))
            .map_err(|e| e.to_string());
        self.state.wifi.reduce(WifiMsg::SwitchResult(res));
    }

    /// Découvre en mDNS un appareil annonçant sa connexion (#28).
    fn discover_wifi(&mut self) {
        self.state.wifi.reduce(WifiMsg::DiscoverRequested);
        let found = mdns::discover_connect_addr()
            .ok()
            .flatten()
            .and_then(|a| wifi::usable_addr(&a));
        self.state.wifi.reduce(WifiMsg::Discovered(found));
    }

    /// Se connecte à l'adresse découverte (#28), avec reprise si demandée (#31).
    fn connect_discovered(&mut self) {
        self.state.wifi.reduce(WifiMsg::ConnectDiscoveredRequested);
        if !self.state.wifi.can_connect_discovered() {
            return;
        }
        let addr = match self.state.wifi.discovered.clone() {
            Some(a) => a,
            None => return,
        };
        let backoff = self.state.wifi.backoff();
        let res = reconnect::retry(backoff, || {
            Connection::TcpIp(addr.clone())
                .establish()
                .map(|_| addr.clone())
        })
        .map_err(|e| e.to_string());
        self.state.wifi.reduce(WifiMsg::ConnectResult(res));
    }

    /// Dessine l'écran « Appareils ». Renvoie les intentions via les sorties.
    fn ui_devices(
        &self,
        ui: &mut egui::Ui,
        want_refresh: &mut bool,
        want_connect: &mut bool,
        select: &mut Option<usize>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.state.is_loading(), egui::Button::new("🔄 Rafraîchir"))
                .clicked()
            {
                *want_refresh = true;
            }
            if ui
                .add_enabled(
                    self.state.selected.is_some(),
                    egui::Button::new("🔌 Connecter"),
                )
                .clicked()
            {
                *want_connect = true;
            }
        });

        ui.separator();

        if self.state.is_loading() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Recherche d'appareils…");
            });
        } else if let Load::Failed(e) = &self.state.load {
            ui.colored_label(egui::Color32::LIGHT_RED, format!("Erreur : {e}"));
        } else if self.state.devices.is_empty() {
            ui.label(
                "Aucun appareil. Branchez un appareil USB ou activez le « Débogage sans fil ».",
            );
        } else {
            egui::Grid::new("appareils")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.strong("Type");
                    ui.strong("Appareil");
                    ui.strong("État");
                    ui.end_row();

                    for (i, e) in self.state.devices.iter().enumerate() {
                        let selected = self.state.selected == Some(i);
                        if ui.selectable_label(selected, e.kind.as_str()).clicked() {
                            *select = Some(i);
                        }
                        if ui.selectable_label(selected, &e.id).clicked() {
                            *select = Some(i);
                        }
                        if ui.selectable_label(selected, &e.status).clicked() {
                            *select = Some(i);
                        }
                        ui.end_row();
                    }
                });
        }
    }

    /// Dessine l'écran « Appairage Wi-Fi ». Renvoie les intentions via sorties.
    fn ui_pairing(
        &mut self,
        ui: &mut egui::Ui,
        want_discover: &mut bool,
        want_pair: &mut bool,
        want_connect: &mut bool,
        want_reset: &mut bool,
    ) {
        ui.heading("Appairage Wi-Fi (Android 11+)");
        ui.label(
            "Sur l'appareil : Débogage sans fil › « Associer l'appareil à l'aide d'un code ». \
             L'appareil affiche un code à 6 chiffres et une adresse.",
        );
        ui.add_space(6.0);

        let p = &mut self.state.pairing;
        egui::Grid::new("formulaire_appairage")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Adresse d'appairage");
                ui.text_edit_singleline(&mut p.pair_addr);
                ui.end_row();
                ui.label("Code (6 chiffres)");
                ui.text_edit_singleline(&mut p.code);
                ui.end_row();
                ui.label("Adresse de connexion");
                ui.text_edit_singleline(&mut p.connect_addr);
                ui.end_row();
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("📡 Détecter (mDNS)").clicked() {
                *want_discover = true;
            }
            if ui
                .add_enabled(p.can_pair(), egui::Button::new("🔗 Appairer"))
                .clicked()
            {
                *want_pair = true;
            }
            if ui
                .add_enabled(p.can_connect(), egui::Button::new("🔌 Connecter"))
                .clicked()
            {
                *want_connect = true;
            }
            if ui.button("↺ Réinitialiser").clicked() {
                *want_reset = true;
            }
        });

        if !p.hint.is_empty() {
            ui.separator();
            let color = match p.phase {
                Phase::Failed(_) => egui::Color32::LIGHT_RED,
                Phase::Connected(_) | Phase::Paired => egui::Color32::LIGHT_GREEN,
                _ => ui.visuals().text_color(),
            };
            ui.colored_label(color, &p.hint);
        }
    }

    /// Dessine l'écran « Connexion Wi-Fi ». Renvoie les intentions via sorties.
    fn ui_wifi(
        &mut self,
        ui: &mut egui::Ui,
        want_switch: &mut bool,
        want_discover: &mut bool,
        want_connect: &mut bool,
        want_reset: &mut bool,
    ) {
        ui.heading("Connexion Wi-Fi");
        ui.label(
            "Passez d'une liaison USB au sans-fil sans saisir d'IP, ou connectez-vous \
             à un appareil annoncé sur le réseau local.",
        );
        ui.add_space(8.0);

        let w = &mut self.state.wifi;

        // --- Bascule USB → Wi-Fi (#27) ---
        ui.strong("Depuis un appareil USB");
        egui::Grid::new("formulaire_wifi")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Série de l'appareil USB");
                ui.text_edit_singleline(&mut w.serial);
                ui.end_row();
                ui.label("Port TCP/IP");
                ui.text_edit_singleline(&mut w.port);
                ui.end_row();
            });
        ui.add_space(4.0);
        if ui
            .add_enabled(w.can_switch(), egui::Button::new("📶 Basculer en Wi-Fi"))
            .clicked()
        {
            *want_switch = true;
        }

        ui.add_space(10.0);
        ui.separator();

        // --- Découverte mDNS (#28) ---
        ui.strong("Sur le réseau local (mDNS, Android 11+)");
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!w.is_busy(), egui::Button::new("📡 Détecter (mDNS)"))
                .clicked()
            {
                *want_discover = true;
            }
            match &w.discovered {
                Some(addr) => {
                    ui.label(format!("Trouvé : {addr}"));
                }
                None => {
                    ui.weak("aucun appareil détecté");
                }
            }
            if ui
                .add_enabled(
                    w.can_connect_discovered(),
                    egui::Button::new("🔌 Connecter"),
                )
                .clicked()
            {
                *want_connect = true;
            }
        });

        ui.add_space(10.0);
        ui.separator();

        // --- Reconnexion automatique (#31) ---
        ui.strong("Fiabilité");
        ui.horizontal(|ui| {
            ui.checkbox(&mut w.reconnect, "Reconnexion automatique");
            ui.add_enabled_ui(w.reconnect, |ui| {
                ui.label("tentatives :");
                ui.add(egui::TextEdit::singleline(&mut w.attempts).desired_width(48.0));
            });
        });
        ui.weak(
            "Si activée, la bascule et la connexion sont relancées avec un délai \
             croissant jusqu'à ce nombre de tentatives.",
        );

        ui.add_space(6.0);
        if ui.button("↺ Réinitialiser").clicked() {
            *want_reset = true;
        }

        if !w.hint.is_empty() {
            ui.separator();
            let color = match w.phase {
                wifi::Phase::Failed(_) => egui::Color32::LIGHT_RED,
                wifi::Phase::Connected(_) => egui::Color32::LIGHT_GREEN,
                _ => ui.visuals().text_color(),
            };
            ui.colored_label(color, &w.hint);
        }
    }

    /// Dessine l'écran « Options du miroir ». Renvoie les intentions via sorties.
    fn ui_options(&mut self, ui: &mut egui::Ui, want_reset: &mut bool) {
        ui.heading("Options du miroir");
        ui.label(
            "Réglages vidéo appliqués au prochain miroir. Les champs sont validés \
             en direct ; l'aperçu montre la commande équivalente.",
        );
        ui.add_space(8.0);

        // Marque de validité à poser après chaque champ.
        fn mark(ui: &mut egui::Ui, ok: bool) {
            if ok {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "✓");
            } else {
                ui.colored_label(egui::Color32::LIGHT_RED, "✗");
            }
        }

        let o = &mut self.state.options;
        egui::Grid::new("formulaire_options")
            .num_columns(3)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Résolution max (px, 0 = illimité)");
                ui.add(egui::TextEdit::singleline(&mut o.max_size).desired_width(90.0));
                mark(ui, o.max_size_valid());
                ui.end_row();

                ui.label("FPS max");
                ui.add(egui::TextEdit::singleline(&mut o.max_fps).desired_width(90.0));
                mark(ui, o.max_fps_valid());
                ui.end_row();

                ui.label("Codec vidéo");
                egui::ComboBox::from_id_salt("codec_combo")
                    .selected_text(o.codec.as_str())
                    .show_ui(ui, |ui| {
                        for c in VideoCodec::ALL {
                            ui.selectable_value(&mut o.codec, c, c.as_str());
                        }
                    });
                ui.label("");
                ui.end_row();

                ui.label("Débit (ex. 8M, 1500K ; vide = auto)");
                ui.add(egui::TextEdit::singleline(&mut o.bit_rate).desired_width(90.0));
                mark(ui, o.bit_rate_valid());
                ui.end_row();

                ui.label("Rognage (W:H:X:Y ; vide = aucun)");
                ui.add(egui::TextEdit::singleline(&mut o.crop).desired_width(140.0));
                mark(ui, o.crop_valid());
                ui.end_row();

                ui.label("Rotation");
                ui.horizontal(|ui| {
                    for r in ROTATIONS {
                        ui.selectable_value(&mut o.rotation, r, format!("{r}°"));
                    }
                });
                ui.label("");
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();
        ui.strong("Commande équivalente");
        ui.code(o.cli_preview());
        ui.add_space(4.0);
        ui.strong("Arguments transmis au serveur");
        ui.code(o.server_args().join("  "));

        ui.add_space(6.0);
        if o.is_valid() {
            ui.colored_label(egui::Color32::LIGHT_GREEN, "Réglages valides.");
        } else {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                "Un ou plusieurs champs sont invalides (voir ✗).",
            );
        }
        ui.weak("Le lancement du miroir depuis la fenêtre arrivera dans un prochain incrément.");

        ui.add_space(6.0);
        if ui.button("↺ Réinitialiser").clicked() {
            *want_reset = true;
        }
    }
}

impl eframe::App for GninouApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Intentions collectées pendant le rendu, appliquées après (évite les
        // conflits d'emprunt avec l'état lu dans les closures).
        let mut goto: Option<Screen> = None;
        let mut want_refresh = false;
        let mut want_connect = false;
        let mut select: Option<usize> = None;
        let mut want_discover = false;
        let mut want_pair = false;
        let mut want_connect_pair = false;
        let mut want_reset = false;
        let mut want_switch_wifi = false;
        let mut want_discover_wifi = false;
        let mut want_connect_wifi = false;
        let mut want_reset_wifi = false;
        let mut want_reset_options = false;

        egui::TopBottomPanel::top("entete").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("gninou");
                ui.separator();
                if ui
                    .selectable_label(self.state.screen == Screen::Devices, "Appareils")
                    .clicked()
                {
                    goto = Some(Screen::Devices);
                }
                if ui
                    .selectable_label(self.state.screen == Screen::Pairing, "Appairage Wi-Fi")
                    .clicked()
                {
                    goto = Some(Screen::Pairing);
                }
                if ui
                    .selectable_label(self.state.screen == Screen::Wifi, "Connexion Wi-Fi")
                    .clicked()
                {
                    goto = Some(Screen::Wifi);
                }
                if ui
                    .selectable_label(self.state.screen == Screen::Options, "Options")
                    .clicked()
                {
                    goto = Some(Screen::Options);
                }
            });
        });

        egui::TopBottomPanel::bottom("statut").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.state.status);
            });
        });

        let screen = self.state.screen.clone();
        egui::CentralPanel::default().show(ctx, |ui| match screen {
            Screen::Devices => {
                self.ui_devices(ui, &mut want_refresh, &mut want_connect, &mut select)
            }
            Screen::Pairing => self.ui_pairing(
                ui,
                &mut want_discover,
                &mut want_pair,
                &mut want_connect_pair,
                &mut want_reset,
            ),
            Screen::Wifi => self.ui_wifi(
                ui,
                &mut want_switch_wifi,
                &mut want_discover_wifi,
                &mut want_connect_wifi,
                &mut want_reset_wifi,
            ),
            Screen::Options => self.ui_options(ui, &mut want_reset_options),
        });

        // Application des intentions (I/O autorisée ici : plus d'emprunt actif).
        if let Some(s) = goto {
            let entering_wifi = s == Screen::Wifi;
            self.state.reduce(Msg::GoTo(s));
            if entering_wifi {
                self.prefill_wifi_serial();
            }
        }
        if let Some(i) = select {
            self.state.reduce(Msg::Selected(i));
        }
        if want_refresh {
            self.refresh();
        }
        if want_connect {
            self.connect_selected();
        }
        if want_discover {
            self.discover_pairing();
        }
        if want_pair {
            self.do_pair();
        }
        if want_connect_pair {
            self.do_connect_pairing();
        }
        if want_reset {
            self.state.pairing.reduce(PairMsg::Reset);
        }
        if want_switch_wifi {
            self.switch_wifi();
        }
        if want_discover_wifi {
            self.discover_wifi();
        }
        if want_connect_wifi {
            self.connect_discovered();
        }
        if want_reset_wifi {
            self.state.wifi.reduce(WifiMsg::Reset);
        }
        if want_reset_options {
            self.state.options.reduce(super::options::OptMsg::Reset);
        }
    }
}

/// Lance la fenêtre GUI (bloquant jusqu'à sa fermeture).
pub fn launch() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("gninou")
            .with_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "gninou",
        options,
        Box::new(|_cc| Ok(Box::new(GninouApp::new()))),
    )
    .map_err(|e| anyhow::anyhow!("échec du lancement de l'interface graphique : {e}"))
}
