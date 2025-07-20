use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::services::gpyes_provider::GpyesProvider;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
    pub timestamp: f64,
}

#[derive(Resource)]
pub struct GpsService {
    pub current_position: Option<GpsData>,
    pub is_enabled: bool,
    pub last_update: f64,
    #[cfg(not(target_arch = "wasm32"))]
    pub gpyes_provider: Option<GpyesProvider>,
    #[cfg(not(target_arch = "wasm32"))]
    pub gps_receiver: Option<mpsc::Receiver<GpsData>>,
}

impl Default for GpsService {
    fn default() -> Self {
        Self::new()
    }
}

impl GpsService {
    pub fn new() -> Self {
        GpsService {
            current_position: None,
            is_enabled: false,
            last_update: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            gpyes_provider: None,
            #[cfg(not(target_arch = "wasm32"))]
            gps_receiver: None,
        }
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
        info!("GPS service enabled");
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Initialize GPYes provider if not already done
            if self.gpyes_provider.is_none() {
                let provider = GpyesProvider::new();
                match provider.start_streaming() {
                    Ok(receiver) => {
                        info!("GPYes provider started successfully");
                        self.gps_receiver = Some(receiver);
                        self.gpyes_provider = Some(provider);
                    }
                    Err(e) => {
                        error!("Failed to start GPYes provider: {}", e);
                        // Fall back to mock data if hardware fails
                    }
                }
            }
        }
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
        info!("GPS service disabled");
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Stop GPYes provider if running
            if let Some(provider) = &self.gpyes_provider {
                provider.stop_streaming();
            }
            self.gpyes_provider = None;
            self.gps_receiver = None;
        }
    }

    pub fn update_position(&mut self, gps_data: GpsData) {
        self.current_position = Some(gps_data.clone());
        self.last_update = gps_data.timestamp;
        debug!("GPS position updated: lat={:.6}, lon={:.6}, heading={:?}", 
               gps_data.latitude, gps_data.longitude, gps_data.heading);
    }

    pub fn get_current_position(&self) -> Option<&GpsData> {
        self.current_position.as_ref()
    }
}

// Native GPS implementation using GPYes device
#[cfg(not(target_arch = "wasm32"))]
pub fn start_native_gps_tracking(mut gps_service: ResMut<GpsService>, time: Res<Time>) {
    use std::time::{SystemTime, UNIX_EPOCH};

    if !gps_service.is_enabled {
        return;
    }

    // Try to receive GPS data from GPYes provider
    if let Some(receiver) = &mut gps_service.gps_receiver {
        match receiver.try_recv() {
            Ok(gps_data) => {
                gps_service.update_position(gps_data);
                return;
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                // No new data available, continue with fallback
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                warn!("GPS receiver disconnected, clearing provider");
                gps_service.gpyes_provider = None;
                gps_service.gps_receiver = None;
            }
        }
    }

    // Fallback to mock data if no hardware provider is available
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    // Only update every 2 seconds to avoid spam
    if timestamp - gps_service.last_update < 2.0 {
        return;
    }

    warn!("Using mock GPS data - no hardware provider available");
    
    // Simulate GPS coordinates around Monaco with realistic movement
    let base_lat = 43.7384;
    let base_lon = 7.4246;
    
    // Create some movement pattern
    let time_offset = (timestamp / 10.0).sin() * 0.001;
    let lat_offset = (timestamp / 15.0).cos() * 0.0005;
    
    let mock_gps_data = GpsData {
        latitude: base_lat + time_offset,
        longitude: base_lon + lat_offset,
        altitude: Some(10.0 + (timestamp / 20.0).sin() * 5.0),
        accuracy: Some(3.0),
        heading: Some(((timestamp / 30.0) * 57.2958) % 360.0), // Convert to degrees
        speed: Some(5.0 + (timestamp / 25.0).sin() * 2.0), // 3-7 knots
        timestamp,
    };

    gps_service.update_position(mock_gps_data);
}

// Bevy plugin for GPS service
pub struct GpsServicePlugin;

impl Plugin for GpsServicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpsService>();

        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(Update, start_native_gps_tracking);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_service_creation() {
        let service = GpsService::new();
        assert!(!service.is_enabled);
        assert!(service.current_position.is_none());
        assert_eq!(service.last_update, 0.0);
    }

    #[test]
    fn test_gps_service_enable_disable() {
        let mut service = GpsService::new();
        
        service.enable();
        assert!(service.is_enabled);
        
        service.disable();
        assert!(!service.is_enabled);
    }

    #[test]
    fn test_gps_data_update() {
        let mut service = GpsService::new();
        
        let gps_data = GpsData {
            latitude: 43.7384,
            longitude: 7.4246,
            altitude: Some(10.0),
            accuracy: Some(3.0),
            heading: Some(90.0),
            speed: Some(5.0),
            timestamp: 1234567890.0,
        };
        
        service.update_position(gps_data.clone());
        
        assert!(service.current_position.is_some());
        assert_eq!(service.last_update, 1234567890.0);
        
        let position = service.get_current_position().unwrap();
        assert_eq!(position.latitude, 43.7384);
        assert_eq!(position.longitude, 7.4246);
        assert_eq!(position.heading, Some(90.0));
    }
}