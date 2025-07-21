// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use yachtpit::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;
use tokio::process::Command;

#[cfg(not(target_arch = "wasm32"))]
use bevy_webview_wry::WebviewWryPlugin;


#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    
    launch_bevy();
}


#[cfg(not(target_arch = "wasm32"))]
fn launch_bevy() {
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Bind to canvas included in `index.html`
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .add_systems(Update, start_ais_server)
        .add_plugins(WebviewWryPlugin::default())
        .run();
}


#[cfg(target_arch = "wasm32")]
fn launch_bevy() {
    {
        // Add console logging for WASM debugging
        console_error_panic_hook::set_once();

        info!("Starting WASM Bevy application");

        App::new()
            .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Dark gray background instead of transparent
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            // Bind to canvas included in `index.html`
                            canvas: Some("#yachtpit-canvas".to_owned()),
                            fit_canvas_to_parent: true,
                            // Tells wasm not to override default event handling, like F5 and Ctrl+R
                            prevent_default_event_handling: false,
                            ..default()
                        }),
                        ..default()
                    })
                    .set(AssetPlugin {
                        meta_check: AssetMetaCheck::Never,
                        ..default()
                    }),
            )
            .add_plugins(GamePlugin)
            .add_systems(Startup, || {
                info!("WASM Bevy startup system running");
            })
            .run();
    }
}

fn start_ais_server() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        if let Ok(mut cmd) = Command::new("cargo")
            .current_dir("../ais-server")
            .arg("run").arg("--release").spawn() {
            let _ = cmd.wait().await;
        }
    });
}


// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) -> Result {
    let primary_entity = primary_window.single()?;
    let Some(primary) = windows.get_window(primary_entity) else {
        return Err(BevyError::from("No primary window!"));
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };

    Ok(())
}
