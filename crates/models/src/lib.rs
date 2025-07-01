#![allow(clippy::type_complexity)]

pub mod player;
pub mod cluster;
pub mod instruments;
pub mod systems;
pub mod yacht_systems;

pub use player::{PlayerPlugin, setup_instrument_cluster_system, get_yacht_systems};
pub use yacht_systems::{GpsSystem, RadarSystem, AisSystem, create_yacht_systems, SystemStatus, SystemInteraction, YachtSystem};
pub use cluster::setup_instrument_cluster;
pub use instruments::{YachtData, update_yacht_data, update_instrument_displays};
