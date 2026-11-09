//! Interface graphique de bureau (#32).
//!
//! Le **view-model** ([`state`]) est pur et toujours compilé (donc testé dans
//! le build par défaut). Le **rendu** egui ([`app`]) et le lancement de la
//! fenêtre ne sont compilés qu'avec la feature `gui`, pour garder le build par
//! défaut léger et sans dépendance graphique.

pub mod pairing;
pub mod state;

#[cfg(feature = "gui")]
mod app;

#[cfg(feature = "gui")]
pub use app::launch;
