//! Couche « appareil » : détection, modes de connexion et tunnel adb.

pub mod adb;
pub mod connection;

pub use connection::Connection;
