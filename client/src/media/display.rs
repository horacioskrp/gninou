//! Affichage vidéo.
//!
//! Le rendu réel s'appuie sur SDL3 et n'est compilé qu'avec la feature
//! `display` (sinon le build reste léger, sans dépendance native). La logique
//! de **cadrage** (letterbox en conservant le ratio) est pure et testée.

/// Rectangle de destination `(x, y, largeur, hauteur)` en pixels.
pub type Rect = (i32, i32, u32, u32);

/// Calcule le rectangle où dessiner une frame `frame_w x frame_h` dans une
/// fenêtre `win_w x win_h`, en conservant le ratio et en centrant (letterbox).
///
/// Logique **pure et testable**.
pub fn fit_rect(frame_w: u32, frame_h: u32, win_w: u32, win_h: u32) -> Rect {
    if frame_w == 0 || frame_h == 0 || win_w == 0 || win_h == 0 {
        return (0, 0, win_w, win_h);
    }
    let scale = f64::min(win_w as f64 / frame_w as f64, win_h as f64 / frame_h as f64);
    let w = (frame_w as f64 * scale).round() as u32;
    let h = (frame_h as f64 * scale).round() as u32;
    let x = ((win_w - w) / 2) as i32;
    let y = ((win_h - h) / 2) as i32;
    (x, y, w, h)
}

/// Fenêtre d'affichage (API stable, indépendante du backend).
pub struct Display {
    #[cfg(feature = "display")]
    backend: sdl_backend::SdlDisplay,
    #[cfg(not(feature = "display"))]
    _private: (),
}

impl Display {
    /// Crée la fenêtre aux dimensions initiales données.
    pub fn new(_title: &str, _width: u32, _height: u32) -> anyhow::Result<Self> {
        Ok(Self {
            #[cfg(feature = "display")]
            backend: sdl_backend::SdlDisplay::new(_title, _width, _height)?,
            #[cfg(not(feature = "display"))]
            _private: (),
        })
    }

    /// Présente une frame décodée (données I420/YUV + dimensions).
    pub fn present(&mut self, _frame: &[u8], _w: u32, _h: u32) -> anyhow::Result<()> {
        #[cfg(feature = "display")]
        self.backend.present(_frame, _w, _h)?;
        Ok(())
    }
}

#[cfg(feature = "display")]
mod sdl_backend {
    use anyhow::Result;

    /// Implémentation SDL3 (fenêtre + canvas + texture YUV).
    pub struct SdlDisplay {
        // TODO: sdl3::video::Window, Canvas, Texture (format IYUV), EventPump.
    }

    impl SdlDisplay {
        pub fn new(_title: &str, _width: u32, _height: u32) -> Result<Self> {
            // TODO: sdl3::init(), video_subsystem, window, canvas, texture_creator.
            Ok(Self {})
        }

        pub fn present(&mut self, _frame: &[u8], _w: u32, _h: u32) -> Result<()> {
            // TODO: mettre à jour la texture IYUV, copier dans le rect fit_rect,
            //       canvas.clear()/copy()/present().
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meme_ratio_remplit() {
        // Frame 16:9 dans fenêtre 16:9 -> occupe toute la fenêtre.
        assert_eq!(fit_rect(1920, 1080, 1280, 720), (0, 0, 1280, 720));
    }

    #[test]
    fn fenetre_trop_large_bandes_laterales() {
        // Fenêtre plus large que la frame -> centrage horizontal (x > 0, y = 0).
        let (x, y, w, h) = fit_rect(1080, 1920, 2000, 1080);
        assert_eq!(h, 1080);
        assert_eq!(y, 0);
        assert!(x > 0);
        assert!(w < 2000);
    }

    #[test]
    fn fenetre_trop_haute_bandes_horizontales() {
        let (x, y, _w, h) = fit_rect(1920, 1080, 1280, 1000);
        assert_eq!(x, 0);
        assert!(y > 0);
        assert!(h < 1000);
    }

    #[test]
    fn frame_vide_remplit_la_fenetre() {
        assert_eq!(fit_rect(0, 0, 800, 600), (0, 0, 800, 600));
    }
}
