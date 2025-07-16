use axum_embed::ServeEmbed;
use base_map::build_router;
use rust_embed::RustEmbed;
use tokio::net::TcpListener;

#[derive(RustEmbed, Clone)]
#[folder = "map/dist/"]
struct Assets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let local_address = listener.local_addr()?;
    tracing::info!("Server listening on http://{}", local_address);

    async fn fallback(uri: axum::http::Uri) -> (axum::http::StatusCode, String) {
        (axum::http::StatusCode::NOT_FOUND, format!("No route for {uri}"))
    }

    let serve_assets = ServeEmbed::<Assets>::new();
    let router = build_router();
    let app = router
        .nest_service("/", serve_assets)
        .fallback(fallback);
    
    axum::serve(listener, app).await?;

    Ok(())
}
