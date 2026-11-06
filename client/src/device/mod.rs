//! Couche « appareil » : détection, modes de connexion et tunnel adb.

pub mod adb;
pub mod connection;
pub mod server_bin;

pub use connection::Connection;
