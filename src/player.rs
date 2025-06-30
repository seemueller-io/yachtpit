use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct InstrumentCluster;

#[derive(Component)]
pub struct SpeedGauge;

#[derive(Component)]
pub struct DepthGauge;

#[derive(Component)]
pub struct CompassGauge;

#[derive(Component)]
pub struct EngineStatus;

#[derive(Component)]
pub struct NavigationDisplay;

#[derive(Component)]
pub struct GpsIndicator;

#[derive(Component)]
pub struct RadarIndicator;

#[derive(Component)]
pub struct AisIndicator;

#[derive(Component)]
pub struct SystemDisplay;

#[derive(Resource, Default)]
pub struct SelectedSystem {
    pub current: Option<SystemType>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemType {
    Gps,
    Radar,
    Ais,
}

#[derive(Resource)]
pub struct YachtData {
    pub speed: f32,           // knots
    pub depth: f32,           // meters
    pub heading: f32,         // degrees
    pub engine_temp: f32,     // celsius
    pub fuel_level: f32,      // percentage
    pub battery_level: f32,   // percentage
    pub wind_speed: f32,      // knots
    pub wind_direction: f32,  // degrees
}

impl Default for YachtData {
    fn default() -> Self {
        Self {
            speed: 12.5,
            depth: 15.2,
            heading: 045.0,
            engine_temp: 82.0,
            fuel_level: 75.0,
            battery_level: 88.0,
            wind_speed: 8.3,
            wind_direction: 120.0,
        }
    }
}

/// This plugin handles the futuristic yacht instrument cluster
/// Instrument cluster is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<YachtData>()
            .init_resource::<SelectedSystem>()
            .add_systems(OnEnter(GameState::Playing), setup_instrument_cluster)
            .add_systems(
                Update, 
                (update_yacht_data, update_instrument_displays, handle_system_interactions, update_system_display)
                    .run_if(in_state(GameState::Playing))
            );
    }
}

fn setup_instrument_cluster(mut commands: Commands) {
    // Spawn camera since we're bypassing the menu system
    commands.spawn((Camera2d, Msaa::Off));

    // Main container for the instrument cluster
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::linear_rgb(0.05, 0.05, 0.1)),
            InstrumentCluster,
        ))
        .with_children(|parent| {
            // Top row - Main navigation and speed
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(60.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|row| {
                    // Speed Gauge
                    row.spawn((
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(180.0),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15)),
                        BorderColor(Color::linear_rgb(0.0, 0.8, 1.0)),
                        SpeedGauge,
                    ))
                    .with_children(|gauge| {
                        gauge.spawn((
                            Text::new("SPEED"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 0.8, 1.0)),
                        ));
                        gauge.spawn((
                            Text::new("12.5"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 1.0, 0.8)),
                        ));
                        gauge.spawn((
                            Text::new("KTS"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                        ));
                    });

                    // Central Navigation Display
                    row.spawn((
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(300.0),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.1, 0.15, 0.2)),
                        BorderColor(Color::linear_rgb(0.0, 0.8, 1.0)),
                        NavigationDisplay,
                    ))
                    .with_children(|nav| {
                        nav.spawn((
                            Text::new("NAVIGATION"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 0.8, 1.0)),
                        ));
                        nav.spawn((
                            Text::new("045°"),
                            TextFont {
                                font_size: 48.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 1.0, 0.8)),
                            CompassGauge,
                        ));
                        nav.spawn((
                            Text::new("HEADING"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                        ));
                    });

                    // Depth Gauge
                    row.spawn((
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(180.0),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15)),
                        BorderColor(Color::linear_rgb(0.0, 0.8, 1.0)),
                        DepthGauge,
                    ))
                    .with_children(|gauge| {
                        gauge.spawn((
                            Text::new("DEPTH"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 0.8, 1.0)),
                        ));
                        gauge.spawn((
                            Text::new("15.2"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 1.0, 0.8)),
                        ));
                        gauge.spawn((
                            Text::new("M"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                        ));
                    });
                });

            // Bottom row - Engine and system status
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(40.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|row| {
                    // Engine Status Panel
                    row.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(150.0),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15)),
                        BorderColor(Color::linear_rgb(0.8, 0.4, 0.0)),
                        EngineStatus,
                    ))
                    .with_children(|engine| {
                        engine.spawn((
                            Text::new("ENGINE"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.8, 0.4, 0.0)),
                        ));
                        engine.spawn((
                            Text::new("82°C"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 1.0, 0.0)),
                        ));
                        engine.spawn((
                            Text::new("TEMP NORMAL"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                        ));
                    });

                    // System Status Grid
                    row.spawn((
                        Node {
                            width: Val::Px(250.0),
                            height: Val::Px(150.0),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceEvenly,
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.08, 0.08, 0.12)),
                        BorderColor(Color::linear_rgb(0.4, 0.4, 0.6)),
                    ))
                    .with_children(|grid| {
                        grid.spawn((
                            Text::new("SYSTEMS"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.4, 0.4, 0.8)),
                        ));

                        // Fuel Level Bar
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|bar| {
                            bar.spawn((
                                Text::new("FUEL"),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::linear_rgb(0.7, 0.7, 0.7)),
                            ));

                            bar.spawn((
                                Node {
                                    width: Val::Px(80.0),
                                    height: Val::Px(8.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::linear_rgb(0.2, 0.2, 0.2)),
                                BorderColor(Color::linear_rgb(0.4, 0.4, 0.4)),
                            ))
                            .with_children(|bar_bg| {
                                bar_bg.spawn((
                                    Node {
                                        width: Val::Percent(75.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::linear_rgb(0.0, 0.8, 0.0)),
                                ));
                            });

                            bar.spawn((
                                Text::new("75%"),
                                TextFont {
                                    font_size: 9.0,
                                    ..default()
                                },
                                TextColor(Color::linear_rgb(0.8, 0.8, 0.8)),
                            ));
                        });

                        // Battery Level Bar
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|bar| {
                            bar.spawn((
                                Text::new("BATTERY"),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::linear_rgb(0.7, 0.7, 0.7)),
                            ));

                            bar.spawn((
                                Node {
                                    width: Val::Px(80.0),
                                    height: Val::Px(8.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::linear_rgb(0.2, 0.2, 0.2)),
                                BorderColor(Color::linear_rgb(0.4, 0.4, 0.4)),
                            ))
                            .with_children(|bar_bg| {
                                bar_bg.spawn((
                                    Node {
                                        width: Val::Percent(88.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::linear_rgb(0.0, 0.6, 1.0)),
                                ));
                            });

                            bar.spawn((
                                Text::new("88%"),
                                TextFont {
                                    font_size: 9.0,
                                    ..default()
                                },
                                TextColor(Color::linear_rgb(0.8, 0.8, 0.8)),
                            ));
                        });

                        // System Status Indicators
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|indicators| {
                            // GPS Indicator
                            indicators.spawn((
                                Button,
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(8.0)),
                                    width: Val::Px(60.0),
                                    height: Val::Px(40.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15)),
                                BorderColor(Color::linear_rgb(0.3, 0.3, 0.4)),
                                GpsIndicator,
                            ))
                            .with_children(|indicator| {
                                indicator.spawn((
                                    Text::new("🛰️"),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::linear_rgb(0.0, 1.0, 0.0)),
                                ));
                                indicator.spawn((
                                    Text::new("GPS"),
                                    TextFont {
                                        font_size: 8.0,
                                        ..default()
                                    },
                                    TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                                ));
                            });

                            // RADAR Indicator
                            indicators.spawn((
                                Button,
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(8.0)),
                                    width: Val::Px(60.0),
                                    height: Val::Px(40.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15)),
                                BorderColor(Color::linear_rgb(0.3, 0.3, 0.4)),
                                RadarIndicator,
                            ))
                            .with_children(|indicator| {
                                indicator.spawn((
                                    Text::new("📡"),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::linear_rgb(0.0, 1.0, 0.0)),
                                ));
                                indicator.spawn((
                                    Text::new("RADAR"),
                                    TextFont {
                                        font_size: 8.0,
                                        ..default()
                                    },
                                    TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                                ));
                            });

                            // AIS Indicator
                            indicators.spawn((
                                Button,
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(8.0)),
                                    width: Val::Px(60.0),
                                    height: Val::Px(40.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.15)),
                                BorderColor(Color::linear_rgb(0.3, 0.3, 0.4)),
                                AisIndicator,
                            ))
                            .with_children(|indicator| {
                                indicator.spawn((
                                    Text::new("🚢"),
                                    TextFont {
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::linear_rgb(0.8, 0.0, 0.0)),
                                ));
                                indicator.spawn((
                                    Text::new("AIS"),
                                    TextFont {
                                        font_size: 8.0,
                                        ..default()
                                    },
                                    TextColor(Color::linear_rgb(0.6, 0.6, 0.6)),
                                ));
                            });
                        });
                    });

                    // Wind Information
                    row.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(150.0),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::linear_rgb(0.1, 0.15, 0.1)),
                        BorderColor(Color::linear_rgb(0.0, 0.8, 0.4)),
                    ))
                    .with_children(|wind| {
                        wind.spawn((
                            Text::new("WIND"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 0.8, 0.4)),
                        ));
                        wind.spawn((
                            Text::new("8.3 KTS"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.0, 1.0, 0.6)),
                        ));
                        wind.spawn((
                            Text::new("120° REL"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::linear_rgb(0.6, 0.8, 0.6)),
                        ));
                    });
                });

            // System Display Area
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::linear_rgb(0.05, 0.05, 0.1)),
                    BorderColor(Color::linear_rgb(0.2, 0.2, 0.3)),
                    SystemDisplay,
                ))
                .with_children(|display| {
                    display.spawn((
                        Text::new("Select a system above to view details"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::linear_rgb(0.5, 0.5, 0.6)),
                    ));
                });
        });
}


fn update_yacht_data(mut yacht_data: ResMut<YachtData>, time: Res<Time>) {
    let t = time.elapsed_secs();

    // Simulate realistic yacht data with some variation
    yacht_data.speed = 12.5 + (t * 0.3).sin() * 2.0;
    yacht_data.depth = 15.2 + (t * 0.1).sin() * 3.0;
    yacht_data.heading = (yacht_data.heading + time.delta_secs() * 5.0) % 360.0;
    yacht_data.engine_temp = 82.0 + (t * 0.2).sin() * 3.0;
    yacht_data.wind_speed = 8.3 + (t * 0.4).sin() * 1.5;
    yacht_data.wind_direction = (yacht_data.wind_direction + time.delta_secs() * 10.0) % 360.0;

    // Slowly drain fuel and battery (very slowly for demo purposes)
    yacht_data.fuel_level = (yacht_data.fuel_level - time.delta_secs() * 0.01).max(0.0);
    yacht_data.battery_level = (yacht_data.battery_level - time.delta_secs() * 0.005).max(0.0);
}

fn update_instrument_displays(
    yacht_data: Res<YachtData>,
    mut speed_query: Query<&mut Text, (With<SpeedGauge>, Without<DepthGauge>, Without<CompassGauge>)>,
    mut depth_query: Query<&mut Text, (With<DepthGauge>, Without<SpeedGauge>, Without<CompassGauge>)>,
    mut compass_query: Query<&mut Text, (With<CompassGauge>, Without<SpeedGauge>, Without<DepthGauge>)>,
) {
    // Update speed display
    for mut text in speed_query.iter_mut() {
        if text.0.contains('.') {
            text.0 = format!("{:.1}", yacht_data.speed);
        }
    }

    // Update depth display
    for mut text in depth_query.iter_mut() {
        if text.0.contains('.') {
            text.0 = format!("{:.1}", yacht_data.depth);
        }
    }

    // Update compass display
    for mut text in compass_query.iter_mut() {
        if text.0.contains('°') {
            text.0 = format!("{:03.0}°", yacht_data.heading);
        }
    }
}

fn handle_system_interactions(
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

fn update_system_display(
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
                text.0 = "Select a system above to view details".to_string();
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

    #[test]
    fn test_yacht_data_default() {
        let yacht_data = YachtData::default();
        assert_eq!(yacht_data.speed, 12.5);
        assert_eq!(yacht_data.depth, 15.2);
        assert_eq!(yacht_data.heading, 45.0);
        assert_eq!(yacht_data.fuel_level, 75.0);
        assert_eq!(yacht_data.battery_level, 88.0);
    }
}
