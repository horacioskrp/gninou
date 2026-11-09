//! Rendu egui/eframe de l'interface graphique (#32–#36).
//!
//! Fine couche de **vue** au-dessus des view-models purs ([`super::state`],
//! [`super::pairing`], [`super::wifi`], [`super::options`], [`super::mirror`]) :
//! elle lit l'état, dessine l'écran courant et traduit les clics en messages.
//! La collecte (`adb devices` + mDNS), l'appairage/connexion et les écritures
//! PNG (captures/enregistrement) sont faites de façon synchrone pour ces
//! premiers incréments (🟨 : passage en tâche de fond à prévoir).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use eframe::egui;

use super::mirror::{FitMode, MirrorMsg, MirrorState};
use super::options::ROTATIONS;
use super::pairing::{PairMsg, Phase};
use super::state::{AppState, Load, Msg, Screen};
use super::wifi::{self, WifiMsg};
use crate::device::connection::{self, Connection};
use crate::device::listing::{self, Kind};
use crate::device::{mdns, reconnect, screencap};
use crate::media::codec::VideoCodec;
use crate::media::frame::{self, Frame};

/// Résolution de la mire de test affichée dans l'onglet Miroir.
const MIRROR_W: u32 = 480;
const MIRROR_H: u32 = 270;
/// Enregistre une image sur ce nombre de ticks (throttle disque).
const RECORD_EVERY: u64 = 6;
/// Pause entre deux captures d'écran adb (throttle réseau/CPU).
const CAPTURE_INTERVAL: Duration = Duration::from_millis(80);
/// Pause après un échec de capture avant de réessayer.
const CAPTURE_RETRY: Duration = Duration::from_millis(600);

/// Flux de captures d'écran d'un appareil, alimenté par un thread dédié
/// (`adb exec-out screencap`). Le thread s'arrête à la destruction (#38).
struct CaptureFeed {
    latest: Arc<Mutex<Option<Frame>>>,
    error: Arc<Mutex<Option<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl CaptureFeed {
    fn start(serial: String) -> Self {
        let latest = Arc::new(Mutex::new(None));
        let error = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let (l, e, s) = (latest.clone(), error.clone(), stop.clone());
        let handle = std::thread::spawn(move || {
            while !s.load(Ordering::Relaxed) {
                match screencap::capture(&serial) {
                    Ok(frame) => {
                        if let Ok(mut g) = l.lock() {
                            *g = Some(frame);
                        }
                        if let Ok(mut g) = e.lock() {
                            *g = None;
                        }
                        std::thread::sleep(CAPTURE_INTERVAL);
                    }
                    Err(err) => {
                        if let Ok(mut g) = e.lock() {
                            *g = Some(err.to_string());
                        }
                        std::thread::sleep(CAPTURE_RETRY);
                    }
                }
            }
        });
        CaptureFeed {
            latest,
            error,
            stop,
            handle: Some(handle),
        }
    }

    /// Récupère et consomme la dernière image disponible, si une est arrivée.
    fn take(&self) -> Option<Frame> {
        self.latest.lock().ok().and_then(|mut g| g.take())
    }

    /// Dernier message d'erreur de capture, s'il y en a un.
    fn error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|g| g.clone())
    }
}

impl Drop for CaptureFeed {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// État d'exécution d'une fenêtre de miroir : flux de capture, dernière image,
/// texture GPU et dossier d'enregistrement.
struct WindowRuntime {
    feed: CaptureFeed,
    frame: Frame,
    texture: Option<egui::TextureHandle>,
    has_frame: bool,
    record_dir: Option<PathBuf>,
}

impl WindowRuntime {
    fn start(serial: &str) -> Self {
        WindowRuntime {
            feed: CaptureFeed::start(serial.to_string()),
            frame: Frame::black(1, 1),
            texture: None,
            has_frame: false,
            record_dir: None,
        }
    }
}

/// Application GUI gninou.
pub struct GninouApp {
    state: AppState,
    /// Compteur d'animation de la mire de test.
    tick: u64,
    /// Image courante affichée dans l'onglet Miroir.
    frame: Frame,
    /// Texture GPU de l'image courante (recréée à chaque image).
    texture: Option<egui::TextureHandle>,
    /// Dossier de l'enregistrement en cours de l'onglet Miroir, s'il y en a un.
    record_dir: Option<PathBuf>,
    /// Exécution par fenêtre dédiée (capture réelle), clé = id d'appareil.
    runtimes: HashMap<String, WindowRuntime>,
    /// Plein écran demandé (onglet) ?
    fullscreen: bool,
}

impl GninouApp {
    fn new() -> Self {
        let mut app = GninouApp {
            state: AppState::new(),
            tick: 0,
            frame: frame::test_pattern(MIRROR_W, MIRROR_H, 0),
            texture: None,
            record_dir: None,
            runtimes: HashMap::new(),
            fullscreen: false,
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
        open_mirror: &mut Option<String>,
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
            let n = self.state.devices.len();
            ui.separator();
            ui.weak(match n {
                0 => "aucun appareil".to_string(),
                1 => "1 appareil".to_string(),
                _ => format!("{n} appareils"),
            });
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
                .num_columns(4)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Type");
                    ui.strong("Appareil");
                    ui.strong("État");
                    ui.strong("");
                    ui.end_row();

                    for (i, e) in self.state.devices.iter().enumerate() {
                        let selected = self.state.selected == Some(i);
                        if ui
                            .selectable_label(
                                selected,
                                format!("{} {}", e.kind.icon(), e.kind.as_str()),
                            )
                            .clicked()
                        {
                            *select = Some(i);
                        }
                        if ui
                            .selectable_label(selected, egui::RichText::new(e.id.as_str()).monospace())
                            .clicked()
                        {
                            *select = Some(i);
                        }
                        if ui
                            .selectable_label(
                                selected,
                                format!("{} {}", e.status_icon(), e.status),
                            )
                            .clicked()
                        {
                            *select = Some(i);
                        }

                        // Bouton « Miroir » : ouvre l'appareil dans sa fenêtre.
                        let ready = e.is_ready();
                        let already = self.state.windows.is_open(&e.id);
                        let label = if already { "🖥 Ouvert" } else { "🖥 Miroir" };
                        let btn = ui.add_enabled(ready && !already, egui::Button::new(label));
                        let btn = if !ready {
                            btn.on_hover_text(
                                "Connectez d'abord l'appareil (état « prêt ») pour ouvrir le miroir.",
                            )
                        } else {
                            btn
                        };
                        if btn.clicked() {
                            *open_mirror = Some(e.id.clone());
                        }
                        ui.end_row();
                    }
                });
            ui.add_space(6.0);
            ui.weak(
                "Astuce : « 🖥 Miroir » sur un appareil prêt l'ouvre dans sa propre fenêtre \
                 (miroir + contrôle), comme un gestionnaire d'appareils.",
            );
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

    // --- Onglet Miroir (#36) ---

    /// Dossier des captures (`<cwd>/gninou-captures`), créé au besoin.
    fn captures_dir() -> anyhow::Result<PathBuf> {
        let dir = std::env::current_dir()?.join("gninou-captures");
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Écrit une image RGBA dans un fichier PNG.
    fn write_png(path: &Path, frame: &Frame) -> anyhow::Result<()> {
        let file = std::fs::File::create(path)?;
        let w = std::io::BufWriter::new(file);
        let mut enc = png::Encoder::new(w, frame.width, frame.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&frame.rgba)?;
        Ok(())
    }

    /// Fait avancer la mire de l'**onglet** Miroir (#36 : aperçu sans appareil)
    /// et met à jour sa texture. Les fenêtres dédiées (#38) affichent, elles,
    /// l'écran réel via [`Self::pump_windows`].
    fn advance_mirror(&mut self, ctx: &egui::Context) {
        if self.state.mirror.running {
            self.tick = self.tick.wrapping_add(2);
        }
        if self.state.mirror.frame_w != MIRROR_W {
            self.state.mirror.reduce(MirrorMsg::FrameInfo {
                width: MIRROR_W,
                height: MIRROR_H,
            });
        }
        self.frame = frame::test_pattern(MIRROR_W, MIRROR_H, self.tick);
        if !self.frame.is_valid() {
            return;
        }
        if self.state.mirror.recording && self.tick % RECORD_EVERY == 0 {
            Self::record_frame_into(&mut self.state.mirror, &self.record_dir, &self.frame);
        }

        let img = egui::ColorImage::from_rgba_unmultiplied(
            [MIRROR_W as usize, MIRROR_H as usize],
            &self.frame.rgba,
        );
        match &mut self.texture {
            Some(t) => t.set(img, egui::TextureOptions::NEAREST),
            None => {
                self.texture = Some(ctx.load_texture("miroir", img, egui::TextureOptions::NEAREST))
            }
        }
    }

    /// Draine les captures d'écran reçues des fenêtres dédiées : met à jour la
    /// texture et l'image courante de chaque fenêtre, et enregistre si demandé.
    fn pump_windows(&mut self, ctx: &egui::Context) {
        for id in self.state.windows.ids() {
            let rt = match self.runtimes.get_mut(&id) {
                Some(rt) => rt,
                None => continue,
            };
            let frame = match rt.feed.take() {
                Some(f) => f,
                None => continue,
            };

            // Publie les dimensions réelles à l'état d'affichage de la fenêtre.
            if let Some(win) = self.state.windows.get_mut(&id) {
                if win.mirror.frame_w != frame.width || win.mirror.frame_h != frame.height {
                    win.mirror.reduce(MirrorMsg::FrameInfo {
                        width: frame.width,
                        height: frame.height,
                    });
                }
                if win.mirror.recording {
                    Self::record_frame_into(&mut win.mirror, &rt.record_dir, &frame);
                }
            }

            let img = egui::ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.rgba,
            );
            match &mut rt.texture {
                Some(t) => t.set(img, egui::TextureOptions::LINEAR),
                None => {
                    rt.texture = Some(ctx.load_texture(
                        format!("mir-{id}"),
                        img,
                        egui::TextureOptions::LINEAR,
                    ))
                }
            }
            rt.frame = frame;
            rt.has_frame = true;
        }
    }

    /// Écrit l'image courante dans le dossier d'enregistrement d'un état miroir.
    fn record_frame_into(mirror: &mut MirrorState, dir: &Option<PathBuf>, frame: &Frame) {
        let dir = match dir {
            Some(d) => d,
            None => return,
        };
        let path = dir.join(MirrorState::frame_name(mirror.frames_recorded));
        if Self::write_png(&path, frame).is_ok() {
            mirror.reduce(MirrorMsg::FrameRecorded);
        }
    }

    /// Capture l'image courante dans un PNG, nommée via l'état donné.
    fn save_screenshot(mirror: &mut MirrorState, frame: &Frame) {
        let name = mirror.next_shot_name();
        let res = Self::captures_dir().and_then(|d| {
            let p = d.join(&name);
            Self::write_png(&p, frame).map(|_| p.display().to_string())
        });
        match res {
            Ok(path) => mirror.reduce(MirrorMsg::ShotSaved(path)),
            Err(e) => mirror.reduce(MirrorMsg::ShotFailed(e.to_string())),
        }
    }

    /// Démarre/arrête l'enregistrement d'images pour un état miroir donné.
    fn toggle_recording(mirror: &mut MirrorState, dir: &mut Option<PathBuf>) {
        if mirror.recording {
            mirror.reduce(MirrorMsg::RecordStopped);
            *dir = None;
            return;
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let created = Self::captures_dir()
            .map(|d| d.join(format!("rec-{ts}")))
            .and_then(|d| std::fs::create_dir_all(&d).map(|_| d).map_err(Into::into));
        match created {
            Ok(d) => {
                *dir = Some(d);
                mirror.reduce(MirrorMsg::RecordStarted);
            }
            Err(e) => mirror.reduce(MirrorMsg::ShotFailed(e.to_string())),
        }
    }

    /// Barre d'outils + image du miroir. Partagé par l'onglet (#36) et les
    /// fenêtres dédiées (#37). Renvoie les intentions via les sorties.
    fn draw_mirror_canvas(
        ui: &mut egui::Ui,
        mirror: &mut MirrorState,
        texture: Option<&egui::TextureHandle>,
        want_shot: &mut bool,
        want_record: &mut bool,
        want_fullscreen: &mut bool,
        msg: &mut Option<MirrorMsg>,
    ) {
        ui.horizontal_wrapped(|ui| {
            // Ajustement / zoom (redimensionnement).
            if ui
                .selectable_label(mirror.fit == FitMode::Fit, "Ajuster")
                .clicked()
            {
                *msg = Some(MirrorMsg::FitSelected(FitMode::Fit));
            }
            if ui
                .selectable_label(mirror.fit == FitMode::Actual, "100 %")
                .clicked()
            {
                *msg = Some(MirrorMsg::FitSelected(FitMode::Actual));
            }
            if ui.button("➖").clicked() {
                *msg = Some(MirrorMsg::ZoomOut);
            }
            ui.label(mirror.fit_label());
            if ui.button("➕").clicked() {
                *msg = Some(MirrorMsg::ZoomIn);
            }

            ui.separator();

            let run_label = if mirror.running {
                "⏸ Pause"
            } else {
                "▶ Lecture"
            };
            if ui.button(run_label).clicked() {
                *msg = Some(MirrorMsg::RunToggled(!mirror.running));
            }

            ui.separator();

            if ui.button("📸 Capturer").clicked() {
                *want_shot = true;
            }
            let rec_label = if mirror.recording {
                "⏹ Arrêter"
            } else {
                "⏺ Enregistrer"
            };
            if ui.button(rec_label).clicked() {
                *want_record = true;
            }
            if mirror.recording {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    format!("● {} images", mirror.frames_recorded),
                );
            }

            ui.separator();
            if ui.button("⛶ Plein écran").clicked() {
                *want_fullscreen = true;
            }
        });

        ui.separator();

        let avail = ui.available_size();
        let (w, h) = mirror.display_size(avail.x, (avail.y - 4.0).max(1.0));
        if let Some(tex) = texture {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                    tex.id(),
                    egui::vec2(w.max(1.0), h.max(1.0)),
                )));
            });
        } else {
            ui.label("Initialisation de l'affichage…");
        }
    }

    /// Panneau de contrôle (à brancher) affiché sous le miroir d'une fenêtre.
    fn draw_control_placeholder(ui: &mut egui::Ui, device_id: &str) {
        ui.strong("Contrôle");
        ui.weak(format!(
            "Clavier, souris et HID pour {device_id} — câblage à venir (#8–#10). \
             La capture des entrées et leur injection sur l'appareil ne sont pas \
             encore actives."
        ));
    }

    /// Dessine l'onglet « Miroir » (#36). Renvoie les intentions via sorties.
    fn ui_mirror(
        &mut self,
        ui: &mut egui::Ui,
        want_shot: &mut bool,
        want_record: &mut bool,
        want_fullscreen: &mut bool,
        msg: &mut Option<MirrorMsg>,
    ) {
        Self::draw_mirror_canvas(
            ui,
            &mut self.state.mirror,
            self.texture.as_ref(),
            want_shot,
            want_record,
            want_fullscreen,
            msg,
        );
        ui.separator();
        ui.weak(format!(
            "Source : mire de test {MIRROR_W}×{MIRROR_H}. Ouvrez un appareil « prêt » \
             depuis l'onglet Appareils pour l'afficher dans sa propre fenêtre."
        ));
        ui.label(&self.state.mirror.last_action);
    }

    /// Ouvre/rafraîchit une fenêtre dédiée par appareil (viewports immédiats),
    /// affichant l'écran **réel** capturé par `adb screencap` (#38).
    fn render_mirror_windows(&mut self, ctx: &egui::Context) {
        let ids = self.state.windows.ids();
        if ids.is_empty() {
            return;
        }
        let mut to_close: Vec<String> = Vec::new();

        for id in ids {
            // Texture et état de capture de CETTE fenêtre (clonés pour ne pas
            // emprunter self.runtimes dans la closure du viewport).
            let (tex, has_frame, cap_err) = match self.runtimes.get(&id) {
                Some(rt) => (rt.texture.clone(), rt.has_frame, rt.feed.error()),
                None => (None, false, None),
            };

            let vid = egui::ViewportId::from_hash_of(("gninou-mirror", id.as_str()));
            let builder = egui::ViewportBuilder::default()
                .with_title(format!("gninou — miroir : {id}"))
                .with_inner_size([420.0, 780.0]);

            let mut shot = false;
            let mut record = false;
            let mut fullscreen = false;
            let mut msg: Option<MirrorMsg> = None;
            let mut close = false;

            ctx.show_viewport_immediate(vid, builder, |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(win) = self.state.windows.get_mut(&id) {
                        ui.horizontal(|ui| {
                            ui.heading("Miroir");
                            ui.separator();
                            ui.monospace(win.device_id.as_str());
                        });
                        ui.separator();
                        if has_frame {
                            Self::draw_mirror_canvas(
                                ui,
                                &mut win.mirror,
                                tex.as_ref(),
                                &mut shot,
                                &mut record,
                                &mut fullscreen,
                                &mut msg,
                            );
                        } else {
                            ui.add_space(24.0);
                            ui.vertical_centered(|ui| {
                                ui.spinner();
                                match &cap_err {
                                    Some(e) => ui.colored_label(
                                        egui::Color32::LIGHT_RED,
                                        format!("Capture impossible : {e}"),
                                    ),
                                    None => ui.label("Capture de l'écran en cours…"),
                                };
                            });
                        }
                        ui.separator();
                        Self::draw_control_placeholder(ui, &id);
                        ui.separator();
                        ui.label(&win.mirror.last_action);
                    }
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    close = true;
                }
                if fullscreen {
                    let now = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!now));
                }
            });

            // Application des intentions de cette fenêtre.
            if let Some(m) = msg {
                if let Some(win) = self.state.windows.get_mut(&id) {
                    win.mirror.reduce(m);
                }
            }
            if shot {
                if let (Some(win), Some(rt)) =
                    (self.state.windows.get_mut(&id), self.runtimes.get(&id))
                {
                    Self::save_screenshot(&mut win.mirror, &rt.frame);
                }
            }
            if record {
                if let (Some(win), Some(rt)) =
                    (self.state.windows.get_mut(&id), self.runtimes.get_mut(&id))
                {
                    Self::toggle_recording(&mut win.mirror, &mut rt.record_dir);
                }
            }
            if close {
                to_close.push(id);
            }
        }

        for id in to_close {
            self.state.windows.close(&id);
            self.runtimes.remove(&id); // arrête le thread de capture (Drop)
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
        let mut want_shot = false;
        let mut want_record = false;
        let mut want_fullscreen = false;
        let mut mirror_msg: Option<MirrorMsg> = None;
        let mut open_mirror: Option<String> = None;

        // L'onglet Miroir anime la mire d'aperçu (#36).
        if self.state.screen == Screen::Mirror {
            self.advance_mirror(ctx);
            if self.state.mirror.running {
                ctx.request_repaint();
            }
        }
        // Les fenêtres dédiées affichent l'écran réel (#38) : on draine les
        // captures reçues des threads et on redemande un rendu (asynchrone).
        if self.state.windows.count() > 0 {
            self.pump_windows(ctx);
            ctx.request_repaint_after(Duration::from_millis(50));
        }

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
                if ui
                    .selectable_label(self.state.screen == Screen::Mirror, "Miroir")
                    .clicked()
                {
                    goto = Some(Screen::Mirror);
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
            Screen::Devices => self.ui_devices(
                ui,
                &mut want_refresh,
                &mut want_connect,
                &mut select,
                &mut open_mirror,
            ),
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
            Screen::Mirror => self.ui_mirror(
                ui,
                &mut want_shot,
                &mut want_record,
                &mut want_fullscreen,
                &mut mirror_msg,
            ),
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
        if let Some(m) = mirror_msg {
            self.state.mirror.reduce(m);
        }
        if want_shot {
            Self::save_screenshot(&mut self.state.mirror, &self.frame);
        }
        if want_record {
            Self::toggle_recording(&mut self.state.mirror, &mut self.record_dir);
        }
        if want_fullscreen {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
        }
        if let Some(id) = open_mirror {
            // Ouvre la fenêtre et lance le thread de capture de l'appareil.
            if self.state.windows.open(&id) {
                self.runtimes.insert(id.clone(), WindowRuntime::start(&id));
            }
        }

        // Fenêtres de miroir dédiées (une par appareil ouvert).
        self.render_mirror_windows(ctx);
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
