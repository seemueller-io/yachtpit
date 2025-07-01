pub mod actions;
pub mod audio;
pub mod loading;
pub mod menu;
pub mod system_manager;

pub use actions::ActionsPlugin;
pub use audio::InternalAudioPlugin;
pub use loading::LoadingPlugin;
pub use menu::MenuPlugin;
pub use system_manager::{SystemManagerPlugin, SystemManager};
pub use models::{YachtSystem, SystemInteraction, SystemStatus};
