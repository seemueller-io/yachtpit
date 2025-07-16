// src/geo_plugin.rs
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[derive(Resource, Default)]
pub struct UserLocation {
    pub lat: f64,
    pub lon: f64,
    pub accuracy: f64,
    pub fresh: bool,
}

#[derive(Resource)]
pub struct LocationData {
    pub data: Arc<Mutex<Option<(f64, f64, f64)>>>,
}

impl Default for LocationData {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct GeoPlugin;
impl Plugin for GeoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserLocation>()
           .init_resource::<LocationData>();

        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(Startup, request_location)
               .add_systems(Update, update_location);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn request_location(location_data: Res<LocationData>) {
    let window = match window() {
        Some(w) => w,
        None => {
            warn!("No window object available");
            return;
        }
    };

    let geo = match window.navigator().geolocation() {
        Ok(g) => g,
        Err(_) => {
            warn!("Geolocation not available");
            return;
        }
    };

    let data_clone = location_data.data.clone();

    let success = Closure::<dyn FnMut(web_sys::Position)>::new(move |pos: web_sys::Position| {
        let c: web_sys::Coordinates = pos.coords();
        if let Ok(mut data) = data_clone.lock() {
            *data = Some((c.latitude(), c.longitude(), c.accuracy()));
        }
    });

    let error = Closure::<dyn FnMut(web_sys::PositionError)>::new(move |err: web_sys::PositionError| {
        match err.code() {
            1 => {
                warn!("Geolocation permission denied. This may be due to:");
                warn!("  - User denied location access");
                warn!("  - Insecure connection (HTTP instead of HTTPS)");
                warn!("  - Browser security settings");
                warn!("  Consider serving over HTTPS for geolocation access");
            },
            2 => {
                warn!("Geolocation position unavailable: {}", err.message());
            },
            3 => {
                warn!("Geolocation timeout: {}", err.message());
            },
            _ => {
                warn!("Geolocation error: {} (code: {})", err.message(), err.code());
            }
        }
    });

    // watch_position keeps updating; get_current_position is one‑shot
    match geo.watch_position_with_error_callback(success.as_ref().unchecked_ref(), Some(error.as_ref().unchecked_ref())) {
        Ok(_) => {
            success.forget(); // leak the closure so it lives forever
            error.forget(); // leak the error closure too
        }
        Err(e) => {
            warn!("Failed to start watching position: {:?}", e);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn update_location(mut loc: ResMut<UserLocation>, location_data: Res<LocationData>) {
    if let Ok(mut data) = location_data.data.lock() {
        if let Some((lat, lon, accuracy)) = data.take() {
            loc.lat = lat;
            loc.lon = lon;
            loc.accuracy = accuracy;
            loc.fresh = true;
        }
    }
}
