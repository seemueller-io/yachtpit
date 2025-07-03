use bevy::prelude::*;
use components::{setup_instrument_cluster, VesselData, update_vessel_data, update_instrument_displays};
use super::vessel_systems::{create_vessel_systems, VesselSystem};

pub struct PlayerPlugin;

/// bind domain to bevy
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VesselData>()
            .add_systems(
                Update, 
                (update_vessel_data, update_instrument_displays)
            );
    }
}

/// Setup function called by the main app
pub fn setup_instrument_cluster_system() -> impl Fn(Commands) {
    setup_instrument_cluster
}

/// Initialize marine systems - returns the systems for registration
pub fn get_vessel_systems() -> Vec<Box<dyn VesselSystem>> {
    create_vessel_systems()
}
