#![allow(clippy::type_complexity)]


mod core;
mod ui;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use crate::core::{ActionsPlugin, SystemManagerPlugin};
use crate::core::system_manager::SystemManager;
use crate::ui::{LoadingPlugin, MenuPlugin, GpsMapPlugin};
use systems::{PlayerPlugin, setup_instrument_cluster, get_vessel_systems};

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
            ActionsPlugin,
            SystemManagerPlugin,
            PlayerPlugin,
        ))
            
        .add_systems(OnEnter(GameState::Playing), (setup_instrument_cluster, initialize_vessel_systems));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                LogDiagnosticsPlugin::default(),
            ));
        }
    }
}
