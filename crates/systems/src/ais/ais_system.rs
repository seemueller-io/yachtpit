use bevy::prelude::Time;
use components::VesselData;
use crate::{SystemInteraction, SystemStatus, VesselSystem};
use datalink::DataMessage;
#[cfg(not(target_arch = "wasm32"))]
use datalink::{DataLink, DataLinkConfig, DataLinkReceiver};
#[cfg(not(target_arch = "wasm32"))]
use datalink_provider::AisDataLinkProvider;
use std::collections::HashMap;

/// AIS (Automatic Identification System) implementation
pub struct AisSystem {
    status: SystemStatus,
    own_mmsi: u32,
    receiving: bool,
    #[cfg(not(target_arch = "wasm32"))]
    datalink: AisDataLinkProvider,
    vessel_data: HashMap<String, DataMessage>,
}

impl AisSystem {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let datalink = {
            let mut datalink = AisDataLinkProvider::new();

            // Configure for serial AIS receiver (default configuration)
            // This can be customized based on available hardware
            let config = DataLinkConfig::new("ais".to_string())
                .with_parameter("connection_type".to_string(), "serial".to_string())
                .with_parameter("port".to_string(), "/dev/ttyUSB0".to_string())
                .with_parameter("baud_rate".to_string(), "38400".to_string());

            // Try to connect to the AIS datalink
            // If it fails, the system will still work but won't receive real AIS data
            if let Err(e) = datalink.connect(&config) {
                eprintln!("Failed to connect AIS datalink: {} (falling back to no external data)", e);
            }

            datalink
        };

        Self {
            status: SystemStatus::Active,
            own_mmsi: 123456789,
            receiving: true,
            #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
        if self.receiving && self.datalink.is_connected() {
            if let Ok(messages) = self.datalink.receive_all_messages() {
                for message in messages {
                    if message.message_type == "AIS_SENTENCE" {
                        // Process AIS sentence and extract vessel information
                        // For now, we'll create a mock vessel entry based on the sentence
                        // In a real implementation, you would decode the AIS payload
                        let mmsi = format!("AIS_{}", message.source_id);

                        // Create a processed message with basic vessel data
                        let mut processed_message = message.clone();
                        processed_message.message_type = "AIS_POSITION".to_string();

                        // Add mock vessel data (in real implementation, decode from payload)
                        processed_message = processed_message
                            .with_data("mmsi".to_string(), mmsi.clone())
                            .with_data("vessel_name".to_string(), format!("VESSEL_{}", message.source_id))
                            .with_data("latitude".to_string(), "37.7749".to_string())
                            .with_data("longitude".to_string(), "-122.4194".to_string())
                            .with_data("speed".to_string(), "0.0".to_string())
                            .with_data("course".to_string(), "0".to_string());

                        self.vessel_data.insert(mmsi, processed_message);
                    }
                }
            }
        }
    }

    fn render_display(&self, _yacht_data: &VesselData) -> String {
        let datalink_status = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if self.datalink.is_connected() { "CONNECTED" } else { "DISCONNECTED" }
            }
            #[cfg(target_arch = "wasm32")]
            {
                "OFFLINE"
            }
        };

        let mut display = format!(
            "AIS - AUTOMATIC IDENTIFICATION SYSTEM\n\n\
            Status: {}\n\
            Own Ship MMSI: {}\n\
            Datalink: {}\n\
            \n\
            NEARBY VESSELS:\n",
            if self.receiving { "RECEIVING" } else { "STANDBY" },
            self.own_mmsi,
            datalink_status
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
