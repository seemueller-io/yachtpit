use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use tower_http::cors::CorsLayer;
use crate::ais::{AisStreamManager, AppState};

mod ais;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the shared state with the AIS stream manager
    let state = AppState {
        ais_stream_manager: Arc::new(AisStreamManager::new()),
    };

    // Create and start the Axum HTTP server
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("AIS server running on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(ais::shutdown_signal())
        .await?;

    Ok(())
}

// Create the Axum router
fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ais", get(crate::ais::get_ais_data))
        .route("/ws", get(crate::ais::websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}