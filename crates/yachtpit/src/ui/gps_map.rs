use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::Window;
use std::collections::HashMap;


#[cfg(not(target_arch = "wasm32"))]
use bevy_webview_wry::prelude::*;

/// Render layer for GPS map entities to isolate them from other cameras
const GPS_MAP_LAYER: usize = 1;

/// Component to mark the GPS map window
#[derive(Component)]
pub struct GpsMapWindow;

/// Component to mark map tiles
#[derive(Component)]
pub struct MapTile {
    pub x: i32,
    pub y: i32,
    pub zoom: u8,
}

/// Resource to manage the GPS map state
#[derive(Resource, Default)]
pub struct GpsMapState {
    pub window_id: Option<Entity>,
    pub center_lat: f64,
    pub center_lon: f64,
    pub zoom_level: u8,
    pub tile_cache: HashMap<String, Handle<Image>>,
}

impl GpsMapState {
    pub fn new() -> Self {
        Self {
            window_id: None,
            center_lat: 43.6377, // Default to Monaco coordinates from GPS system
            center_lon: -1.4497,
            zoom_level: 10,
            tile_cache: HashMap::new(),
        }
    }
}

/// Plugin for GPS map functionality
pub struct GpsMapPlugin;

impl Plugin for GpsMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpsMapState>()
            .add_systems(Update, (handle_gps_map_window_events, update_map_tiles));
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

/// System to update map tiles
fn update_map_tiles(
    mut commands: Commands,
    gps_map_state: Res<GpsMapState>,
    asset_server: Res<AssetServer>,
    map_tiles: Query<Entity, With<MapTile>>,
) {
    if gps_map_state.window_id.is_none() {
        return;
    }

    // For now, we'll create a simple placeholder map
    // In a full implementation, this would fetch actual OSM tiles
    if map_tiles.is_empty() && gps_map_state.window_id.is_some() {
        spawn_placeholder_map(&mut commands, &asset_server);
    }
}

/// Spawn a placeholder map with basic tiles
fn spawn_placeholder_map(commands: &mut Commands, _asset_server: &Res<AssetServer>) {
    // Create a simple grid of colored rectangles to represent map tiles
    for x in -2..=2 {
        for y in -2..=2 {
            let color = if (x + y) % 2 == 0 {
                Color::linear_rgb(0.3, 0.7, 0.3) // Green for land
            } else {
                Color::linear_rgb(0.2, 0.4, 0.8) // Blue for water
            };

            commands.spawn((
                GpsMapWindow,
                Transform::from_translation(Vec3::new(x as f32 * 100.0, y as f32 * 100.0, 0.0)),
                GlobalTransform::default(),
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                },
                MapTile {
                    x: x as i32,
                    y: y as i32,
                    zoom: 10,
                },
                RenderLayers::layer(GPS_MAP_LAYER),
            ));
        }
    }

    // Add a marker for the vessel position
    commands.spawn((
        // Sprite {
        //     color: Color::linear_rgb(1.0, 0.0, 0.0),
        //     custom_size: Some(Vec2::new(20.0, 20.0)),
        //     ..default()
        // },
        // Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        GpsMapWindow,
        RenderLayers::layer(GPS_MAP_LAYER),
    ));
}

/// Function to spawn the GPS map window
pub fn spawn_gps_map_window(commands: &mut Commands, gps_map_state: &mut ResMut<GpsMapState>) {
    if gps_map_state.window_id.is_some() {
        info!("GPS map window already open");
        return;
    }

    info!("Spawning GPS map window");

    // Create a new window for the GPS map
    let window_entity = commands
        .spawn((
            Window {
                title: "GPS Navigation - OpenStreetMap".to_string(),
                resolution: (800.0, 600.0).into(),
                position: bevy::window::WindowPosition::Centered(
                    bevy::window::MonitorSelection::Current,
                ),
                ..default()
            },
            GpsMapWindow,
        ))
        .id();

    // Create a camera for the map window
    commands.spawn((
        Camera2d,
        Camera {
            target: bevy::render::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(
                window_entity,
            )),
            ..default()
        },
        RenderLayers::layer(GPS_MAP_LAYER),
        GpsMapWindow,
    ));

    gps_map_state.window_id = Some(window_entity);


    info!("GPS map window spawned with entity: {:?}", window_entity);


    #[cfg(not(target_arch = "wasm32"))]
    spawn_gps_webview(commands, gps_map_state);
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_gps_webview(commands: &mut Commands, gps_map_state: &mut ResMut<GpsMapState>) {
    if let Some(win) = gps_map_state.window_id {
        commands.entity(win).insert(Webview::Uri(WebviewUri::relative_local(
            // Using the build output of the base-map package
            "packages/base-map/dist/index.html",
        )));
    }
}
