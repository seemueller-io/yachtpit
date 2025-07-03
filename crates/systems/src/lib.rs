#![allow(clippy::type_complexity)]

pub mod player;
mod marine;
use marine::*;


// Re-export components from the components crate
pub use components::{
    setup_instrument_cluster, update_instrument_displays, update_vessel_data, VesselData,
    SpeedGauge, DepthGauge, CompassGauge, EngineStatus, NavigationDisplay,
    InstrumentCluster, GpsIndicator, RadarIndicator, AisIndicator, SystemDisplay
};

pub use player::{get_vessel_systems, setup_instrument_cluster_system, PlayerPlugin};
pub use vessel_systems::{create_vessel_systems, AisSystem, GpsSystem, RadarSystem, SystemInteraction, SystemStatus, VesselSystem};
