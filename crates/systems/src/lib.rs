#![allow(clippy::type_complexity)]

pub mod player;
pub mod systems;
pub mod yacht_systems;

// Re-export components from the components crate
pub use components::{
    setup_instrument_cluster, update_instrument_displays, update_yacht_data, YachtData,
    SpeedGauge, DepthGauge, CompassGauge, EngineStatus, NavigationDisplay,
    InstrumentCluster, GpsIndicator, RadarIndicator, AisIndicator, SystemDisplay
};

pub use player::{get_yacht_systems, setup_instrument_cluster_system, PlayerPlugin};
pub use yacht_systems::{create_yacht_systems, AisSystem, GpsSystem, RadarSystem, SystemInteraction, SystemStatus, YachtSystem};
