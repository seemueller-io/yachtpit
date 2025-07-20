//! GPS Device Implementation
//! 
//! This module provides a GPS device that integrates with the hardware abstraction layer.
//! It uses NMEA sentence parsing to extract location data from GPS hardware and broadcasts
//! location updates via the hardware bus.

use crate::{
    BusAddress, BusMessage, DeviceCapability, DeviceConfig, DeviceInfo, DeviceStatus,
    HardwareError, Result, SystemDevice,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// GPS location data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// GNSS parser for NMEA sentences
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

/// GPS Device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsDeviceConfig {
    pub device_paths: Vec<String>,
    pub baud_rate: u32,
    pub timeout_ms: u64,
    pub auto_reconnect: bool,
    pub broadcast_interval_ms: u64,
}

impl Default for GpsDeviceConfig {
    fn default() -> Self {
        GpsDeviceConfig {
            device_paths: vec![
                "/dev/tty.usbmodem2101".to_string(),
                "/dev/cu.usbmodem2101".to_string(),
                "/dev/ttyUSB0".to_string(),
                "/dev/ttyACM0".to_string(),
            ],
            baud_rate: 9600,
            timeout_ms: 1000,
            auto_reconnect: true,
            broadcast_interval_ms: 1000,
        }
    }
}

/// GPS Device implementation
pub struct GpsDevice {
    device_info: DeviceInfo,
    gps_config: GpsDeviceConfig,
    parser: GnssParser,
    last_location: Option<LocationData>,
    serial_port: Option<Arc<Mutex<Box<dyn serialport::SerialPort>>>>,
    running: bool,
}

impl GpsDevice {
    pub fn new(gps_config: GpsDeviceConfig) -> Self {
        let address = BusAddress::new("GPS_DEVICE");
        
        // Create device config for the hardware abstraction layer
        let device_config = DeviceConfig {
            name: "GPS Device".to_string(),
            capabilities: vec![DeviceCapability::Gps, DeviceCapability::Navigation],
            update_interval_ms: gps_config.broadcast_interval_ms,
            max_queue_size: 100,
            custom_config: HashMap::new(),
        };

        let device_info = DeviceInfo {
            address,
            config: device_config,
            status: DeviceStatus::Offline,
            last_seen: SystemTime::now(),
            version: "1.0.0".to_string(),
            manufacturer: "YachtPit".to_string(),
        };
        
        GpsDevice {
            device_info,
            gps_config,
            parser: GnssParser::new(),
            last_location: None,
            serial_port: None,
            running: false,
        }
    }

    pub fn with_address(mut self, address: BusAddress) -> Self {
        self.device_info.address = address;
        self
    }

    async fn try_connect_serial(&mut self) -> Result<()> {
        for device_path in &self.gps_config.device_paths {
            debug!("Attempting to connect to GPS device at: {}", device_path);
            
            match serialport::new(device_path, self.gps_config.baud_rate)
                .timeout(Duration::from_millis(self.gps_config.timeout_ms))
                .open()
            {
                Ok(port) => {
                    info!("Successfully connected to GPS device at {}", device_path);
                    self.serial_port = Some(Arc::new(Mutex::new(port)));
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed to connect to {}: {}", device_path, e);
                }
            }
        }
        
        Err(HardwareError::generic(
            "Could not connect to any GPS device"
        ))
    }

    async fn read_and_parse_gps_data(&mut self) -> Result<Vec<BusMessage>> {
        let mut messages = Vec::new();
        
        if let Some(ref serial_port) = self.serial_port {
            let mut port_guard = serial_port.lock().await;
            let mut reader = BufReader::new(port_guard.as_mut());
            let mut line = String::new();
            
            // Try to read a line with timeout handling
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF - connection lost
                    warn!("GPS device connection lost (EOF)");
                    drop(port_guard); // Release the lock before modifying self
                    self.serial_port = None;
                    self.device_info.status = DeviceStatus::Offline;
                }
                Ok(_) => {
                    let sentence = line.trim();
                    if !sentence.is_empty() {
                        debug!("Received GPS sentence: {}", sentence);
                        
                        if let Some(location) = self.parser.parse_sentence(sentence) {
                            info!("Parsed GPS location: {:?}", location);
                            self.last_location = Some(location.clone());
                            
                            // Create broadcast message with location data
                            if let Ok(payload) = serde_json::to_vec(&location) {
                                let message = BusMessage::Broadcast {
                                    from: self.device_info.address.clone(),
                                    payload,
                                    message_id: Uuid::new_v4(),
                                };
                                messages.push(message);
                            }
                        }
                    }
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::TimedOut => {
                            // Timeout is normal, just continue
                            debug!("GPS read timeout - continuing");
                        }
                        ErrorKind::Interrupted => {
                            // Interrupted system call - continue
                            debug!("GPS read interrupted - continuing");
                        }
                        _ => {
                            error!("Error reading from GPS device: {}", e);
                            drop(port_guard); // Release the lock before modifying self
                            self.serial_port = None;
                            self.device_info.status = DeviceStatus::Error {
                                message: format!("GPS read error: {}", e),
                            };
                        }
                    }
                }
            }
        }
        
        Ok(messages)
    }

    pub fn get_last_location(&self) -> Option<&LocationData> {
        self.last_location.as_ref()
    }
}

#[async_trait::async_trait]
impl SystemDevice for GpsDevice {
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing GPS device");
        self.device_info.status = DeviceStatus::Initializing;
        self.device_info.last_seen = SystemTime::now();
        
        // Try to connect to GPS hardware
        match self.try_connect_serial().await {
            Ok(()) => {
                self.device_info.status = DeviceStatus::Online;
                info!("GPS device initialized successfully");
                Ok(())
            }
            Err(e) => {
                warn!("GPS device initialization failed: {}", e);
                self.device_info.status = DeviceStatus::Error {
                    message: format!("Initialization failed: {}", e),
                };
                // Return error when hardware GPS is not available
                Err(e)
            }
        }
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting GPS device");
        self.running = true;
        self.device_info.last_seen = SystemTime::now();
        
        if self.serial_port.is_some() {
            self.device_info.status = DeviceStatus::Online;
        } else {
            // Try to reconnect if auto-reconnect is enabled
            if self.gps_config.auto_reconnect {
                if let Ok(()) = self.try_connect_serial().await {
                    self.device_info.status = DeviceStatus::Online;
                }
            }
        }
        
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping GPS device");
        self.running = false;
        self.serial_port = None;
        self.device_info.status = DeviceStatus::Offline;
        self.device_info.last_seen = SystemTime::now();
        Ok(())
    }

    fn get_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }

    fn get_status(&self) -> DeviceStatus {
        self.device_info.status.clone()
    }

    async fn handle_message(&mut self, message: BusMessage) -> Result<Option<BusMessage>> {
        debug!("GPS device received message: {:?}", message);
        self.device_info.last_seen = SystemTime::now();
        
        match message {
            BusMessage::Control { command, .. } => {
                match command {
                    crate::bus::ControlCommand::Ping { target } => {
                        if target == self.device_info.address {
                            return Ok(Some(BusMessage::Control {
                                from: self.device_info.address.clone(),
                                command: crate::bus::ControlCommand::Pong {
                                    from: self.device_info.address.clone(),
                                },
                                message_id: Uuid::new_v4(),
                            }));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(None)
    }

    async fn process(&mut self) -> Result<Vec<BusMessage>> {
        if !self.running {
            return Ok(Vec::new());
        }

        self.device_info.last_seen = SystemTime::now();

        // Try to read GPS data if connected
        if self.serial_port.is_some() {
            self.read_and_parse_gps_data().await
        } else if self.gps_config.auto_reconnect {
            // Try to reconnect
            if let Ok(()) = self.try_connect_serial().await {
                info!("GPS device reconnected successfully");
                self.device_info.status = DeviceStatus::Online;
            }
            Ok(Vec::new())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_capabilities(&self) -> Vec<DeviceCapability> {
        self.device_info.config.capabilities.clone()
    }

    async fn update_config(&mut self, _config: DeviceConfig) -> Result<()> {
        // For now, we don't support dynamic config updates
        // In a real implementation, you might want to parse the config
        // and update the GPS-specific settings
        warn!("GPS device config update not implemented");
        self.device_info.last_seen = SystemTime::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gnss_parser_creation() {
        let parser = GnssParser::new();
        // Parser should be created successfully
    }

    #[test]
    fn test_parse_gpgga_sentence() {
        let parser = GnssParser::new();
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.latitude.is_some());
        assert!(location.longitude.is_some());
        assert!(location.altitude.is_some());
        assert!((location.latitude.unwrap() - 48.1173).abs() < 0.001);
        assert!((location.longitude.unwrap() - 11.5167).abs() < 0.001);
    }

    #[test]
    fn test_gps_device_creation() {
        let config = GpsDeviceConfig::default();
        let device = GpsDevice::new(config);
        assert_eq!(device.get_status(), DeviceStatus::Offline);
        assert!(device.get_capabilities().contains(&DeviceCapability::Gps));
    }

    #[test]
    fn test_location_data_serialization() {
        let location = LocationData {
            latitude: Some(48.1173),
            longitude: Some(11.5167),
            altitude: Some(545.4),
            speed: Some(22.4),
            timestamp: Some("123519".to_string()),
            fix_quality: Some(1),
            satellites: Some(8),
        };

        let serialized = serde_json::to_string(&location).unwrap();
        let deserialized: LocationData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(location, deserialized);
    }
}