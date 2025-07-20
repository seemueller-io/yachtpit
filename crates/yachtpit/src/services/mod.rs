pub mod gps_service;

#[cfg(not(target_arch = "wasm32"))]
pub mod gpyes_provider;

pub use gps_service::*;