use bevy::prelude::Time;
use components::VesselData;
use crate::{SystemInteraction, SystemStatus, VesselSystem};

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

impl VesselSystem for GpsSystem {
    fn id(&self) -> &'static str {
        "gps"
    }

    fn display_name(&self) -> &'static str {
        "GPS Navigation"
    }

    fn update(&mut self, _yacht_data: &VesselData, time: &Time) {
        // Simulate satellite connection variations
        let t = time.elapsed_secs();
        self.satellites_connected = (12.0 + (t * 0.1).sin() * 2.0).max(8.0) as u8;
        self.hdop = 0.8 + (t * 0.05).sin() * 0.2;
    }

    fn render_display(&self, yacht_data: &VesselData) -> String {
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
