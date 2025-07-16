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
use systems::{PlayerPlugin, setup_instrument_cluster, get_vessel_systems};
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
        .add_systems(Startup, simple_test_render);

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

fn simple_test_render(mut commands: Commands) {
    info!("Simple test render: spawning camera and test rectangle");

    // Spawn a 2D camera
    commands.spawn((Camera2d, Msaa::Off));

    // Spawn a simple colored rectangle to test rendering
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0), // Red color
            custom_size: Some(Vec2::new(200.0, 200.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));

    info!("Simple test render: entities spawned");
}
