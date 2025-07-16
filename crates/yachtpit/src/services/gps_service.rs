use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Resource, Default)]
pub struct GpsService {
    pub current_position: Option<GpsData>,
    pub is_enabled: bool,
    pub last_update: f64,
}

impl GpsService {
    pub fn new() -> Self {
        Self {
            current_position: None,
            is_enabled: false,
            last_update: 0.0,
        }
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
        info!("GPS service enabled");
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
        info!("GPS service disabled");
    }

    pub fn update_position(&mut self, gps_data: GpsData) {
        self.current_position = Some(gps_data.clone());
        self.last_update = gps_data.timestamp;
        info!("GPS position updated: lat={:.6}, lon={:.6}", gps_data.latitude, gps_data.longitude);
    }

    pub fn get_current_position(&self) -> Option<&GpsData> {
        self.current_position.as_ref()
    }
}

// Native GPS implementation - Mock implementation for demonstration
// TODO: Replace with real GPS hardware access (e.g., using gpsd, CoreLocation, etc.)
#[cfg(not(target_arch = "wasm32"))]
pub fn start_native_gps_tracking(mut gps_service: ResMut<GpsService>, time: Res<Time>) {
    use std::time::{SystemTime, UNIX_EPOCH};

    if !gps_service.is_enabled {
        return;
    }

    // Mock GPS data that simulates realistic movement
    // In a real implementation, this would read from GPS hardware
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    // Only update every 2 seconds to simulate realistic GPS update rate
    if timestamp - gps_service.last_update < 2.0 {
        return;
    }

    // Simulate GPS coordinates around Monaco with realistic movement
    let base_lat = 43.7384;
    let base_lon = 7.4246;
    let time_factor = time.elapsed_secs() * 0.1;

    // Simulate a boat moving in a realistic pattern
    let lat_offset = (time_factor.sin() * 0.002) as f64;
    let lon_offset = (time_factor.cos() * 0.003) as f64;

    let gps_data = GpsData {
        latitude: base_lat + lat_offset,
        longitude: base_lon + lon_offset,
        altitude: Some(0.0), // Sea level
        accuracy: Some(3.0), // 3 meter accuracy
        heading: Some(((time_factor * 20.0) % 360.0) as f64),
        speed: Some(5.2), // 5.2 knots
        timestamp,
    };

    gps_service.update_position(gps_data);
}

// Web GPS implementation using geolocation API
// For web platforms, we'll use a simplified approach that requests position periodically
#[cfg(target_arch = "wasm32")]
pub fn start_web_gps_tracking(mut gps_service: ResMut<GpsService>, time: Res<Time>) {
    if !gps_service.is_enabled {
        return;
    }

    // Use Bevy's time instead of std::time for WASM compatibility
    let current_time = time.elapsed_secs_f64();

    // Only try to get GPS every 5 seconds to avoid overwhelming the browser
    if current_time - gps_service.last_update < 5.0 {
        return;
    }

    // For now, use mock data for web as well
    // TODO: Implement proper web geolocation API integration using channels or events
    let time_factor = time.elapsed_secs() * 0.1;
    let base_lat = 43.7384;
    let base_lon = 7.4246;

    let lat_offset = (time_factor.sin() * 0.001) as f64;
    let lon_offset = (time_factor.cos() * 0.002) as f64;

    let gps_data = GpsData {
        latitude: base_lat + lat_offset,
        longitude: base_lon + lon_offset,
        altitude: Some(0.0),
        accuracy: Some(5.0), // Slightly less accurate on web
        heading: Some(((time_factor * 15.0) % 360.0) as f64),
        speed: Some(4.8), // Slightly different speed for web
        timestamp: current_time,
    };

    gps_service.update_position(gps_data.clone());
    info!("Web GPS position updated: lat={:.6}, lon={:.6}", gps_data.latitude, gps_data.longitude);
}

pub struct GpsServicePlugin;

impl Plugin for GpsServicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpsService>()
            .add_systems(Update, (
                #[cfg(not(target_arch = "wasm32"))]
                start_native_gps_tracking,
                #[cfg(target_arch = "wasm32")]
                start_web_gps_tracking,
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_gps_service_initialization() {
        let mut gps_service = GpsService::new();
        assert!(!gps_service.is_enabled);
        assert!(gps_service.current_position.is_none());

        gps_service.enable();
        assert!(gps_service.is_enabled);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_gps_data_update() {
        let mut gps_service = GpsService::new();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let test_gps_data = GpsData {
            latitude: 43.7384,
            longitude: 7.4246,
            altitude: Some(0.0),
            accuracy: Some(3.0),
            heading: Some(45.0),
            speed: Some(5.2),
            timestamp,
        };

        gps_service.update_position(test_gps_data.clone());

        let current_pos = gps_service.get_current_position().unwrap();
        assert_eq!(current_pos.latitude, 43.7384);
        assert_eq!(current_pos.longitude, 7.4246);
        assert_eq!(current_pos.speed, Some(5.2));
        assert_eq!(current_pos.heading, Some(45.0));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_gps_data_update_wasm() {
        let mut gps_service = GpsService::new();

        // Use a mock timestamp for WASM testing
        let timestamp = 1234567890.0;

        let test_gps_data = GpsData {
            latitude: 43.7384,
            longitude: 7.4246,
            altitude: Some(0.0),
            accuracy: Some(3.0),
            heading: Some(45.0),
            speed: Some(5.2),
            timestamp,
        };

        gps_service.update_position(test_gps_data.clone());

        let current_pos = gps_service.get_current_position().unwrap();
        assert_eq!(current_pos.latitude, 43.7384);
        assert_eq!(current_pos.longitude, 7.4246);
        assert_eq!(current_pos.speed, Some(5.2));
        assert_eq!(current_pos.heading, Some(45.0));
    }

    #[test]
    fn test_gps_service_enable_disable() {
        let mut gps_service = GpsService::new();

        // Test initial state
        assert!(!gps_service.is_enabled);

        // Test enable
        gps_service.enable();
        assert!(gps_service.is_enabled);

        // Test disable
        gps_service.disable();
        assert!(!gps_service.is_enabled);
    }
}
