use crate::GameState;
use bevy::prelude::*;

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, click_play_button.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component, Clone)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
    pressed: Color,
}

// Neumorphic color palette for luxury design
struct NeumorphicColors;

impl NeumorphicColors {
    // Base surface color - soft gray with warm undertones
    const SURFACE: Color = Color::linear_rgb(0.88, 0.90, 0.92);
    
    // Primary button colors with depth
    const PRIMARY_NORMAL: Color = Color::linear_rgb(0.85, 0.87, 0.90);
    const PRIMARY_HOVERED: Color = Color::linear_rgb(0.90, 0.92, 0.95);
    const PRIMARY_PRESSED: Color = Color::linear_rgb(0.80, 0.82, 0.85);
    
    // Secondary button colors (more subtle)
    const SECONDARY_NORMAL: Color = Color::linear_rgb(0.86, 0.88, 0.91);
    const SECONDARY_HOVERED: Color = Color::linear_rgb(0.88, 0.90, 0.93);
    const SECONDARY_PRESSED: Color = Color::linear_rgb(0.82, 0.84, 0.87);
    
    // Text colors for contrast
    const TEXT_PRIMARY: Color = Color::linear_rgb(0.25, 0.30, 0.35);
    const TEXT_SECONDARY: Color = Color::linear_rgb(0.45, 0.50, 0.55);
    const TEXT_ACCENT: Color = Color::linear_rgb(0.20, 0.45, 0.75);
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: NeumorphicColors::PRIMARY_NORMAL,
            hovered: NeumorphicColors::PRIMARY_HOVERED,
            pressed: NeumorphicColors::PRIMARY_PRESSED,
        }
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands) {
    info!("menu");
    commands.spawn((Camera2d, Msaa::Off));
    
    // Set neumorphic background
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(NeumorphicColors::SURFACE),
    ));
    
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            let button_colors = ButtonColors::default();
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        margin: UiRect::all(Val::Px(8.0)),
                        ..Default::default()
                    },
                    BackgroundColor(button_colors.normal),
                    BorderColor(Color::linear_rgb(0.82, 0.84, 0.87)),
                    BorderRadius::all(Val::Px(16.0)),
                    button_colors,
                    ChangeState(GameState::Playing),
                ))
                .with_child((
                    Text::new("▶ PLAY"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(NeumorphicColors::TEXT_PRIMARY),
                ));
        });
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                bottom: Val::Px(5.),
                width: Val::Percent(100.),
                position_type: PositionType::Absolute,
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            let secondary_button_colors = ButtonColors {
                normal: NeumorphicColors::SECONDARY_NORMAL,
                hovered: NeumorphicColors::SECONDARY_HOVERED,
                pressed: NeumorphicColors::SECONDARY_PRESSED,
            };
            
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(8.)),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::horizontal(Val::Px(8.0)),
                        ..Default::default()
                    },
                    BackgroundColor(secondary_button_colors.normal),
                    BorderColor(Color::linear_rgb(0.80, 0.82, 0.85)),
                    BorderRadius::all(Val::Px(12.0)),
                    secondary_button_colors,
                    OpenLink("https://bevyengine.org"),
                ))
                .with_child((
                    Text::new("🚀 Made with Bevy"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(NeumorphicColors::TEXT_SECONDARY),
                ));
                
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(8.)),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::horizontal(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(secondary_button_colors.normal),
                    BorderColor(Color::linear_rgb(0.80, 0.82, 0.85)),
                    BorderRadius::all(Val::Px(12.0)),
                    ButtonColors {
                        normal: NeumorphicColors::SECONDARY_NORMAL,
                        hovered: NeumorphicColors::SECONDARY_HOVERED,
                        pressed: NeumorphicColors::SECONDARY_PRESSED,
                    },
                    OpenLink("https://github.com/NiklasEi/bevy_game_template"),
                ))
                .with_child((
                    Text::new("📖 Open Source"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(NeumorphicColors::TEXT_SECONDARY),
                ));
        });
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors, change_state, open_link) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Apply pressed state visual feedback
                *color = button_colors.pressed.into();
                
                // Handle button actions
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                // Smooth transition to hovered state
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                // Return to normal state
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}