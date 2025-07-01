use crate::GameState;
use bevy::prelude::*;
use super::cluster::setup_instrument_cluster;
use super::instruments::{YachtData, update_yacht_data, update_instrument_displays};
use super::yacht_systems::create_yacht_systems;
use crate::core::system_manager::{SystemManager, SystemManagerPlugin};

pub struct PlayerPlugin;

/// This plugin handles the futuristic yacht instrument cluster
/// Instrument cluster is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<YachtData>()
            .add_plugins(SystemManagerPlugin)
            .add_systems(OnEnter(GameState::Playing), (setup_instrument_cluster, initialize_yacht_systems))
            .add_systems(
                Update, 
                (update_yacht_data, update_instrument_displays)
                    .run_if(in_state(GameState::Playing))
            );
    }
}

/// Initialize yacht systems in the SystemManager
fn initialize_yacht_systems(mut system_manager: ResMut<SystemManager>) {
    let systems = create_yacht_systems();
    for system in systems {
        system_manager.register_system(system);
    }
}
