// src/lib.rs
use axum::{Router, routing::get};
use tower_http::{
    trace::TraceLayer,
};

// a helper for integration tests or other binaries
pub fn build_router() -> Router {
    Router::new()
        .route("/status", get(|| async { "OK" }))
        .layer(TraceLayer::new_for_http())
}