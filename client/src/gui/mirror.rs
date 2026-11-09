//! View-model **pur** de l'onglet « Miroir » (#36).
//!
//! Gère l'affichage du flux vidéo dans la fenêtre : mode d'ajustement/zoom
//! (redimensionnement), capture d'écran (PNG) et enregistrement (suite d'images).
//! Il ne fait aucune I/O ni rendu : il tient l'état, calcule la taille d'affichage
//! et nomme les fichiers ; le rendu egui (`super::app`) lit cet état, dessine la
//! [`Frame`](crate::media::frame::Frame) et déclenche les écritures PNG. Pur et
//! testé dans le build par défaut.

/// Zoom minimal/maximal et pas, en pourcentage.
pub const ZOOM_MIN: u32 = 25;
pub const ZOOM_MAX: u32 = 400;
pub const ZOOM_STEP: u32 = 25;

/// Façon d'adapter l'image à la zone d'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitMode {
    /// Ajuste à la fenêtre en gardant les proportions.
    Fit,
    /// Taille réelle (1 pixel image = 1 pixel écran).
    Actual,
    /// Échelle explicite en pourcentage.
    Zoom(u32),
}

/// Messages émis par l'UI ou l'I/O.
#[derive(Debug, Clone)]
pub enum MirrorMsg {
    FitSelected(FitMode),
    ZoomIn,
    ZoomOut,
    RunToggled(bool),
    ShotSaved(String),
    ShotFailed(String),
    RecordStarted,
    RecordStopped,
    FrameRecorded,
    FrameInfo { width: u32, height: u32 },
}

/// État de l'onglet Miroir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirrorState {
    pub fit: FitMode,
    /// La source (mire/flux) tourne-t-elle ?
    pub running: bool,
    pub recording: bool,
    /// Images écrites dans l'enregistrement en cours.
    pub frames_recorded: u32,
    /// Nombre de captures déjà prises (sert à nommer les fichiers).
    pub shots_taken: u32,
    /// Dernière action, affichée en bas de l'onglet.
    pub last_action: String,
    /// Dimensions de l'image courante.
    pub frame_w: u32,
    pub frame_h: u32,
}

impl Default for MirrorState {
    fn default() -> Self {
        MirrorState {
            fit: FitMode::Fit,
            running: true,
            recording: false,
            frames_recorded: 0,
            shots_taken: 0,
            last_action: "Mire de test — en attente d'un flux d'appareil.".to_string(),
            frame_w: 0,
            frame_h: 0,
        }
    }
}

impl MirrorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pourcentage de zoom nominal (100 % pour Fit/Actual).
    pub fn zoom_percent(&self) -> u32 {
        match self.fit {
            FitMode::Zoom(p) => p,
            _ => 100,
        }
    }

    /// Libellé court du mode d'ajustement.
    pub fn fit_label(&self) -> String {
        match self.fit {
            FitMode::Fit => "Ajusté".to_string(),
            FitMode::Actual => "100 %".to_string(),
            FitMode::Zoom(_) => format!("{} %", self.zoom_percent()),
        }
    }

    /// Taille d'affichage (largeur, hauteur) en pixels écran pour la zone
    /// disponible donnée, selon le mode courant. `(0, 0)` si l'image est vide.
    /// **Pur.**
    pub fn display_size(&self, avail_w: f32, avail_h: f32) -> (f32, f32) {
        let fw = self.frame_w as f32;
        let fh = self.frame_h as f32;
        if fw <= 0.0 || fh <= 0.0 {
            return (0.0, 0.0);
        }
        match self.fit {
            FitMode::Fit => {
                let scale = (avail_w / fw).min(avail_h / fh).max(0.0);
                (fw * scale, fh * scale)
            }
            FitMode::Actual => (fw, fh),
            FitMode::Zoom(p) => {
                let s = p as f32 / 100.0;
                (fw * s, fh * s)
            }
        }
    }

    /// Nom du prochain fichier de capture (incrémente le compteur). Ex. :
    /// `gninou-capture-001.png`.
    pub fn next_shot_name(&mut self) -> String {
        self.shots_taken += 1;
        format!("gninou-capture-{:03}.png", self.shots_taken)
    }

    /// Nom d'une image dans un enregistrement. Ex. : `image-00042.png`.
    pub fn frame_name(seq: u32) -> String {
        format!("image-{seq:05}.png")
    }

    /// Base interne du zoom pour un cran : part du zoom courant, ou de 100 %.
    fn zoom_base(&self) -> u32 {
        match self.fit {
            FitMode::Zoom(p) => p,
            _ => 100,
        }
    }

    /// Applique un message et fait évoluer l'état. **Pur et testable.**
    pub fn reduce(&mut self, msg: MirrorMsg) {
        match msg {
            MirrorMsg::FitSelected(f) => self.fit = f,
            MirrorMsg::ZoomIn => {
                let p = (self.zoom_base() + ZOOM_STEP).min(ZOOM_MAX);
                self.fit = FitMode::Zoom(p);
            }
            MirrorMsg::ZoomOut => {
                let p = self.zoom_base().saturating_sub(ZOOM_STEP).max(ZOOM_MIN);
                self.fit = FitMode::Zoom(p);
            }
            MirrorMsg::RunToggled(b) => self.running = b,
            MirrorMsg::ShotSaved(name) => {
                self.last_action = format!("Capture enregistrée : {name}");
            }
            MirrorMsg::ShotFailed(e) => {
                self.last_action = format!("Échec de la capture : {e}");
            }
            MirrorMsg::RecordStarted => {
                self.recording = true;
                self.frames_recorded = 0;
                self.last_action = "Enregistrement démarré…".to_string();
            }
            MirrorMsg::RecordStopped => {
                self.recording = false;
                self.last_action =
                    format!("Enregistrement arrêté ({} images).", self.frames_recorded);
            }
            MirrorMsg::FrameRecorded => {
                if self.recording {
                    self.frames_recorded += 1;
                }
            }
            MirrorMsg::FrameInfo { width, height } => {
                self.frame_w = width;
                self.frame_h = height;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_frame(w: u32, h: u32) -> MirrorState {
        let mut s = MirrorState::new();
        s.reduce(MirrorMsg::FrameInfo {
            width: w,
            height: h,
        });
        s
    }

    #[test]
    fn etat_initial() {
        let s = MirrorState::new();
        assert_eq!(s.fit, FitMode::Fit);
        assert!(s.running);
        assert!(!s.recording);
        assert_eq!(s.zoom_percent(), 100);
    }

    #[test]
    fn fit_ajuste_en_gardant_les_proportions() {
        let s = with_frame(200, 100);
        // Zone 100x100 : limité par la largeur -> échelle 0.5.
        assert_eq!(s.display_size(100.0, 100.0), (100.0, 50.0));
    }

    #[test]
    fn taille_reelle() {
        let mut s = with_frame(320, 180);
        s.reduce(MirrorMsg::FitSelected(FitMode::Actual));
        assert_eq!(s.display_size(1000.0, 1000.0), (320.0, 180.0));
    }

    #[test]
    fn zoom_explicite() {
        let mut s = with_frame(100, 50);
        s.reduce(MirrorMsg::FitSelected(FitMode::Zoom(200)));
        assert_eq!(s.display_size(9999.0, 9999.0), (200.0, 100.0));
        assert_eq!(s.zoom_percent(), 200);
    }

    #[test]
    fn image_vide_taille_nulle() {
        let s = MirrorState::new(); // frame 0x0
        assert_eq!(s.display_size(500.0, 500.0), (0.0, 0.0));
    }

    #[test]
    fn zoom_in_depuis_fit_puis_plafonne() {
        let mut s = with_frame(10, 10);
        s.reduce(MirrorMsg::ZoomIn); // 100 -> 125
        assert_eq!(s.fit, FitMode::Zoom(125));
        for _ in 0..50 {
            s.reduce(MirrorMsg::ZoomIn);
        }
        assert_eq!(s.fit, FitMode::Zoom(ZOOM_MAX));
    }

    #[test]
    fn zoom_out_plancher() {
        let mut s = with_frame(10, 10);
        for _ in 0..50 {
            s.reduce(MirrorMsg::ZoomOut);
        }
        assert_eq!(s.fit, FitMode::Zoom(ZOOM_MIN));
    }

    #[test]
    fn marche_arret_source() {
        let mut s = MirrorState::new();
        s.reduce(MirrorMsg::RunToggled(false));
        assert!(!s.running);
    }

    #[test]
    fn capture_nomme_et_annonce() {
        let mut s = MirrorState::new();
        assert_eq!(s.next_shot_name(), "gninou-capture-001.png");
        assert_eq!(s.next_shot_name(), "gninou-capture-002.png");
        s.reduce(MirrorMsg::ShotSaved("gninou-capture-002.png".into()));
        assert!(s.last_action.contains("gninou-capture-002.png"));
    }

    #[test]
    fn nom_image_enregistrement() {
        assert_eq!(MirrorState::frame_name(42), "image-00042.png");
    }

    #[test]
    fn enregistrement_compte_les_images() {
        let mut s = MirrorState::new();
        // Hors enregistrement, les images ne comptent pas.
        s.reduce(MirrorMsg::FrameRecorded);
        assert_eq!(s.frames_recorded, 0);
        s.reduce(MirrorMsg::RecordStarted);
        assert!(s.recording);
        s.reduce(MirrorMsg::FrameRecorded);
        s.reduce(MirrorMsg::FrameRecorded);
        assert_eq!(s.frames_recorded, 2);
        s.reduce(MirrorMsg::RecordStopped);
        assert!(!s.recording);
        assert!(s.last_action.contains('2'));
    }

    #[test]
    fn info_image_met_a_jour_les_dimensions() {
        let s = with_frame(1080, 2400);
        assert_eq!((s.frame_w, s.frame_h), (1080, 2400));
        assert_eq!(s.fit_label(), "Ajusté");
    }
}
