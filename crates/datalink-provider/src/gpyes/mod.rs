use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use datalink::{DataLinkConfig, DataLinkError, DataLinkReceiver, DataLinkResult, DataLinkStatus, DataLinkTransmitter, DataMessage};

#[derive(Debug, Clone, PartialEq)]
pub struct LocationData {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub timestamp: Option<String>,
    pub fix_quality: Option<u8>,
    pub satellites: Option<u8>,
}

impl Default for LocationData {
    fn default() -> Self {
        LocationData {
            latitude: None,
            longitude: None,
            altitude: None,
            speed: None,
            timestamp: None,
            fix_quality: None,
            satellites: None,
        }
    }
}

pub struct GnssParser;

impl GnssParser {
    pub fn new() -> Self {
        GnssParser
    }

    pub fn parse_sentence(&self, sentence: &str) -> Option<LocationData> {
        if sentence.is_empty() || !sentence.starts_with('$') {
            return None;
        }

        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.is_empty() {
            return None;
        }

        let sentence_type = parts[0];

        match sentence_type {
            "$GPGGA" | "$GNGGA" => self.parse_gpgga(&parts),
            "$GPRMC" | "$GNRMC" => self.parse_gprmc(&parts),
            _ => None,
        }
    }

    fn parse_gpgga(&self, parts: &[&str]) -> Option<LocationData> {
        if parts.len() < 15 {
            return None;
        }

        let mut location = LocationData::default();

        // Parse timestamp (field 1)
        if !parts[1].is_empty() {
            location.timestamp = Some(parts[1].to_string());
        }

        // Parse latitude (fields 2 and 3)
        if !parts[2].is_empty() && !parts[3].is_empty() {
            if let Ok(lat_raw) = parts[2].parse::<f64>() {
                let degrees = (lat_raw / 100.0).floor();
                let minutes = lat_raw - (degrees * 100.0);
                let mut latitude = degrees + (minutes / 60.0);

                if parts[3] == "S" {
                    latitude = -latitude;
                }
                location.latitude = Some(latitude);
            }
        }

        // Parse longitude (fields 4 and 5)
        if !parts[4].is_empty() && !parts[5].is_empty() {
            if let Ok(lon_raw) = parts[4].parse::<f64>() {
                let degrees = (lon_raw / 100.0).floor();
                let minutes = lon_raw - (degrees * 100.0);
                let mut longitude = degrees + (minutes / 60.0);

                if parts[5] == "W" {
                    longitude = -longitude;
                }
                location.longitude = Some(longitude);
            }
        }

        // Parse fix quality (field 6)
        if !parts[6].is_empty() {
            if let Ok(quality) = parts[6].parse::<u8>() {
                location.fix_quality = Some(quality);
            }
        }

        // Parse number of satellites (field 7)
        if !parts[7].is_empty() {
            if let Ok(sats) = parts[7].parse::<u8>() {
                location.satellites = Some(sats);
            }
        }

        // Parse altitude (field 9)
        if !parts[9].is_empty() {
            if let Ok(alt) = parts[9].parse::<f64>() {
                location.altitude = Some(alt);
            }
        }

        Some(location)
    }

    fn parse_gprmc(&self, parts: &[&str]) -> Option<LocationData> {
        if parts.len() < 12 {
            return None;
        }

        let mut location = LocationData::default();

        // Parse timestamp (field 1)
        if !parts[1].is_empty() {
            location.timestamp = Some(parts[1].to_string());
        }

        // Check if data is valid (field 2)
        if parts[2] != "A" {
            return None; // Invalid data
        }

        // Parse latitude (fields 3 and 4)
        if !parts[3].is_empty() && !parts[4].is_empty() {
            if let Ok(lat_raw) = parts[3].parse::<f64>() {
                let degrees = (lat_raw / 100.0).floor();
                let minutes = lat_raw - (degrees * 100.0);
                let mut latitude = degrees + (minutes / 60.0);

                if parts[4] == "S" {
                    latitude = -latitude;
                }
                location.latitude = Some(latitude);
            }
        }

        // Parse longitude (fields 5 and 6)
        if !parts[5].is_empty() && !parts[6].is_empty() {
            if let Ok(lon_raw) = parts[5].parse::<f64>() {
                let degrees = (lon_raw / 100.0).floor();
                let minutes = lon_raw - (degrees * 100.0);
                let mut longitude = degrees + (minutes / 60.0);

                if parts[6] == "W" {
                    longitude = -longitude;
                }
                location.longitude = Some(longitude);
            }
        }

        // Parse speed (field 7) - in knots
        if !parts[7].is_empty() {
            if let Ok(speed_knots) = parts[7].parse::<f64>() {
                location.speed = Some(speed_knots);
            }
        }

        Some(location)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpyesSourceConfig {
    Serial {
        port: String,
        baud_rate: u32,
    },
    Tcp {
        host: String,
        port: u16,
    },
    Udp {
        bind_addr: String,
        port: u16,
    },
    File {
        path: String,
        replay_speed: f64,
    },
}

pub struct GpyesDataLinkProvider {
    status: DataLinkStatus,
    message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    parser: GnssParser,
}

impl GpyesDataLinkProvider {
    pub fn new() -> Self {
        GpyesDataLinkProvider {
            status: DataLinkStatus::Disconnected,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            shutdown_tx: None,
            parser: GnssParser::new(),
        }
    }

    pub fn parse_source_config(config: &DataLinkConfig) -> DataLinkResult<GpyesSourceConfig> {
        let connection_type = config.parameters.get("connection_type")
            .ok_or_else(|| DataLinkError::InvalidConfig("Missing connection_type parameter".to_string()))?;

        match connection_type.as_str() {
            "serial" => {
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port parameter for serial connection".to_string()))?
                    .clone();
                
                let baud_rate = config.parameters.get("baud_rate")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing baud_rate parameter for serial connection".to_string()))?
                    .parse::<u32>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid baud_rate parameter".to_string()))?;

                Ok(GpyesSourceConfig::Serial { port, baud_rate })
            }
            "tcp" => {
                let host = config.parameters.get("host")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing host parameter for TCP connection".to_string()))?
                    .clone();
                
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port parameter for TCP connection".to_string()))?
                    .parse::<u16>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid port parameter".to_string()))?;

                Ok(GpyesSourceConfig::Tcp { host, port })
            }
            "udp" => {
                let bind_addr = config.parameters.get("bind_addr")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing bind_addr parameter for UDP connection".to_string()))?
                    .clone();
                
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port parameter for UDP connection".to_string()))?
                    .parse::<u16>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid port parameter".to_string()))?;

                Ok(GpyesSourceConfig::Udp { bind_addr, port })
            }
            "file" => {
                let path = config.parameters.get("path")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing path parameter for file connection".to_string()))?
                    .clone();
                
                let replay_speed = config.parameters.get("replay_speed")
                    .unwrap_or(&"1.0".to_string())
                    .parse::<f64>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid replay_speed parameter".to_string()))?;

                Ok(GpyesSourceConfig::File { path, replay_speed })
            }
            _ => Err(DataLinkError::InvalidConfig(format!("Unsupported connection type: {}", connection_type)))
        }
    }

    fn start_receiver(&mut self) -> DataLinkResult<()> {
        // Implementation would be similar to GPS provider but using gpyes parser
        // For now, just set status to connected
        self.status = DataLinkStatus::Connected;
        Ok(())
    }

    fn location_data_to_message(&self, location: &LocationData) -> DataMessage {
        let mut message = DataMessage::new(
            "GPYES_LOCATION".to_string(),
            "GPYES_RECEIVER".to_string(),
            Vec::new(),
        );

        if let Some(lat) = location.latitude {
            message = message.with_data("latitude".to_string(), lat.to_string());
        }
        if let Some(lon) = location.longitude {
            message = message.with_data("longitude".to_string(), lon.to_string());
        }
        if let Some(alt) = location.altitude {
            message = message.with_data("altitude".to_string(), alt.to_string());
        }
        if let Some(speed) = location.speed {
            message = message.with_data("speed".to_string(), speed.to_string());
        }
        if let Some(timestamp) = &location.timestamp {
            message = message.with_data("timestamp".to_string(), timestamp.clone());
        }
        if let Some(quality) = location.fix_quality {
            message = message.with_data("fix_quality".to_string(), quality.to_string());
        }
        if let Some(sats) = location.satellites {
            message = message.with_data("satellites".to_string(), sats.to_string());
        }

        message
    }

    pub fn parse_gpyes_sentence(&self, sentence: &str) -> Option<DataMessage> {
        if let Some(location) = self.parser.parse_sentence(sentence) {
            Some(self.location_data_to_message(&location))
        } else {
            None
        }
    }

    fn stop_receiver(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.try_send(());
        }
        self.status = DataLinkStatus::Disconnected;
    }
}

impl Default for GpyesDataLinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLinkReceiver for GpyesDataLinkProvider {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn receive_message(&mut self) -> DataLinkResult<Option<DataMessage>> {
        if let Ok(mut queue) = self.message_queue.lock() {
            Ok(queue.pop_front())
        } else {
            Err(DataLinkError::TransportError("Failed to access message queue".to_string()))
        }
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        info!("Connecting GPYES data link provider with config: {:?}", config);
        
        let _source_config = Self::parse_source_config(config)?;
        
        self.status = DataLinkStatus::Connecting;
        
        // Start the receiver
        self.start_receiver()?;
        
        info!("GPYES data link provider connected successfully");
        Ok(())
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        info!("Disconnecting GPYES data link provider");
        
        self.stop_receiver();
        
        info!("GPYES data link provider disconnected");
        Ok(())
    }
}

impl DataLinkTransmitter for GpyesDataLinkProvider {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn send_message(&mut self, _message: &DataMessage) -> DataLinkResult<()> {
        // GPYES provider is primarily a receiver, but we implement this for completeness
        Err(DataLinkError::TransportError("GPYES provider does not support message transmission".to_string()))
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        DataLinkReceiver::connect(self, config)
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        DataLinkReceiver::disconnect(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let _parser = GnssParser::new();
        // Parser should be created successfully
    }

    #[test]
    fn test_parse_gpgga_sentence() {
        let parser = GnssParser::new();

        // Example GPGGA sentence: Global Positioning System Fix Data
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.latitude.is_some());
        assert!(location.longitude.is_some());
        assert!(location.altitude.is_some());
        assert!(location.fix_quality.is_some());
        assert!(location.satellites.is_some());

        // Check specific values
        assert!((location.latitude.unwrap() - 48.1173).abs() < 0.001); // 4807.038N
        assert!((location.longitude.unwrap() - 11.5167).abs() < 0.001); // 01131.000E
        assert!((location.altitude.unwrap() - 545.4).abs() < 0.1);
        assert_eq!(location.fix_quality.unwrap(), 1);
        assert_eq!(location.satellites.unwrap(), 8);
    }

    #[test]
    fn test_parse_gprmc_sentence() {
        let parser = GnssParser::new();

        // Example GPRMC sentence: Recommended Minimum Navigation Information
        let sentence = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.latitude.is_some());
        assert!(location.longitude.is_some());
        assert!(location.speed.is_some());
        assert!(location.timestamp.is_some());

        // Check specific values
        assert!((location.latitude.unwrap() - 48.1173).abs() < 0.001);
        assert!((location.longitude.unwrap() - 11.5167).abs() < 0.001);
        assert!((location.speed.unwrap() - 22.4).abs() < 0.1);
    }

    #[test]
    fn test_parse_gngga_sentence() {
        let parser = GnssParser::new();

        // Example GNGGA sentence (modern GNSS format)
        let sentence = "$GNGGA,144751.00,3708.15162,N,07621.52868,W,1,06,1.39,-14.3,M,-35.8,M,,*69";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.latitude.is_some());
        assert!(location.longitude.is_some());
        assert!(location.altitude.is_some());
        assert!(location.fix_quality.is_some());
        assert!(location.satellites.is_some());

        // Check specific values
        assert!((location.latitude.unwrap() - 37.1359).abs() < 0.001); // 3708.15162N
        assert!((location.longitude.unwrap() - (-76.3588)).abs() < 0.001); // 07621.52868W
        assert!((location.altitude.unwrap() - (-14.3)).abs() < 0.1);
        assert_eq!(location.fix_quality.unwrap(), 1);
        assert_eq!(location.satellites.unwrap(), 6);
    }

    #[test]
    fn test_parse_gnrmc_sentence() {
        let parser = GnssParser::new();

        // Example GNRMC sentence (modern GNSS format)
        let sentence = "$GNRMC,144751.00,A,3708.15162,N,07621.52868,W,0.009,,200725,,,A,V*01";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.latitude.is_some());
        assert!(location.longitude.is_some());
        assert!(location.speed.is_some());
        assert!(location.timestamp.is_some());

        // Check specific values
        assert!((location.latitude.unwrap() - 37.1359).abs() < 0.001);
        assert!((location.longitude.unwrap() - (-76.3588)).abs() < 0.001);
        assert!((location.speed.unwrap() - 0.009).abs() < 0.001);
    }

    #[test]
    fn test_parse_invalid_sentence() {
        let parser = GnssParser::new();

        let invalid_sentence = "invalid sentence";
        let result = parser.parse_sentence(invalid_sentence);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_sentence() {
        let parser = GnssParser::new();

        let result = parser.parse_sentence("");
        assert!(result.is_none());
    }

    #[test]
    fn test_location_data_default() {
        let location = LocationData::default();
        assert!(location.latitude.is_none());
        assert!(location.longitude.is_none());
        assert!(location.altitude.is_none());
        assert!(location.speed.is_none());
        assert!(location.timestamp.is_none());
        assert!(location.fix_quality.is_none());
        assert!(location.satellites.is_none());
    }

    #[test]
    fn test_gpyes_provider_creation() {
        let provider = GpyesDataLinkProvider::new();
        assert!(matches!(DataLinkReceiver::status(&provider), DataLinkStatus::Disconnected));
    }

    #[test]
    fn test_location_data_to_message() {
        let provider = GpyesDataLinkProvider::new();
        let mut location = LocationData::default();
        location.latitude = Some(48.1173);
        location.longitude = Some(11.5167);
        location.altitude = Some(545.4);
        location.fix_quality = Some(1);
        location.satellites = Some(8);

        let message = provider.location_data_to_message(&location);
        assert_eq!(message.message_type, "GPYES_LOCATION");
        assert_eq!(message.source_id, "GPYES_RECEIVER");
        assert_eq!(message.get_data("latitude"), Some(&"48.1173".to_string()));
        assert_eq!(message.get_data("longitude"), Some(&"11.5167".to_string()));
        assert_eq!(message.get_data("altitude"), Some(&"545.4".to_string()));
        assert_eq!(message.get_data("fix_quality"), Some(&"1".to_string()));
        assert_eq!(message.get_data("satellites"), Some(&"8".to_string()));
    }

    #[test]
    fn test_parse_gpyes_sentence() {
        let provider = GpyesDataLinkProvider::new();
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";

        let message = provider.parse_gpyes_sentence(sentence);
        assert!(message.is_some());

        let msg = message.unwrap();
        assert_eq!(msg.message_type, "GPYES_LOCATION");
        assert_eq!(msg.source_id, "GPYES_RECEIVER");
        assert!(msg.get_data("latitude").is_some());
        assert!(msg.get_data("longitude").is_some());
        assert!(msg.get_data("altitude").is_some());
    }
}