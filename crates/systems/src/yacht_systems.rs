//! Concrete implementations of yacht systems using the SystemManager abstraction
//! 
//! This module provides implementations of the YachtSystem trait for GPS, Radar, and AIS systems,
//! bridging the existing functionality with the new higher-level abstraction.

use bevy::prelude::*;
use components::YachtData;

/// Status of a yacht system
#[derive(Debug, Clone, PartialEq)]
pub enum SystemStatus {
    Active,
    Inactive,
    Error(String),
    Maintenance,
}

/// Interaction types for yacht systems
#[derive(Debug, Clone)]
pub enum SystemInteraction {
    Select,
    Toggle,
    Reset,
    Configure(String, String),
}

/// Common trait for all yacht systems
pub trait YachtSystem: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn update(&mut self, yacht_data: &YachtData, time: &Time);
    fn render_display(&self, yacht_data: &YachtData) -> String;
    fn handle_interaction(&mut self, interaction: SystemInteraction) -> bool;
    fn status(&self) -> SystemStatus;
}

/// GPS Navigation System implementation
pub struct GpsSystem {
    status: SystemStatus,
    satellites_connected: u8,
    hdop: f32,
}

impl GpsSystem {
    pub fn new() -> Self {
        Self {
            status: SystemStatus::Active,
            satellites_connected: 12,
            hdop: 0.8,
        }
    }
}

impl YachtSystem for GpsSystem {
    fn id(&self) -> &'static str {
        "gps"
    }

    fn display_name(&self) -> &'static str {
        "GPS Navigation"
    }

    fn update(&mut self, _yacht_data: &YachtData, time: &Time) {
        // Simulate satellite connection variations
        let t = time.elapsed_secs();
        self.satellites_connected = (12.0 + (t * 0.1).sin() * 2.0).max(8.0) as u8;
        self.hdop = 0.8 + (t * 0.05).sin() * 0.2;
    }

    fn render_display(&self, yacht_data: &YachtData) -> String {
        format!(
            "GPS NAVIGATION SYSTEM\n\n\
            Position: 43°38'19.5\"N 1°26'58.3\"W\n\
            Heading: {:.0}°\n\
            Speed: {:.1} knots\n\
            Course Over Ground: {:.0}°\n\
            Satellites: {} connected\n\
            HDOP: {:.1} ({})\n\
            \n\
            Next Waypoint: MONACO HARBOR\n\
            Distance: 127.3 NM\n\
            ETA: 10h 12m",
            yacht_data.heading,
            yacht_data.speed,
            yacht_data.heading + 5.0,
            self.satellites_connected,
            self.hdop,
            if self.hdop < 1.0 { "Excellent" } else if self.hdop < 2.0 { "Good" } else { "Fair" }
        )
    }

    fn handle_interaction(&mut self, interaction: SystemInteraction) -> bool {
        match interaction {
            SystemInteraction::Select => {
                self.status = SystemStatus::Active;
                true
            }
            SystemInteraction::Reset => {
                self.satellites_connected = 12;
                self.hdop = 0.8;
                true
            }
            SystemInteraction::Toggle => {
                self.status = match self.status {
                    SystemStatus::Active => SystemStatus::Inactive,
                    SystemStatus::Inactive => SystemStatus::Active,
                    _ => SystemStatus::Active,
                };
                true
            }
            _ => false,
        }
    }

    fn status(&self) -> SystemStatus {
        self.status.clone()
    }
}

/// Radar System implementation
pub struct RadarSystem {
    status: SystemStatus,
    range_nm: f32,
    gain: String,
    sea_clutter_db: i8,
    rain_clutter: bool,
    sweep_angle: f32,
}

impl RadarSystem {
    pub fn new() -> Self {
        Self {
            status: SystemStatus::Active,
            range_nm: 12.0,
            gain: "AUTO".to_string(),
            sea_clutter_db: -15,
            rain_clutter: false,
            sweep_angle: 0.0,
        }
    }
}

impl YachtSystem for RadarSystem {
    fn id(&self) -> &'static str {
        "radar"
    }

    fn display_name(&self) -> &'static str {
        "Radar System"
    }

    fn update(&mut self, _yacht_data: &YachtData, time: &Time) {
        // Update radar sweep angle
        self.sweep_angle = (time.elapsed_secs() * 60.0) % 360.0;
    }

    fn render_display(&self, _yacht_data: &YachtData) -> String {
        format!(
            "RADAR SYSTEM - {:.0} NM RANGE\n\n\
            Status: {}\n\
            Sweep: {:.0}°\n\
            Gain: {}\n\
            Sea Clutter: {} dB\n\
            Rain Clutter: {}\n\
            \n\
            CONTACTS DETECTED:\n\
            • Vessel 1: 2.3 NM @ 045° (15 kts)\n\
            • Vessel 2: 5.7 NM @ 180° (8 kts)\n\
            • Land Mass: 8.2 NM @ 270°\n\
            • Buoy: 1.1 NM @ 315°",
            self.range_nm,
            match self.status {
                SystemStatus::Active => "ACTIVE",
                SystemStatus::Inactive => "STANDBY",
                SystemStatus::Error(_) => "ERROR",
                SystemStatus::Maintenance => "MAINTENANCE",
            },
            self.sweep_angle,
            self.gain,
            self.sea_clutter_db,
            if self.rain_clutter { "ON" } else { "OFF" }
        )
    }

    fn handle_interaction(&mut self, interaction: SystemInteraction) -> bool {
        match interaction {
            SystemInteraction::Select => {
                self.status = SystemStatus::Active;
                true
            }
            SystemInteraction::Configure(key, value) => {
                match key.as_str() {
                    "range" => {
                        if let Ok(range) = value.parse::<f32>() {
                            self.range_nm = range.clamp(1.0, 48.0);
                            true
                        } else {
                            false
                        }
                    }
                    "gain" => {
                        self.gain = value;
                        true
                    }
                    "sea_clutter" => {
                        if let Ok(db) = value.parse::<i8>() {
                            self.sea_clutter_db = db.clamp(-30, 0);
                            true
                        } else {
                            false
                        }
                    }
                    "rain_clutter" => {
                        self.rain_clutter = value.to_lowercase() == "on" || value == "true";
                        true
                    }
                    _ => false,
                }
            }
            SystemInteraction::Reset => {
                self.range_nm = 12.0;
                self.gain = "AUTO".to_string();
                self.sea_clutter_db = -15;
                self.rain_clutter = false;
                true
            }
            SystemInteraction::Toggle => {
                self.status = match self.status {
                    SystemStatus::Active => SystemStatus::Inactive,
                    SystemStatus::Inactive => SystemStatus::Active,
                    _ => SystemStatus::Active,
                };
                true
            }
        }
    }

    fn status(&self) -> SystemStatus {
        self.status.clone()
    }
}

/// AIS (Automatic Identification System) implementation
pub struct AisSystem {
    status: SystemStatus,
    own_mmsi: u32,
    receiving: bool,
}

impl AisSystem {
    pub fn new() -> Self {
        Self {
            status: SystemStatus::Active,
            own_mmsi: 123456789,
            receiving: true,
        }
    }
}

impl YachtSystem for AisSystem {
    fn id(&self) -> &'static str {
        "ais"
    }

    fn display_name(&self) -> &'static str {
        "AIS System"
    }

    fn update(&mut self, _yacht_data: &YachtData, _time: &Time) {
        // AIS system is relatively static, but we could simulate
        // vessel movements or signal strength variations here
    }

    fn render_display(&self, _yacht_data: &YachtData) -> String {
        format!(
            "AIS - AUTOMATIC IDENTIFICATION SYSTEM\n\n\
            Status: {}\n\
            Own Ship MMSI: {}\n\
            \n\
            NEARBY VESSELS:\n\
            \n\
            🛥️ M/Y SERENITY\n\
            MMSI: 987654321\n\
            Distance: 2.1 NM @ 045°\n\
            Speed: 12.5 kts\n\
            Course: 180°\n\
            \n\
            🚢 CARGO VESSEL ATLANTIS\n\
            MMSI: 456789123\n\
            Distance: 5.8 NM @ 270°\n\
            Speed: 18.2 kts\n\
            Course: 090°\n\
            \n\
            ⛵ S/Y WIND DANCER\n\
            MMSI: 789123456\n\
            Distance: 1.3 NM @ 135°\n\
            Speed: 6.8 kts\n\
            Course: 225°",
            if self.receiving { "RECEIVING" } else { "STANDBY" },
            self.own_mmsi
        )
    }

    fn handle_interaction(&mut self, interaction: SystemInteraction) -> bool {
        match interaction {
            SystemInteraction::Select => {
                self.status = SystemStatus::Active;
                self.receiving = true;
                true
            }
            SystemInteraction::Configure(key, value) => {
                match key.as_str() {
                    "mmsi" => {
                        if let Ok(mmsi) = value.parse::<u32>() {
                            self.own_mmsi = mmsi;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            SystemInteraction::Toggle => {
                self.receiving = !self.receiving;
                self.status = if self.receiving {
                    SystemStatus::Active
                } else {
                    SystemStatus::Inactive
                };
                true
            }
            SystemInteraction::Reset => {
                self.own_mmsi = 123456789;
                self.receiving = true;
                self.status = SystemStatus::Active;
                true
            }
        }
    }

    fn status(&self) -> SystemStatus {
        self.status.clone()
    }
}

/// Helper function to create and register all yacht systems
pub fn create_yacht_systems() -> Vec<Box<dyn YachtSystem>> {
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

        let yacht_data = YachtData::default();
        let display = gps.render_display(&yacht_data);
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
        let display = radar.render_display(&YachtData::default());
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
    fn test_create_yacht_systems() {
        let systems = create_yacht_systems();
        assert_eq!(systems.len(), 3);

        let ids: Vec<&str> = systems.iter().map(|s| s.id()).collect();
        assert!(ids.contains(&"gps"));
        assert!(ids.contains(&"radar"));
        assert!(ids.contains(&"ais"));
    }
}
