//! Couche « appareil » : détection, modes de connexion et tunnel adb.

pub mod adb;
pub mod connection;
pub mod mdns;
pub mod server_bin;
pub mod tunnel;

pub use connection::Connection;
pub use tunnel::Tunnel;
