use bevy::prelude::*;
use bevy::window::Window;
// use bevy_slippy_tiles::*;  // Temporarily disabled due to ehttp compatibility issues

/// Component to mark the GPS map window
#[derive(Component)]
pub struct GpsMapWindow;


/// Resource to manage the GPS map state
#[derive(Resource)]
pub struct GpsMapState {
    pub window_id: Option<Entity>,
    pub center_lat: f64,
    pub center_lon: f64,
    pub zoom_level: u8,
    pub tiles_requested: bool,
}

impl GpsMapState {
    pub fn new() -> Self {
        Self {
            window_id: None,
            center_lat: 43.6377, // Default to Monaco coordinates from GPS system
            center_lon: -1.4497,
            zoom_level: 10,
            tiles_requested: false,
        }
    }
}

/// Plugin for GPS map functionality
pub struct GpsMapPlugin;

impl Plugin for GpsMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GpsMapState::new())
            .add_systems(Update, (
                handle_gps_map_window_events,
                // request_slippy_tiles_system,  // Temporarily disabled due to ehttp compatibility issues
            ));
    }
}

/// System to handle GPS map window events
fn handle_gps_map_window_events(
    mut commands: Commands,
    mut gps_map_state: ResMut<GpsMapState>,
    windows: Query<Entity, With<Window>>,
) {
    // For now, we'll keep this simple and just track the window
    // In a full implementation, we would handle window close events properly
    if let Some(map_window_id) = gps_map_state.window_id {
        // Check if the window still exists
        if windows.get(map_window_id).is_err() {
            gps_map_state.window_id = None;
            info!("GPS map window was closed");
        }
    }
}

/// System to request slippy tiles for the GPS map
// Temporarily disabled due to ehttp compatibility issues
/*
fn request_slippy_tiles_system(
    mut commands: Commands,
    mut gps_map_state: ResMut<GpsMapState>,
    mut download_slippy_tile_events: EventWriter<DownloadSlippyTilesEvent>,
) {
    if gps_map_state.window_id.is_none() {
        return;
    }

    // Request tiles only once when the map is first opened
    if !gps_map_state.tiles_requested {
        // Create slippy tile event
        let slippy_tile_event = DownloadSlippyTilesEvent {
            tile_size: TileSize::Normal,    // Size of tiles - Normal = 256px, Large = 512px
            zoom_level: ZoomLevel::L18,     // Map zoom level (L0 = entire world, L19 = closest)
            coordinates: Coordinates::from_latitude_longitude(gps_map_state.center_lat, gps_map_state.center_lon),
            radius: Radius(2),              // Request surrounding tiles (2 = 25 tiles total)
            use_cache: true,                // Use cached tiles if available
        };
        download_slippy_tile_events.send(slippy_tile_event);
        gps_map_state.tiles_requested = true;

        info!("Requested slippy tiles for GPS map at lat: {}, lon: {}", 
              gps_map_state.center_lat, gps_map_state.center_lon);
    }
}
*/


/// Function to spawn the GPS map window
pub fn spawn_gps_map_window(
    commands: &mut Commands,
    gps_map_state: &mut ResMut<GpsMapState>,
) {
    if gps_map_state.window_id.is_some() {
        info!("GPS map window already open");
        return;
    }

    info!("Spawning GPS map window");

    // Create a new window for the GPS map
    let window_entity = commands.spawn((
        Window {
            title: "GPS Navigation - OpenStreetMap".to_string(),
            resolution: (800.0, 600.0).into(),
            position: bevy::window::WindowPosition::Centered(bevy::window::MonitorSelection::Current),
            ..default()
        },
        GpsMapWindow,
    )).id();

    // Create a camera for the map window
    commands.spawn((
        Camera2d,
        Camera {
            target: bevy::render::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window_entity)),
            ..default()
        },
        GpsMapWindow,
    ));

    gps_map_state.window_id = Some(window_entity);

    info!("GPS map window spawned with entity: {:?}", window_entity);
}
