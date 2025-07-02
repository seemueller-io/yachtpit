use bevy::prelude::*;
use super::theme::*;
use super::composition::*;
use super::speed_gauge::SpeedGauge;
use super::depth_gauge::DepthGauge;
use super::compass_gauge::CompassGauge;
use super::engine_status::EngineStatus;
use super::navigation_display::NavigationDisplay;
use super::gps_indicator::GpsIndicator;
use super::radar_indicator::RadarIndicator;
use super::ais_indicator::AisIndicator;
use super::system_display::SystemDisplay;
use super::wind_display::WindDisplay;

/// Main instrument cluster component
#[derive(Component)]
pub struct InstrumentCluster;

/// Sets up the main instrument cluster UI using composable components
pub fn setup_instrument_cluster(mut commands: Commands) {
    // Spawn camera since we're bypassing the menu system
    commands.spawn((Camera2d, Msaa::Off));

    // Create main container using composition
    commands.spawn((
        main_container_node(),
        BackgroundColor(BACKGROUND_COLOR_PRIMARY),
        InstrumentCluster,
    ))
    .with_children(|parent| {
        // Top row - Main navigation and speed (60% height)
        parent.spawn(row_container_node(60.0, 20.0))
        .with_children(|row| {
            // Speed Gauge
            row.spawn((
                circular_gauge_node(),
                BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                BorderColor(BORDER_COLOR_PRIMARY),
                SpeedGauge,
            ))
            .with_children(|gauge| {
                gauge.spawn(create_text("SPEED", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                gauge.spawn(create_text("12.5", FONT_SIZE_LARGE, TEXT_COLOR_SUCCESS));
                gauge.spawn(create_text("KTS", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
            });

            // Central Navigation Display
            row.spawn((
                navigation_display_node(),
                BackgroundColor(BACKGROUND_COLOR_ACCENT),
                BorderColor(BORDER_COLOR_PRIMARY),
                NavigationDisplay,
            ))
            .with_children(|nav| {
                nav.spawn(create_text("NAVIGATION", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
                nav.spawn((
                    create_text("045°", FONT_SIZE_LARGE, TEXT_COLOR_PRIMARY).0,
                    create_text("045°", FONT_SIZE_LARGE, TEXT_COLOR_PRIMARY).1,
                    create_text("045°", FONT_SIZE_LARGE, TEXT_COLOR_PRIMARY).2,
                    CompassGauge,
                ));
                nav.spawn(create_text("HEADING", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
            });

            // Depth Gauge
            row.spawn((
                circular_gauge_node(),
                BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                BorderColor(BORDER_COLOR_PRIMARY),
                DepthGauge,
            ))
            .with_children(|gauge| {
                gauge.spawn(create_text("DEPTH", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                gauge.spawn(create_text("15.2", FONT_SIZE_LARGE, Color::linear_rgb(0.0, 1.0, 0.8)));
                gauge.spawn(create_text("M", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
            });
        });

        // Bottom row - Engine and system status (40% height)
        parent.spawn(row_container_node(40.0, 20.0))
        .with_children(|row| {
            // Engine Status Panel
            row.spawn((
                status_panel_node(200.0, 150.0),
                BackgroundColor(BACKGROUND_COLOR_PRIMARY),
                BorderColor(BORDER_COLOR_PRIMARY),
                EngineStatus,
            ))
            .with_children(|panel| {
                panel.spawn(create_text("ENGINE", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
                panel.spawn(create_text("82°C", FONT_SIZE_LARGE, TEXT_COLOR_SUCCESS));
                panel.spawn(create_text("TEMP NORMAL", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
            });

            // System Status Grid
            row.spawn((
                status_panel_node(250.0, 150.0),
                BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                BorderColor(BORDER_COLOR_SECONDARY),
            ))
            .with_children(|grid| {
                grid.spawn(create_text("SYSTEMS", 12.0, TEXT_COLOR_SECONDARY));

                // Fuel Level Bar
                grid.spawn(progress_bar_node())
                .with_children(|bar| {
                    bar.spawn(create_text("FUEL", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                    bar.spawn(progress_bar_background_node())
                    .with_children(|bg| {
                        bg.spawn((
                            progress_bar_fill_node(75.0),
                            BackgroundColor(TEXT_COLOR_SUCCESS),
                        ));
                    });
                    bar.spawn(create_text("75%", FONT_SIZE_SMALL, TEXT_COLOR_SUCCESS));
                });

                // Battery Level Bar
                grid.spawn(progress_bar_node())
                .with_children(|bar| {
                    bar.spawn(create_text("BATT", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                    bar.spawn(progress_bar_background_node())
                    .with_children(|bg| {
                        bg.spawn((
                            progress_bar_fill_node(88.0),
                            BackgroundColor(TEXT_COLOR_SUCCESS),
                        ));
                    });
                    bar.spawn(create_text("88%", FONT_SIZE_SMALL, TEXT_COLOR_SUCCESS));
                });

                // System Indicators Row
                grid.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ))
                .with_children(|indicators| {
                    // GPS Indicator
                    indicators.spawn((
                        Button,
                        system_indicator_node(),
                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                        BorderColor(BORDER_COLOR_SECONDARY),
                        GpsIndicator,
                    ))
                    .with_children(|indicator| {
                        indicator.spawn(create_text("🛰️", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
                        indicator.spawn(create_text("GPS", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                    });

                    // RADAR Indicator
                    indicators.spawn((
                        Button,
                        system_indicator_node(),
                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                        BorderColor(BORDER_COLOR_SECONDARY),
                        RadarIndicator,
                    ))
                    .with_children(|indicator| {
                        indicator.spawn(create_text("📡", FONT_SIZE_NORMAL, Color::linear_rgb(0.0, 1.0, 0.0)));
                        indicator.spawn(create_text("RADAR", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                    });

                    // AIS Indicator
                    indicators.spawn((
                        Button,
                        system_indicator_node(),
                        BackgroundColor(BACKGROUND_COLOR_SECONDARY),
                        BorderColor(BORDER_COLOR_SECONDARY),
                        AisIndicator,
                    ))
                    .with_children(|indicator| {
                        indicator.spawn(create_text("🚢", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
                        indicator.spawn(create_text("AIS", FONT_SIZE_SMALL, TEXT_COLOR_PRIMARY));
                    });
                });
            });

            // Wind Information
            row.spawn((
                status_panel_node(200.0, 150.0),
                BackgroundColor(BACKGROUND_COLOR_ACCENT),
                BorderColor(BORDER_COLOR_PRIMARY),
                WindDisplay,
            ))
            .with_children(|panel| {
                panel.spawn(create_text("WIND", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
                panel.spawn(create_text("8.3 KTS", FONT_SIZE_NORMAL, TEXT_COLOR_SUCCESS));
                panel.spawn(create_text("120° REL", FONT_SIZE_NORMAL, TEXT_COLOR_PRIMARY));
            });
        });

        // System Display Area
        parent.spawn((
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
            display.spawn(create_text("Select a system above to view details", FONT_SIZE_SMALL, TEXT_COLOR_SECONDARY));
        });
    });
}