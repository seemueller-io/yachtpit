#![allow(clippy::type_complexity)]

// Components crate for yacht pit application
// This crate contains reusable UI and game components

// Shared modules
pub mod ui;
pub mod theme;
pub mod composition;

// Individual component modules
pub mod speed_gauge;
pub mod depth_gauge;
pub mod compass_gauge;
pub mod engine_status;
pub mod navigation_display;
pub mod yacht_data;
pub mod instrument_cluster;
pub mod gps_indicator;
pub mod radar_indicator;
pub mod ais_indicator;
pub mod system_display;
pub mod wind_display;

// Re-export everything
pub use ui::*;
pub use theme::*;
pub use composition::*;
pub use speed_gauge::*;
pub use depth_gauge::*;
pub use compass_gauge::*;
pub use engine_status::*;
pub use navigation_display::*;
pub use yacht_data::*;
pub use instrument_cluster::*;
pub use gps_indicator::*;
pub use radar_indicator::*;
pub use ais_indicator::*;
pub use system_display::*;
pub use wind_display::*;
