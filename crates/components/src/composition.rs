use bevy::prelude::*;

/// Composition utilities for building instrument cluster components
/// This module provides reusable building blocks for creating UI components

/// Creates a circular gauge node bundle
pub fn circular_gauge_node() -> Node {
    Node {
        width: Val::Px(180.0),
        height: Val::Px(180.0),
        border: UiRect::all(Val::Px(2.0)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates a status panel node bundle
pub fn status_panel_node(width: f32, height: f32) -> Node {
    Node {
        width: Val::Px(width),
        height: Val::Px(height),
        border: UiRect::all(Val::Px(1.0)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(10.0)),
        ..default()
    }
}

/// Creates a progress bar container node
pub fn progress_bar_node() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        width: Val::Percent(100.0),
        ..default()
    }
}

/// Creates a progress bar background node
pub fn progress_bar_background_node() -> Node {
    Node {
        width: Val::Px(80.0),
        height: Val::Px(8.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

/// Creates a progress bar fill node
pub fn progress_bar_fill_node(percentage: f32) -> Node {
    Node {
        width: Val::Percent(percentage),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Creates a system indicator button node
pub fn system_indicator_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(8.0)),
        width: Val::Px(60.0),
        height: Val::Px(40.0),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    }
}

/// Creates a navigation display node
pub fn navigation_display_node() -> Node {
    Node {
        width: Val::Px(300.0),
        height: Val::Px(300.0),
        border: UiRect::all(Val::Px(2.0)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates a row container node
pub fn row_container_node(height_percent: f32, padding: f32) -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(height_percent),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(padding)),
        ..default()
    }
}

/// Creates the main container node
pub fn main_container_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

/// Creates a text bundle with specified content, font size, and color
pub fn create_text(content: &str, font_size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(content),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
    )
}
