use bevy::prelude::Time;
use components::VesselData;
use crate::{SystemInteraction, SystemStatus, VesselSystem};

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

impl VesselSystem for AisSystem {
    fn id(&self) -> &'static str {
        "ais"
    }

    fn display_name(&self) -> &'static str {
        "AIS System"
    }

    fn update(&mut self, _yacht_data: &VesselData, _time: &Time) {
        // AIS system is relatively static, but we could simulate
        // vessel movements or signal strength variations here
    }

    fn render_display(&self, _yacht_data: &VesselData) -> String {
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