mod geolocate;

use axum::response::IntoResponse;
use axum::routing::post;
// src/lib.rs
use axum::{routing::get, Json, Router};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

// ===== JSON coming back from the browser =====
#[derive(Deserialize, Debug)]
struct LocationPayload {
    id:  String,
    lat: f64,
    lon: f64,
}

// ===== POST /api/location handler =====
async fn receive_location(axum::Json(p): Json<LocationPayload>) -> impl IntoResponse {
    println!("Got location: {p:?}");
    axum::http::StatusCode::OK
}


// a helper for integration tests or other binaries
pub fn build_router() -> Router {
    Router::new()
        .route("/status", get(|| async { "OK" }))
        .route("/geolocate", get(geolocate::geolocate))
        .route("/geolocate", post(receive_location))
        .layer(TraceLayer::new_for_http())
}
