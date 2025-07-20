use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::io::{BufRead, BufReader, ErrorKind};
use tokio::sync::mpsc;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::gps_service::GpsData;

/// Enhanced location data structure that includes heading and additional GPS metadata
#[derive(Debug, Clone, PartialEq)]
pub struct EnhancedLocationData {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub heading: Option<f64>,  // Course over ground in degrees
    pub timestamp: Option<String>,
    pub fix_quality: Option<u8>,
    pub satellites: Option<u8>,
    pub hdop: Option<f64>,  // Horizontal dilution of precision
}

impl Default for EnhancedLocationData {
    fn default() -> Self {
        EnhancedLocationData {
            latitude: None,
            longitude: None,
            altitude: None,
            speed: None,
            heading: None,
            timestamp: None,
            fix_quality: None,
            satellites: None,
            hdop: None,
        }
    }
}

impl From<EnhancedLocationData> for GpsData {
    fn from(enhanced: EnhancedLocationData) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        GpsData {
            latitude: enhanced.latitude.unwrap_or(0.0),
            longitude: enhanced.longitude.unwrap_or(0.0),
            altitude: enhanced.altitude,
            accuracy: enhanced.hdop,
            heading: enhanced.heading,
            speed: enhanced.speed,
            timestamp,
        }
    }
}

/// Enhanced GNSS parser that supports heading and additional GPS data
pub struct EnhancedGnssParser {
    debug_enabled: bool,
}

impl EnhancedGnssParser {
    pub fn new() -> Self {
        EnhancedGnssParser {
            debug_enabled: std::env::var("GPS_DEBUG").is_ok(),
        }
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug_enabled = enabled;
        self
    }

    pub fn parse_sentence(&self, sentence: &str) -> Option<EnhancedLocationData> {
        if sentence.is_empty() || !sentence.starts_with('$') {
            if self.debug_enabled {
                debug!("[GPS_DEBUG] Invalid sentence format: {}", sentence);
            }
            return None;
        }

        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.is_empty() {
            if self.debug_enabled {
                debug!("[GPS_DEBUG] Empty sentence parts");
            }
            return None;
        }

        let sentence_type = parts[0];
        if self.debug_enabled {
            debug!("[GPS_DEBUG] Parsing sentence type: {}", sentence_type);
        }

        match sentence_type {
            "$GPGGA" | "$GNGGA" => self.parse_gpgga(&parts),
            "$GPRMC" | "$GNRMC" => self.parse_gprmc(&parts),
            "$GPVTG" | "$GNVTG" => self.parse_gpvtg(&parts), // Course and speed
            _ => {
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Unsupported sentence type: {}", sentence_type);
                }
                None
            }
        }
    }

    fn parse_gpgga(&self, parts: &[&str]) -> Option<EnhancedLocationData> {
        if parts.len() < 15 {
            if self.debug_enabled {
                debug!("[GPS_DEBUG] GPGGA sentence too short: {} parts", parts.len());
            }
            return None;
        }

        let mut location = EnhancedLocationData::default();

        // Parse timestamp (field 1)
        if !parts[1].is_empty() {
            location.timestamp = Some(parts[1].to_string());
            if self.debug_enabled {
                debug!("[GPS_DEBUG] Parsed timestamp: {}", parts[1]);
            }
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
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed latitude: {:.6}°", latitude);
                }
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
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed longitude: {:.6}°", longitude);
                }
            }
        }

        // Parse fix quality (field 6)
        if !parts[6].is_empty() {
            if let Ok(quality) = parts[6].parse::<u8>() {
                location.fix_quality = Some(quality);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed fix quality: {}", quality);
                }
            }
        }

        // Parse number of satellites (field 7)
        if !parts[7].is_empty() {
            if let Ok(sats) = parts[7].parse::<u8>() {
                location.satellites = Some(sats);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed satellites: {}", sats);
                }
            }
        }

        // Parse HDOP (field 8)
        if !parts[8].is_empty() {
            if let Ok(hdop) = parts[8].parse::<f64>() {
                location.hdop = Some(hdop);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed HDOP: {:.2}", hdop);
                }
            }
        }

        // Parse altitude (field 9)
        if !parts[9].is_empty() {
            if let Ok(alt) = parts[9].parse::<f64>() {
                location.altitude = Some(alt);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed altitude: {:.1} m", alt);
                }
            }
        }

        Some(location)
    }

    fn parse_gprmc(&self, parts: &[&str]) -> Option<EnhancedLocationData> {
        if parts.len() < 12 {
            if self.debug_enabled {
                debug!("[GPS_DEBUG] GPRMC sentence too short: {} parts", parts.len());
            }
            return None;
        }

        let mut location = EnhancedLocationData::default();

        // Parse timestamp (field 1)
        if !parts[1].is_empty() {
            location.timestamp = Some(parts[1].to_string());
            if self.debug_enabled {
                debug!("[GPS_DEBUG] Parsed timestamp: {}", parts[1]);
            }
        }

        // Check if data is valid (field 2)
        if parts[2] != "A" {
            if self.debug_enabled {
                debug!("[GPS_DEBUG] GPRMC data invalid: {}", parts[2]);
            }
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
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed latitude: {:.6}°", latitude);
                }
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
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed longitude: {:.6}°", longitude);
                }
            }
        }

        // Parse speed (field 7) - in knots
        if !parts[7].is_empty() {
            if let Ok(speed_knots) = parts[7].parse::<f64>() {
                location.speed = Some(speed_knots);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed speed: {:.1} knots", speed_knots);
                }
            }
        }

        // Parse course/heading (field 8) - in degrees
        if !parts[8].is_empty() {
            if let Ok(course) = parts[8].parse::<f64>() {
                location.heading = Some(course);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed course: {:.1}°", course);
                }
            }
        }

        Some(location)
    }

    fn parse_gpvtg(&self, parts: &[&str]) -> Option<EnhancedLocationData> {
        if parts.len() < 9 {
            if self.debug_enabled {
                debug!("[GPS_DEBUG] GPVTG sentence too short: {} parts", parts.len());
            }
            return None;
        }

        let mut location = EnhancedLocationData::default();

        // Parse true course (field 1)
        if !parts[1].is_empty() {
            if let Ok(course) = parts[1].parse::<f64>() {
                location.heading = Some(course);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed true course: {:.1}°", course);
                }
            }
        }

        // Parse speed in knots (field 5)
        if !parts[5].is_empty() {
            if let Ok(speed_knots) = parts[5].parse::<f64>() {
                location.speed = Some(speed_knots);
                if self.debug_enabled {
                    debug!("[GPS_DEBUG] Parsed speed: {:.1} knots", speed_knots);
                }
            }
        }

        Some(location)
    }
}

/// GPYes device provider that abstracts the GPS hardware
pub struct GpyesProvider {
    device_paths: Vec<String>,
    baud_rate: u32,
    parser: EnhancedGnssParser,
    is_running: Arc<Mutex<bool>>,
    debug_enabled: bool,
}

impl GpyesProvider {
    pub fn new() -> Self {
        let debug_enabled = std::env::var("GPS_DEBUG").is_ok();
        
        GpyesProvider {
            device_paths: vec![
                "/dev/tty.usbmodem2101".to_string(),
                "/dev/cu.usbmodem2101".to_string(),
                "/dev/ttyUSB0".to_string(),
                "/dev/ttyACM0".to_string(),
            ],
            baud_rate: 9600,
            parser: EnhancedGnssParser::new().with_debug(debug_enabled),
            is_running: Arc::new(Mutex::new(false)),
            debug_enabled,
        }
    }

    pub fn with_device_paths(mut self, paths: Vec<String>) -> Self {
        self.device_paths = paths;
        self
    }

    pub fn with_baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug_enabled = enabled;
        self.parser = self.parser.with_debug(enabled);
        self
    }

    /// Start streaming GPS data from the device
    pub fn start_streaming(&self) -> Result<mpsc::Receiver<GpsData>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = mpsc::channel(100);
        let device_paths = self.device_paths.clone();
        let baud_rate = self.baud_rate;
        let parser = EnhancedGnssParser::new().with_debug(self.debug_enabled);
        let is_running = Arc::clone(&self.is_running);
        let debug_enabled = self.debug_enabled;

        // Set running flag
        {
            let mut running = is_running.lock().unwrap();
            *running = true;
        }

        thread::spawn(move || {
            if debug_enabled {
                info!("[GPS_DEBUG] Starting GPYes device streaming thread");
            }

            // Try to connect to any available device
            let mut connected = false;
            for device_path in &device_paths {
                if debug_enabled {
                    info!("[GPS_DEBUG] Trying to connect to: {}", device_path);
                }

                match serialport::new(device_path, baud_rate)
                    .timeout(Duration::from_millis(1000))
                    .open()
                {
                    Ok(mut port) => {
                        info!("Successfully connected to GPS device at {}", device_path);
                        connected = true;

                        let mut reader = BufReader::new(port.as_mut());
                        let mut line = String::new();

                        loop {
                            // Check if we should stop
                            {
                                let running = is_running.lock().unwrap();
                                if !*running {
                                    if debug_enabled {
                                        info!("[GPS_DEBUG] Stopping GPS streaming thread");
                                    }
                                    break;
                                }
                            }

                            line.clear();
                            match reader.read_line(&mut line) {
                                Ok(0) => {
                                    warn!("GPS device disconnected (EOF)");
                                    break;
                                }
                                Ok(_) => {
                                    let sentence = line.trim();
                                    if !sentence.is_empty() {
                                        if debug_enabled {
                                            debug!("[GPS_DEBUG] Raw NMEA: {}", sentence);
                                        }

                                        if let Some(location) = parser.parse_sentence(sentence) {
                                            // Only send if we have valid position data
                                            if location.latitude.is_some() && location.longitude.is_some() {
                                                let gps_data: GpsData = location.into();
                                                if debug_enabled {
                                                    debug!("[GPS_DEBUG] Sending GPS data: lat={:.6}, lon={:.6}, heading={:?}", 
                                                           gps_data.latitude, gps_data.longitude, gps_data.heading);
                                                }
                                                
                                                if let Err(e) = tx.blocking_send(gps_data) {
                                                    error!("Failed to send GPS data: {}", e);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    match e.kind() {
                                        ErrorKind::TimedOut => {
                                            if debug_enabled {
                                                debug!("[GPS_DEBUG] Read timeout - continuing...");
                                            }
                                            continue;
                                        }
                                        ErrorKind::Interrupted => {
                                            continue;
                                        }
                                        _ => {
                                            error!("Error reading from GPS device: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                    Err(e) => {
                        if debug_enabled {
                            warn!("[GPS_DEBUG] Failed to connect to {}: {}", device_path, e);
                        }
                    }
                }
            }

            if !connected {
                error!("Could not connect to any GPS device. Tried paths: {:?}", device_paths);
            }

            // Clear running flag
            {
                let mut running = is_running.lock().unwrap();
                *running = false;
            }
        });

        Ok(rx)
    }

    /// Stop streaming GPS data
    pub fn stop_streaming(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
        info!("GPS streaming stopped");
    }

    /// Check if the provider is currently streaming
    pub fn is_streaming(&self) -> bool {
        let running = self.is_running.lock().unwrap();
        *running
    }
}

impl Default for GpyesProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_parser_creation() {
        let parser = EnhancedGnssParser::new();
        // Parser should be created successfully
    }

    #[test]
    fn test_parse_gprmc_with_heading() {
        let parser = EnhancedGnssParser::new();

        // GPRMC sentence with course data (084.4 degrees)
        let sentence = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.latitude.is_some());
        assert!(location.longitude.is_some());
        assert!(location.speed.is_some());
        assert!(location.heading.is_some());

        // Check specific values
        assert!((location.latitude.unwrap() - 48.1173).abs() < 0.001);
        assert!((location.longitude.unwrap() - 11.5167).abs() < 0.001);
        assert!((location.speed.unwrap() - 22.4).abs() < 0.1);
        assert!((location.heading.unwrap() - 84.4).abs() < 0.1);
    }

    #[test]
    fn test_parse_gpvtg_sentence() {
        let parser = EnhancedGnssParser::new();

        // GPVTG sentence with course and speed
        let sentence = "$GPVTG,054.7,T,034.4,M,005.5,N,010.2,K*48";

        let result = parser.parse_sentence(sentence);
        assert!(result.is_some());

        let location = result.unwrap();
        assert!(location.heading.is_some());
        assert!(location.speed.is_some());

        // Check specific values
        assert!((location.heading.unwrap() - 54.7).abs() < 0.1);
        assert!((location.speed.unwrap() - 5.5).abs() < 0.1);
    }

    #[test]
    fn test_enhanced_location_to_gps_data_conversion() {
        let enhanced = EnhancedLocationData {
            latitude: Some(48.1173),
            longitude: Some(11.5167),
            altitude: Some(545.4),
            speed: Some(22.4),
            heading: Some(84.4),
            timestamp: Some("123519".to_string()),
            fix_quality: Some(1),
            satellites: Some(8),
            hdop: Some(0.9),
        };

        let gps_data: GpsData = enhanced.into();
        assert_eq!(gps_data.latitude, 48.1173);
        assert_eq!(gps_data.longitude, 11.5167);
        assert_eq!(gps_data.altitude, Some(545.4));
        assert_eq!(gps_data.speed, Some(22.4));
        assert_eq!(gps_data.heading, Some(84.4));
        assert_eq!(gps_data.accuracy, Some(0.9));
    }

    #[test]
    fn test_gpyes_provider_creation() {
        let provider = GpyesProvider::new();
        assert!(!provider.is_streaming());
        assert_eq!(provider.baud_rate, 9600);
        assert!(!provider.device_paths.is_empty());
    }

    #[test]
    fn test_gpyes_provider_configuration() {
        let provider = GpyesProvider::new()
            .with_device_paths(vec!["/dev/test".to_string()])
            .with_baud_rate(115200)
            .with_debug(true);

        assert_eq!(provider.device_paths, vec!["/dev/test"]);
        assert_eq!(provider.baud_rate, 115200);
        assert!(provider.debug_enabled);
    }
}