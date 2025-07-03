use bevy::prelude::Time;
use components::VesselData;
use crate::{SystemInteraction, SystemStatus, VesselSystem};

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

impl VesselSystem for RadarSystem {
    fn id(&self) -> &'static str {
        "radar"
    }

    fn display_name(&self) -> &'static str {
        "Radar System"
    }

    fn update(&mut self, _yacht_data: &VesselData, time: &Time) {
        // Update radar sweep angle
        self.sweep_angle = (time.elapsed_secs() * 60.0) % 360.0;
    }

    fn render_display(&self, _yacht_data: &VesselData) -> String {
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
