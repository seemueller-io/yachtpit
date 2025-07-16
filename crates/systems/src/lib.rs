#![allow(clippy::type_complexity)]

mod world;
mod vessel;
mod ais;
mod gps;
mod radar;
mod geo_plugin;

// Re-export components from the components crate
pub use components::{
    setup_instrument_cluster, update_instrument_displays, update_vessel_data, VesselData,
    SpeedGauge, DepthGauge, CompassGauge, EngineStatus, NavigationDisplay,
    InstrumentCluster, GpsIndicator, RadarIndicator, AisIndicator, SystemDisplay
};


pub use world::player::{get_vessel_systems, setup_instrument_cluster_system, PlayerPlugin};
pub use vessel::vessel_systems::{create_vessel_systems, AisSystem, GpsSystem, RadarSystem, SystemInteraction, SystemStatus, VesselSystem};

pub use geo_plugin::GeoPlugin;