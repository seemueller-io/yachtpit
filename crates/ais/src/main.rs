use crate::ais::start_ais_server;

mod ais;

#[tokio::main]
async fn main() {
    if let Err(e) = start_ais_server().await {
        eprintln!("Server error: {:?}", e);
    }
}