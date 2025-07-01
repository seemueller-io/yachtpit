use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, Element, HtmlElement, Window};

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                // .load_collection::<AudioAssets>()
                // .load_collection::<TextureAssets>(),
        );

        // Add a system to hide the loading indicator when transitioning to the Playing state
        app.add_systems(OnEnter(GameState::Playing), hide_loading_indicator);
    }
}

/// Hides the loading indicator when transitioning to the Playing state
#[cfg(target_arch = "wasm32")]
fn hide_loading_indicator() {
    info!("Hiding loading indicator");

    // Get the window object
    let window = web_sys::window().expect("Failed to get window");

    // Get the document object
    let document = window.document().expect("Failed to get document");

    // Get the loading indicator element
    if let Some(loading_indicator) = document.query_selector(".lds-dual-ring").ok().flatten() {
        // Set its display property to "none" to hide it
        let element = loading_indicator.dyn_into::<HtmlElement>().expect("Failed to cast to HtmlElement");
        element.style().set_property("display", "none").expect("Failed to set style property");
    } else {
        warn!("Could not find loading indicator element");
    }
}

/// No-op implementation for non-wasm32 targets
#[cfg(not(target_arch = "wasm32"))]
fn hide_loading_indicator() {
    info!("Hiding loading indicator (no-op on non-wasm32 targets)");
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)


// #[derive(AssetCollection, Resource)]
// pub struct TextureAssets {
//     #[asset(path = "assets/textures/bevy.png")]
//     pub bevy: Handle<Image>,
//     #[asset(path = "assets/textures/github.png")]
//     pub github: Handle<Image>,
// }
