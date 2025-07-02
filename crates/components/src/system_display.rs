use bevy::prelude::*;

/// System display component for showing detailed system information
#[derive(Component)]
pub struct SystemDisplay;

/// Component for marking UI elements as system indicators
#[derive(Component)]
pub struct SystemIndicator {
    pub system_id: String,
}

/// Component for marking the main system display area
#[derive(Component)]
pub struct SystemDisplayArea;
