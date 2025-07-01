use bevy::prelude::*;
use super::instruments::*;
use super::systems::*;

#[derive(Component)]
pub struct InstrumentCluster;

/// Sets up the main instrument cluster UI
pub fn setup_instrument_cluster(mut commands: Commands) {
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
