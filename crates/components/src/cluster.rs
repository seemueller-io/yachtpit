use bevy::prelude::*;
use super::instruments::*;
use super::theme::*;

#[derive(Component)]
pub struct InstrumentCluster;

#[derive(Component)]
pub struct GpsIndicator;

#[derive(Component)]
pub struct RadarIndicator;

#[derive(Component)]
pub struct AisIndicator;

#[derive(Component)]
pub struct SystemDisplay;

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
            BackgroundColor(BACKGROUND_COLOR_PRIMARY),
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
                        BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                        BorderColor(BORDER_COLOR_PRIMARY),
                        SpeedGauge,
                    ))
                        .with_children(|gauge| {
                            gauge.spawn((
                                Text::new("SPEED"),
                                TextFont {
                                    font_size: FONT_SIZE_SMALL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
                            ));
                            gauge.spawn((
                                Text::new("12.5"),
                                TextFont {
                                    font_size: FONT_SIZE_LARGE,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_SUCCESS),
                            ));
                            gauge.spawn((
                                Text::new("KTS"),
                                TextFont {
                                    font_size: FONT_SIZE_SMALL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
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
                        BackgroundColor(BACKGROUND_COLOR_ACCENT),
                        BorderColor(BORDER_COLOR_PRIMARY),
                        NavigationDisplay,
                    ))
                        .with_children(|nav| {
                            nav.spawn((
                                Text::new("NAVIGATION"),
                                TextFont {
                                    font_size: FONT_SIZE_NORMAL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
                            ));
                            nav.spawn((
                                Text::new("045°"),
                                TextFont {
                                    font_size: FONT_SIZE_LARGE,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
                                CompassGauge,
                            ));
                            nav.spawn((
                                Text::new("HEADING"),
                                TextFont {
                                    font_size: FONT_SIZE_NORMAL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
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
                        BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                        BorderColor(BORDER_COLOR_PRIMARY),
                        DepthGauge,
                    ))
                        .with_children(|gauge| {
                            gauge.spawn((
                                Text::new("DEPTH"),
                                TextFont {
                                    font_size: FONT_SIZE_SMALL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
                            ));
                            gauge.spawn((
                                Text::new("15.2"),
                                TextFont {
                                    font_size: FONT_SIZE_LARGE,
                                    ..default()
                                },
                                TextColor(Color::linear_rgb(0.0, 1.0, 0.8)),
                            ));
                            gauge.spawn((
                                Text::new("M"),
                                TextFont {
                                    font_size: FONT_SIZE_LARGE,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
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
                        BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                        BorderColor(BORDER_COLOR_PRIMARY),
                        EngineStatus,
                    ))
                        .with_children(|engine| {
                            engine.spawn((
                                Text::new("ENGINE"),
                                TextFont {
                                    font_size: FONT_SIZE_NORMAL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
                            ));
                            engine.spawn((
                                Text::new("82°C"),
                                TextFont {
                                    font_size: FONT_SIZE_LARGE,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_SUCCESS),
                            ));
                            engine.spawn((
                                Text::new("TEMP NORMAL"),
                                TextFont {
                                    font_size: FONT_SIZE_SMALL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
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
                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                        BorderColor(BORDER_COLOR_SECONDARY),
                    ))
                        .with_children(|grid| {
                            grid.spawn((
                                Text::new("SYSTEMS"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_SECONDARY),
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
                                        TextColor(TEXT_COLOR_PRIMARY),
                                    ));

                                    bar.spawn((
                                        Node {
                                            width: Val::Px(80.0),
                                            height: Val::Px(8.0),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                                        BorderColor(TEXT_COLOR_PRIMARY),
                                    ))
                                        .with_children(|bar_bg| {
                                            bar_bg.spawn((
                                                Node {
                                                    width: Val::Percent(75.0),
                                                    height: Val::Percent(100.0),
                                                    ..default()
                                                },
                                                BackgroundColor(TEXT_COLOR_SUCCESS),
                                            ));
                                        });

                                    bar.spawn((
                                        Text::new("75%"),
                                        TextFont {
                                            font_size: FONT_SIZE_SMALL,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR_PRIMARY),
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
                                            font_size: FONT_SIZE_SMALL,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR_PRIMARY),
                                    ));

                                    bar.spawn((
                                        Node {
                                            width: Val::Px(80.0),
                                            height: Val::Px(8.0),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                                        BorderColor(BORDER_COLOR_SECONDARY),
                                    ))
                                        .with_children(|bar_bg| {
                                            bar_bg.spawn((
                                                Node {
                                                    width: Val::Percent(88.0),
                                                    height: Val::Percent(100.0),
                                                    ..default()
                                                },
                                                BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                                            ));
                                        });

                                    bar.spawn((
                                        Text::new("88%"),
                                        TextFont {
                                            font_size: FONT_SIZE_SMALL,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR_PRIMARY),
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
                                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                                        BorderColor(BORDER_COLOR_SECONDARY),
                                        GpsIndicator,
                                    ))
                                        .with_children(|indicator| {
                                            indicator.spawn((
                                                Text::new("🛰️"),
                                                TextFont {
                                                    font_size: FONT_SIZE_NORMAL,
                                                    ..default()
                                                },
                                                TextColor(TEXT_COLOR_PRIMARY),
                                            ));
                                            indicator.spawn((
                                                Text::new("GPS"),
                                                TextFont {
                                                    font_size: FONT_SIZE_SMALL,
                                                    ..default()
                                                },
                                                TextColor(TEXT_COLOR_PRIMARY),
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
                                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                                        BorderColor(BORDER_COLOR_SECONDARY),
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
                                                TextColor(TEXT_COLOR_PRIMARY),
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
                                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                                        BorderColor(BORDER_COLOR_SECONDARY),
                                        AisIndicator,
                                    ))
                                        .with_children(|indicator| {
                                            indicator.spawn((
                                                Text::new("🚢"),
                                                TextFont {
                                                    font_size: FONT_SIZE_NORMAL,
                                                    ..default()
                                                },
                                                TextColor(TEXT_COLOR_PRIMARY),
                                            ));
                                            indicator.spawn((
                                                Text::new("AIS"),
                                                TextFont {
                                                    font_size: FONT_SIZE_SMALL,
                                                    ..default()
                                                },
                                                TextColor(TEXT_COLOR_PRIMARY),
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
                        BackgroundColor(BACKGROUND_COLOR_ACCENT),
                        BorderColor(BORDER_COLOR_PRIMARY),
                    ))
                        .with_children(|wind| {
                            wind.spawn((
                                Text::new("WIND"),
                                TextFont {
                                    font_size: FONT_SIZE_NORMAL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
                            ));
                            wind.spawn((
                                Text::new("8.3 KTS"),
                                TextFont {
                                    font_size: FONT_SIZE_NORMAL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_SUCCESS),
                            ));
                            wind.spawn((
                                Text::new("120° REL"),
                                TextFont {
                                    font_size: FONT_SIZE_NORMAL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR_PRIMARY),
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
                    BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                    BorderColor(BORDER_COLOR_PRIMARY),
                    SystemDisplay,
                ))
                .with_children(|display| {
                    display.spawn((
                        Text::new("Select a system above to view details"),
                        TextFont {
                            font_size: FONT_SIZE_SMALL,
                            ..default()
                        },
                        TextColor(TEXT_COLOR_SECONDARY),
                    ));
                });
        });
}