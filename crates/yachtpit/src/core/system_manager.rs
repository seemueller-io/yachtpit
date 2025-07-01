//! Higher-level abstraction for managing yacht systems and their interactions
//! 
//! This module provides a unified approach to handling different yacht systems
//! (GPS, Radar, AIS, etc.) with common patterns for state management, UI updates,
//! and user interactions.

use bevy::prelude::*;
use std::collections::HashMap;
use models::{YachtSystem, SystemInteraction, SystemStatus, instruments::YachtData};

/// Resource for managing all yacht systems
#[derive(Resource)]
pub struct SystemManager {
    systems: HashMap<String, Box<dyn YachtSystem>>,
    active_system: Option<String>,
    system_order: Vec<String>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            active_system: None,
            system_order: Vec::new(),
        }
    }

    /// Register a new yacht system
    pub fn register_system(&mut self, system: Box<dyn YachtSystem>) {
        let id = system.id().to_string();
        self.system_order.push(id.clone());
        self.systems.insert(id, system);
    }

    /// Get the currently active system
    pub fn active_system(&self) -> Option<&dyn YachtSystem> {
        self.active_system.as_ref()
            .and_then(|id| self.systems.get(id))
            .map(|system| system.as_ref())
    }

    /// Set the active system by ID
    pub fn set_active_system(&mut self, system_id: &str) -> bool {
        if self.systems.contains_key(system_id) {
            self.active_system = Some(system_id.to_string());
            true
        } else {
            false
        }
    }

    /// Get all registered systems in order
    pub fn get_systems(&self) -> Vec<&dyn YachtSystem> {
        self.system_order.iter()
            .filter_map(|id| self.systems.get(id))
            .map(|system| system.as_ref())
            .collect()
    }

    /// Update all systems
    pub fn update_systems(&mut self, yacht_data: &YachtData, time: &Time) {
        for system in self.systems.values_mut() {
            system.update(yacht_data, time);
        }
    }

    /// Handle interaction with a specific system
    pub fn handle_system_interaction(&mut self, system_id: &str, interaction: SystemInteraction) -> bool {
        if let Some(system) = self.systems.get_mut(system_id) {
            system.handle_interaction(interaction)
        } else {
            false
        }
    }

    /// Get system by ID
    pub fn get_system(&self, system_id: &str) -> Option<&dyn YachtSystem> {
        self.systems.get(system_id).map(|s| s.as_ref())
    }

    /// Get mutable system by ID
    pub fn get_system_mut(&mut self, system_id: &str) -> Option<&mut Box<dyn YachtSystem>> {
        self.systems.get_mut(system_id)
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Component for marking UI elements as system indicators
#[derive(Component)]
pub struct SystemIndicator {
    pub system_id: String,
}

/// Component for marking the main system display area
#[derive(Component)]
pub struct SystemDisplayArea;

/// Plugin for the system manager
pub struct SystemManagerPlugin;

impl Plugin for SystemManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SystemManager>()
            .add_systems(
                Update,
                (
                    update_all_systems,
                    handle_system_indicator_interactions,
                    update_system_display_content,
                ).run_if(in_state(crate::GameState::Playing))
            );
    }
}

/// System to update all yacht systems
fn update_all_systems(
    mut system_manager: ResMut<SystemManager>,
    yacht_data: Res<models::instruments::YachtData>,
    time: Res<Time>,
) {
    system_manager.update_systems(&yacht_data, &time);
}

/// System to handle interactions with system indicator buttons
fn handle_system_indicator_interactions(
    mut system_manager: ResMut<SystemManager>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &SystemIndicator),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, indicator) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                system_manager.set_active_system(&indicator.system_id);
                system_manager.handle_system_interaction(
                    &indicator.system_id, 
                    SystemInteraction::Select
                );
                *background_color = BackgroundColor(Color::linear_rgb(0.0, 0.3, 0.5));
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::linear_rgb(0.15, 0.15, 0.2));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15));
            }
        }
    }
}

/// System to update the main display area with active system content
fn update_system_display_content(
    system_manager: Res<SystemManager>,
    mut display_query: Query<&mut Text, With<SystemDisplayArea>>,
    yacht_data: Res<models::instruments::YachtData>,
) {
    if let Ok(mut text) = display_query.single_mut() {
        if let Some(active_system) = system_manager.active_system() {
            text.0 = active_system.render_display(&yacht_data);
        } else {
            text.0 = "Select a system above to view details".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::instruments::YachtData;

    struct MockSystem {
        id: &'static str,
        name: &'static str,
        status: SystemStatus,
    }

    impl YachtSystem for MockSystem {
        fn id(&self) -> &'static str { self.id }
        fn display_name(&self) -> &'static str { self.name }
        fn update(&mut self, _yacht_data: &YachtData, _time: &Time) {}
        fn render_display(&self, _yacht_data: &YachtData) -> String {
            format!("Mock system: {}", self.name)
        }
        fn handle_interaction(&mut self, _interaction: SystemInteraction) -> bool { true }
        fn status(&self) -> SystemStatus { self.status.clone() }
    }

    #[test]
    fn test_system_manager_registration() {
        let mut manager = SystemManager::new();
        let mock_system = Box::new(MockSystem {
            id: "test",
            name: "Test System",
            status: SystemStatus::Active,
        });

        manager.register_system(mock_system);
        assert!(manager.get_system("test").is_some());
        assert_eq!(manager.get_systems().len(), 1);
    }

    #[test]
    fn test_active_system_management() {
        let mut manager = SystemManager::new();
        let mock_system = Box::new(MockSystem {
            id: "test",
            name: "Test System",
            status: SystemStatus::Active,
        });

        manager.register_system(mock_system);
        assert!(manager.set_active_system("test"));
        assert!(manager.active_system().is_some());
        assert_eq!(manager.active_system().unwrap().id(), "test");
    }
}
