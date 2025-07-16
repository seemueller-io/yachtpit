// src/lib.rs
pub mod routes;
pub mod state;

use axum::{Router, routing::get};

// a helper for integration tests or other binaries
pub fn build_app() -> Router {
    Router::new().route("/", get(|| async { "OK" }))
}