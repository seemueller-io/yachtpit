use bevy::prelude::*;
use super::cluster::setup_instrument_cluster;
use super::instruments::{YachtData, update_yacht_data, update_instrument_displays};
use super::yacht_systems::{create_yacht_systems, YachtSystem};

pub struct PlayerPlugin;

/// This plugin handles the futuristic yacht instrument cluster
/// The main app should handle state management and system registration
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<YachtData>()
            .add_systems(
                Update, 
                (update_yacht_data, update_instrument_displays)
            );
    }
}

/// Setup function for instrument cluster - to be called by the main app
pub fn setup_instrument_cluster_system() -> impl Fn(Commands) {
    setup_instrument_cluster
}

/// Initialize yacht systems - returns the systems for registration
pub fn get_yacht_systems() -> Vec<Box<dyn YachtSystem>> {
    create_yacht_systems()
}
