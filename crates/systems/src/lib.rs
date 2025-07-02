#![allow(clippy::type_complexity)]

pub mod player;
pub mod cluster;
pub mod instruments;
pub mod systems;
pub mod yacht_systems;
pub mod instrument_theme;

pub use player::{get_yacht_systems, setup_instrument_cluster_system, PlayerPlugin};
pub use yacht_systems::{create_yacht_systems, AisSystem, GpsSystem, RadarSystem, SystemInteraction, SystemStatus, YachtSystem};
pub use cluster::setup_instrument_cluster;
pub use instruments::{update_instrument_displays, update_yacht_data, YachtData};
