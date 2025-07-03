use bevy::prelude::*;
use components::{YachtData, GpsIndicator, RadarIndicator, AisIndicator, SystemDisplay};

/// Resource to track which system is currently selected
#[derive(Resource, Default)]
pub struct SelectedSystem {
    pub current: Option<SystemType>,
}

/// Types of navigation systems available
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemType {
    Gps,
    Radar,
    Ais,
}

/// Handles user interactions with system indicator buttons
pub fn handle_system_interactions(
    mut selected_system: ResMut<SelectedSystem>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&GpsIndicator>, Option<&RadarIndicator>, Option<&AisIndicator>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, gps, radar, ais) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if gps.is_some() {
                    selected_system.current = Some(SystemType::Gps);
                    *background_color = BackgroundColor(Color::linear_rgb(0.0, 0.3, 0.5));
                } else if radar.is_some() {
                    selected_system.current = Some(SystemType::Radar);
                    *background_color = BackgroundColor(Color::linear_rgb(0.0, 0.3, 0.5));
                } else if ais.is_some() {
                    selected_system.current = Some(SystemType::Ais);
                    *background_color = BackgroundColor(Color::linear_rgb(0.0, 0.3, 0.5));
                }
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

/// Updates the system display area with detailed information about the selected system
pub fn update_system_display(
    selected_system: Res<SelectedSystem>,
    mut display_query: Query<&mut Text, With<SystemDisplay>>,
    yacht_data: Res<YachtData>,
    time: Res<Time>,
) {
    if let Ok(mut text) = display_query.single_mut() {
        match selected_system.current {
            Some(SystemType::Gps) => {
                text.0 = format!(
                    "GPS NAVIGATION SYSTEM\n\n\
                    Position: 43°38'19.5\"N 1°26'58.3\"W\n\
                    Heading: {:.0}°\n\
                    Speed: {:.1} knots\n\
                    Course Over Ground: {:.0}°\n\
                    Satellites: 12 connected\n\
                    HDOP: 0.8 (Excellent)\n\
                    \n\
                    Next Waypoint: MONACO HARBOR\n\
                    Distance: 127.3 NM\n\
                    ETA: 10h 12m",
                    yacht_data.heading,
                    yacht_data.speed,
                    yacht_data.heading + 5.0
                );
            }
            Some(SystemType::Radar) => {
                let sweep_angle = (time.elapsed_secs() * 60.0) % 360.0;
                text.0 = format!(
                    "RADAR SYSTEM - 12 NM RANGE\n\n\
                    Status: ACTIVE\n\
                    Sweep: {:.0}°\n\
                    Gain: AUTO\n\
                    Sea Clutter: -15 dB\n\
                    Rain Clutter: OFF\n\
                    \n\
                    CONTACTS DETECTED:\n\
                    • Vessel 1: 2.3 NM @ 045° (15 kts)\n\
                    • Vessel 2: 5.7 NM @ 180° (8 kts)\n\
                    • Land Mass: 8.2 NM @ 270°\n\
                    • Buoy: 1.1 NM @ 315°",
                    sweep_angle
                );
            }
            Some(SystemType::Ais) => {
                text.0 = format!(
                    "AIS - AUTOMATIC IDENTIFICATION SYSTEM\n\n\
                    Status: RECEIVING\n\
                    Own Ship MMSI: 123456789\n\
                    \n\
                    NEARBY VESSELS:\n\
                    \n\
                    🛥️ M/Y SERENITY\n\
                    MMSI: 987654321\n\
                    Distance: 2.1 NM @ 045°\n\
                    Speed: 12.5 kts\n\
                    Course: 180°\n\
                    \n\
                    🚢 CARGO VESSEL ATLANTIS\n\
                    MMSI: 456789123\n\
                    Distance: 5.8 NM @ 270°\n\
                    Speed: 18.2 kts\n\
                    Course: 090°\n\
                    \n\
                    ⛵ S/Y WIND DANCER\n\
                    MMSI: 789123456\n\
                    Distance: 1.3 NM @ 135°\n\
                    Speed: 6.8 kts\n\
                    Course: 225°"
                );
            }
            None => {
                text.0 = "".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_type_enum() {
        let gps = SystemType::Gps;
        let radar = SystemType::Radar;
        let ais = SystemType::Ais;

        assert_ne!(gps, radar);
        assert_ne!(radar, ais);
        assert_ne!(ais, gps);
    }

    #[test]
    fn test_selected_system_default() {
        let selected_system = SelectedSystem::default();
        assert_eq!(selected_system.current, None);
    }
}
