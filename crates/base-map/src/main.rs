use my_crate::build_app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, build_app()).await?;
    Ok(())
}