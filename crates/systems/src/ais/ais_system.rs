use bevy::prelude::Time;
use components::VesselData;
use crate::{SystemInteraction, SystemStatus, VesselSystem};
use datalink::{DataLink, DataLinkConfig, DataLinkReceiver, DataMessage, SimulationDataLink};
use std::collections::HashMap;

/// AIS (Automatic Identification System) implementation
pub struct AisSystem {
    status: SystemStatus,
    own_mmsi: u32,
    receiving: bool,
    datalink: SimulationDataLink,
    vessel_data: HashMap<String, DataMessage>,
}

impl AisSystem {
    pub fn new() -> Self {
        let mut datalink = SimulationDataLink::new();
        let config = DataLinkConfig::new("simulation".to_string());

        // Connect to the simulation datalink
        if let Err(e) = datalink.connect(&config) {
            eprintln!("Failed to connect AIS datalink: {}", e);
        }

        Self {
            status: SystemStatus::Active,
            own_mmsi: 123456789,
            receiving: true,
            datalink,
            vessel_data: HashMap::new(),
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
        // Receive new AIS messages from the datalink
        if self.receiving && self.datalink.is_connected() {
            if let Ok(messages) = self.datalink.receive_all_messages() {
                for message in messages {
                    if message.message_type == "AIS_POSITION" {
                        // Store vessel data by MMSI
                        if let Some(mmsi) = message.get_data("mmsi") {
                            self.vessel_data.insert(mmsi.clone(), message);
                        }
                    }
                }
            }
        }
    }

    fn render_display(&self, _yacht_data: &VesselData) -> String {
        let mut display = format!(
            "AIS - AUTOMATIC IDENTIFICATION SYSTEM\n\n\
            Status: {}\n\
            Own Ship MMSI: {}\n\
            Datalink: {}\n\
            \n\
            NEARBY VESSELS:\n",
            if self.receiving { "RECEIVING" } else { "STANDBY" },
            self.own_mmsi,
            if self.datalink.is_connected() { "CONNECTED" } else { "DISCONNECTED" }
        );

        if self.vessel_data.is_empty() {
            display.push_str("\nNo vessels detected");
        } else {
            for (mmsi, message) in &self.vessel_data {
                let vessel_name = message.get_data("vessel_name").unwrap_or(mmsi);
                let speed = message.get_data("speed").map(|s| s.as_str()).unwrap_or("N/A");
                let course = message.get_data("course").map(|s| s.as_str()).unwrap_or("N/A");
                let lat = message.get_data("latitude").map(|s| s.as_str()).unwrap_or("N/A");
                let lon = message.get_data("longitude").map(|s| s.as_str()).unwrap_or("N/A");

                // Determine vessel icon based on name
                let icon = if vessel_name.contains("M/Y") || vessel_name.contains("YACHT") {
                    "🛥️"
                } else if vessel_name.contains("CARGO") || vessel_name.contains("SHIP") {
                    "🚢"
                } else if vessel_name.contains("S/Y") || vessel_name.contains("SAIL") {
                    "⛵"
                } else {
                    "🚤"
                };

                display.push_str(&format!(
                    "\n{} {}\n\
                    MMSI: {}\n\
                    Position: {}°N, {}°W\n\
                    Speed: {} kts\n\
                    Course: {}°\n",
                    icon, vessel_name, mmsi, lat, lon, speed, course
                ));

                if let Some(quality) = message.signal_quality {
                    display.push_str(&format!("Signal: {}%\n", quality));
                }
                display.push('\n');
            }
        }

        display
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
