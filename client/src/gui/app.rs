//! Rendu egui/eframe de l'interface graphique (#32).
//!
//! Fine couche de **vue** au-dessus du view-model pur [`super::state`] : elle
//! lit l'état, dessine l'écran « Appareils » et traduit les clics en
//! [`Msg`](super::state::Msg). La collecte (`adb devices` + mDNS) et la
//! connexion (`adb connect`) sont faites de façon synchrone pour ce premier
//! incrément (🟨 : passage en tâche de fond à prévoir).

use eframe::egui;

use super::state::{AppState, Load, Msg};
use crate::device::connection::Connection;
use crate::device::listing::{self, Kind};

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

    /// Connecte l'appareil sélectionné (réseau uniquement pour l'instant).
    fn connect_selected(&mut self) {
        self.state.reduce(Msg::ConnectRequested);
        let entry = match self.state.selected_entry() {
            Some(e) => e.clone(),
            None => return, // le reduce a déjà posé le message d'invite
        };

        // Seules les cibles réseau sont « connectables » ici (adb connect).
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
}

impl eframe::App for GninouApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Intentions collectées pendant le rendu, appliquées après (évite les
        // conflits d'emprunt avec l'état lu dans la closure).
        let mut want_refresh = false;
        let mut want_connect = false;
        let mut select: Option<usize> = None;

        egui::TopBottomPanel::top("entete").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("gninou");
                ui.label("— Appareils");
            });
        });

        egui::TopBottomPanel::bottom("statut").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.state.status);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.state.is_loading(), egui::Button::new("🔄 Rafraîchir"))
                    .clicked()
                {
                    want_refresh = true;
                }
                if ui
                    .add_enabled(
                        self.state.selected.is_some(),
                        egui::Button::new("🔌 Connecter"),
                    )
                    .clicked()
                {
                    want_connect = true;
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
                                select = Some(i);
                            }
                            if ui.selectable_label(selected, &e.id).clicked() {
                                select = Some(i);
                            }
                            if ui.selectable_label(selected, &e.status).clicked() {
                                select = Some(i);
                            }
                            ui.end_row();
                        }
                    });
            }
        });

        if let Some(i) = select {
            self.state.reduce(Msg::Selected(i));
        }
        if want_refresh {
            self.refresh();
        }
        if want_connect {
            self.connect_selected();
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
