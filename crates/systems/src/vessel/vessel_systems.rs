//! Concrete implementations of vessel systems using the SystemManager abstraction
//! 
//! This module provides implementations of the VesselSystem trait for GPS, Radar, and AIS systems,
//! bridging the existing functionality with the new higher-level abstraction.

pub use crate::ais::ais_system::AisSystem;
pub use crate::gps::gps_system::GpsSystem;
pub use crate::radar::radar_system::RadarSystem;
use bevy::prelude::*;
use components::VesselData;



/// Common trait for all yacht systems
pub trait VesselSystem: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn update(&mut self, yacht_data: &VesselData, time: &Time);
    fn render_display(&self, yacht_data: &VesselData) -> String;
    fn handle_interaction(&mut self, interaction: SystemInteraction) -> bool;
    fn status(&self) -> SystemStatus;
}


/// Status of a vessel system
#[derive(Debug, Clone, PartialEq)]
pub enum SystemStatus {
    Active,
    Inactive,
    Error(String),
    Maintenance,
}

/// Interaction types for vessel systems
#[derive(Debug, Clone)]
pub enum SystemInteraction {
    Select,
    Toggle,
    Reset,
    Configure(String, String),
}



/// Helper function to create and register all vessel systems
pub fn create_vessel_systems() -> Vec<Box<dyn VesselSystem>> {
    vec![
        Box::new(GpsSystem::new()),
        Box::new(RadarSystem::new()),
        Box::new(AisSystem::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_system() {
        let gps = GpsSystem::new();
        assert_eq!(gps.id(), "gps");
        assert_eq!(gps.display_name(), "GPS Navigation");
        assert_eq!(gps.status(), SystemStatus::Active);

        let vessel_data = VesselData::default();
        let display = gps.render_display(&vessel_data);
        assert!(display.contains("GPS NAVIGATION SYSTEM"));
        assert!(display.contains("Satellites: 12 connected"));
    }

    #[test]
    fn test_radar_system() {
        let mut radar = RadarSystem::new();
        assert_eq!(radar.id(), "radar");
        assert_eq!(radar.display_name(), "Radar System");

        // Test configuration
        assert!(radar.handle_interaction(SystemInteraction::Configure("range".to_string(), "24".to_string())));
        let display = radar.render_display(&VesselData::default());
        assert!(display.contains("24 NM RANGE"));
    }

    #[test]
    fn test_ais_system() {
        let mut ais = AisSystem::new();
        assert_eq!(ais.id(), "ais");
        assert_eq!(ais.display_name(), "AIS System");

        // Test toggle
        assert!(ais.handle_interaction(SystemInteraction::Toggle));
        assert_eq!(ais.status(), SystemStatus::Inactive);
    }

    #[test]
    fn test_create_vessel_systems() {
        let systems = create_vessel_systems();
        assert_eq!(systems.len(), 3);

        let ids: Vec<&str> = systems.iter().map(|s| s.id()).collect();
        assert!(ids.contains(&"gps"));
        assert!(ids.contains(&"radar"));
        assert!(ids.contains(&"ais"));
    }
}
