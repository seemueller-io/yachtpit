#![allow(clippy::type_complexity)]


mod core;
mod ui;
mod services;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use crate::core::{ActionsPlugin, SystemManagerPlugin};
use crate::core::system_manager::SystemManager;
use crate::ui::{LoadingPlugin, MenuPlugin, GpsMapPlugin};
use crate::services::GpsServicePlugin;
use systems::{PlayerPlugin, setup_instrument_cluster, get_vessel_systems, CompassGauge, SpeedGauge, VesselData, update_vessel_data_with_gps};
use crate::ui::GpsMapState;
#[cfg(target_arch = "wasm32")]
use systems::GeoPlugin;

// See https://bevy-cheatbook.github.io/programming/states.html
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

/// Initialize systems in the SystemManager
fn initialize_vessel_systems(mut system_manager: ResMut<SystemManager>) {
    let systems = get_vessel_systems();
    for system in systems {
        system_manager.register_system(system);
    }
}

/// Update compass gauge with real GPS heading data
fn update_compass_heading(
    gps_map_state: Res<GpsMapState>,
    mut compass_query: Query<&mut Text, With<CompassGauge>>,
) {
    for mut text in compass_query.iter_mut() {
        // Update compass display with real GPS heading
        text.0 = format!("{:03.0}°", gps_map_state.vessel_heading);
    }
}

/// Update speed gauge with real GPS speed data
fn update_speed_gauge(
    gps_map_state: Res<GpsMapState>,
    mut speed_query: Query<&mut Text, With<SpeedGauge>>,
) {
    for mut text in speed_query.iter_mut() {
        // Update speed display with real GPS speed data
        // Only update if the text contains a decimal point (to avoid updating labels)
        if text.0.contains('.') {
            text.0 = format!("{:.1}", gps_map_state.vessel_speed);
        }
    }
}

/// Update vessel data with real GPS data for consistent system displays
fn update_vessel_data_with_real_gps(
    gps_map_state: Res<GpsMapState>,
    vessel_data: ResMut<VesselData>,
    time: Res<Time>,
) {
    // Use real GPS data from GpsMapState
    let gps_data = Some((gps_map_state.vessel_speed, gps_map_state.vessel_heading));
    update_vessel_data_with_gps(vessel_data, time, gps_data);
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            GpsMapPlugin,
            GpsServicePlugin,
            ActionsPlugin,
            SystemManagerPlugin,
            PlayerPlugin,
        ))

        .add_systems(OnEnter(GameState::Playing), (setup_instrument_cluster, initialize_vessel_systems))
        .add_systems(Update, (
            update_compass_heading,
            update_speed_gauge,
            update_vessel_data_with_real_gps,
        ).run_if(in_state(GameState::Playing)));

        #[cfg(target_arch = "wasm32")]
        {
            app.add_plugins(GeoPlugin);
        }

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
            ));
        }
    }
}
