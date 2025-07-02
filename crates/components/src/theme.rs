use bevy::prelude::*;
use bevy::color::Color;

pub const BACKGROUND_COLOR_PRIMARY: Color = Color::linear_rgb(0.05, 0.05, 0.1);
pub const BACKGROUND_COLOR_SECONDARY: Color = Color::linear_rgb(0.1, 0.1, 0.15);
pub const BACKGROUND_COLOR_ACCENT: Color = Color::linear_rgb(0.1, 0.15, 0.2);

pub const BORDER_COLOR_PRIMARY: Color = Color::linear_rgb(0.0, 0.8, 1.0);
pub const BORDER_COLOR_SECONDARY: Color = Color::linear_rgb(0.8, 0.4, 0.0);
pub const BORDER_COLOR_TERTIARY: Color = Color::linear_rgb(0.4, 0.4, 0.6);

pub const TEXT_COLOR_PRIMARY: Color = Color::linear_rgb(0.0, 0.8, 1.0);
pub const TEXT_COLOR_SECONDARY: Color = Color::linear_rgb(0.6, 0.6, 0.6);
pub const TEXT_COLOR_SUCCESS: Color = Color::linear_rgb(0.0, 1.0, 0.0);
pub const TEXT_COLOR_WARNING: Color = Color::linear_rgb(0.8, 0.4, 0.0);
pub const TEXT_COLOR_DANGER: Color = Color::linear_rgb(0.8, 0.0, 0.0);

pub const FONT_SIZE_SMALL: f32 = 10.0;
pub const FONT_SIZE_NORMAL: f32 = 14.0;
pub const FONT_SIZE_LARGE: f32 = 32.0;

pub const PADDING_DEFAULT: f32 = 20.0;
pub const BORDER_WIDTH_DEFAULT: f32 = 2.0;

pub fn create_node_style(width: Val, height: Val, direction: FlexDirection) -> Node {
    Node {
        width,
        height,
        flex_direction: direction,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}