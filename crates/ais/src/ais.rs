use axum::{
    extract::{ws::{Message as WsMessage, WebSocket}, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response}

    ,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::{
    sync::{broadcast, Mutex},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::sync::CancellationToken;
use url::Url;


#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionMessage {
    #[serde(rename = "Apikey")]
    apikey: String,
    #[serde(rename = "BoundingBoxes")]
    bounding_boxes: Vec<Vec<[f64; 2]>>,
    #[serde(rename = "FiltersShipMMSI")]
    filters_ship_mmsi: Vec<String>,
    // Uncomment and add if needed:
    // #[serde(rename = "FilterMessageTypes")]
    // filter_message_types: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct BoundingBoxQuery {
    sw_lat: f64,  // Southwest latitude
    sw_lon: f64,  // Southwest longitude
    ne_lat: f64,  // Northeast latitude
    ne_lon: f64,  // Northeast longitude
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSocketBoundingBox {
    sw_lat: f64,  // Southwest latitude
    sw_lon: f64,  // Southwest longitude
    ne_lat: f64,  // Northeast latitude
    ne_lon: f64,  // Northeast longitude
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bounding_box: Option<WebSocketBoundingBox>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AisResponse {
    message_type: Option<String>,
    mmsi: Option<String>,
    ship_name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timestamp: Option<String>,
    speed_over_ground: Option<f64>,
    course_over_ground: Option<f64>,
    heading: Option<f64>,
    navigation_status: Option<String>,
    ship_type: Option<String>,
    raw_message: Value,
}

// Manages the lifecycle of the upstream AIS stream.
pub struct AisStreamManager {
    state: Mutex<ManagerState>,
}

// The internal state of the manager, protected by a Mutex.
#[derive(Default)]
struct ManagerState {
    tx: Option<broadcast::Sender<AisResponse>>,
    stream_task: Option<JoinHandle<()>>,
    cancellation_token: Option<CancellationToken>,
    client_count: usize,
}

impl AisStreamManager {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(ManagerState::default()),
        }
    }

    // Starts the AIS stream if it's not already running.
    // This is called by the first client that connects.
    async fn start_stream_if_needed(&self) -> broadcast::Sender<AisResponse> {
        let mut state = self.state.lock().await;

        state.client_count += 1;
        println!("Client connected. Total clients: {}", state.client_count);

        if state.stream_task.is_none() {
            println!("Starting new AIS stream...");
            let (tx, _) = broadcast::channel(1000);
            let token = CancellationToken::new();

            let stream_task = tokio::spawn(connect_to_ais_stream_with_broadcast(
                tx.clone(),
                token.clone(),
            ));

            state.tx = Some(tx.clone());
            state.stream_task = Some(stream_task);
            state.cancellation_token = Some(token);
            println!("AIS stream started.");
            tx
        } else {
            // Stream is already running, return the existing sender.
            state.tx.as_ref().unwrap().clone()
        }
    }

    // Stops the AIS stream if no clients are connected.
    async fn stop_stream_if_unneeded(&self) {
        let mut state = self.state.lock().await;

        state.client_count -= 1;
        println!("Client disconnected. Total clients: {}", state.client_count);

        if state.client_count == 0 {
            println!("Last client disconnected. Stopping AIS stream...");
            if let Some(token) = state.cancellation_token.take() {
                token.cancel();
            }
            if let Some(task) = state.stream_task.take() {
                // Wait for the task to finish to ensure clean shutdown.
                let _ = task.await;
            }
            state.tx = None;
            println!("AIS stream stopped.");
        }
    }
}

// An RAII guard to ensure we decrement the client count when a connection is dropped.
struct ConnectionGuard {
    manager: Arc<AisStreamManager>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let manager = self.manager.clone();
        tokio::spawn(async move {
            manager.stop_stream_if_unneeded().await;
        });
    }
}


// Shared state for the application
#[derive(Clone)]
pub struct AppState {
    pub(crate) ais_stream_manager: Arc<AisStreamManager>,
}

// Convert raw AIS message to structured response
fn parse_ais_message(ais_message: &Value) -> AisResponse {
    let message_type = ais_message.get("MessageType")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let metadata = ais_message.get("MetaData");
    let mmsi = metadata
        .and_then(|m| m.get("MMSI"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let ship_name = metadata
        .and_then(|m| m.get("ShipName"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string());

    let latitude = metadata
        .and_then(|m| m.get("latitude"))
        .and_then(|v| v.as_f64());

    let longitude = metadata
        .and_then(|m| m.get("longitude"))
        .and_then(|v| v.as_f64());

    let timestamp = metadata
        .and_then(|m| m.get("time_utc"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract position report data
    let message = ais_message.get("Message");
    let pos_report = message.and_then(|m| m.get("PositionReport"));

    let speed_over_ground = pos_report
        .and_then(|pr| pr.get("Sog"))
        .and_then(|v| v.as_f64());

    let course_over_ground = pos_report
        .and_then(|pr| pr.get("Cog"))
        .and_then(|v| v.as_f64());

    let heading = pos_report
        .and_then(|pr| pr.get("TrueHeading"))
        .and_then(|v| v.as_f64());

    let navigation_status = pos_report
        .and_then(|pr| pr.get("NavigationalStatus"))
        .and_then(|v| v.as_u64())
        .map(|status| match status {
            0 => "Under way using engine",
            1 => "At anchor",
            2 => "Not under command",
            3 => "Restricted manoeuvrability",
            4 => "Constrained by her draught",
            5 => "Moored",
            6 => "Aground",
            7 => "Engaged in fishing",
            8 => "Under way sailing",
            _ => "Other"
        }.to_string());

    // Extract ship type from static data
    let ship_type = message
        .and_then(|m| m.get("StaticDataReport"))
        .and_then(|sdr| sdr.get("ReportB"))
        .and_then(|rb| rb.get("ShipType"))
        .and_then(|v| v.as_u64())
        .map(|st| get_ship_type_description(st).to_string());

    AisResponse {
        message_type,
        mmsi,
        ship_name,
        latitude,
        longitude,
        timestamp,
        speed_over_ground,
        course_over_ground,
        heading,
        navigation_status,
        ship_type,
        raw_message: ais_message.clone(),
    }
}

// HTTP endpoint to get AIS data for a bounding box
pub(crate) async fn get_ais_data(
    Query(params): Query<BoundingBoxQuery>,
    State(_state): State<AppState>,
) -> Result<Json<Vec<AisResponse>>, StatusCode> {
    println!("Received bounding box request: {:?}", params);

    // This remains a placeholder. A full implementation could query a database
    // populated by the AIS stream.

    let response = vec![AisResponse {
        message_type: Some("Info".to_string()),
        mmsi: None,
        ship_name: Some("Bounding Box Query Received".to_string()),
        latitude: Some((params.sw_lat + params.ne_lat) / 2.0),
        longitude: Some((params.sw_lon + params.ne_lon) / 2.0),
        timestamp: Some("Query processed".to_string()),
        speed_over_ground: None,
        course_over_ground: None,
        heading: None,
        navigation_status: Some("Query processed".to_string()),
        ship_type: None,
        raw_message: serde_json::json!({
            "bounding_box": {
                "sw_lat": params.sw_lat,
                "sw_lon": params.sw_lon,
                "ne_lat": params.ne_lat,
                "ne_lon": params.ne_lon
            }
        }),
    }];

    Ok(Json(response))
}


// WebSocket handler for real-time AIS data streaming
pub(crate) async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state.ais_stream_manager))
}

// Function to check if AIS data is within bounding box
fn is_within_bounding_box(ais_data: &AisResponse, bbox: &WebSocketBoundingBox) -> bool {
    if let (Some(lat), Some(lon)) = (ais_data.latitude, ais_data.longitude) {
        lat >= bbox.sw_lat && lat <= bbox.ne_lat &&
            lon >= bbox.sw_lon && lon <= bbox.ne_lon
    } else {
        false // If no coordinates, don't include
    }
}

// Handle individual WebSocket connections
async fn handle_websocket(mut socket: WebSocket, manager: Arc<AisStreamManager>) {
    // This guard ensures that when the function returns (and the connection closes),
    // the client count is decremented.
    let _guard = ConnectionGuard { manager: manager.clone() };

    // Start the stream if it's the first client, and get a sender.
    let ais_tx = manager.start_stream_if_needed().await;
    let mut ais_rx = ais_tx.subscribe();

    // Store bounding box state for this connection
    let mut bounding_box: Option<WebSocketBoundingBox> = None;

    // Send initial connection confirmation
    if socket.send(WsMessage::Text("Connected to AIS stream".to_string())).await.is_err() {
        return;
    }

    // Handle incoming messages and broadcast AIS data
    loop {
        tokio::select! {
             // Handle incoming messages from the client (e.g., to set a bounding box)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        // Try to parse as a command message
                        if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                            if ws_msg.message_type == "set_bounding_box" {
                                if let Some(bbox) = ws_msg.bounding_box {
                                    println!("Setting bounding box: {:?}", bbox);
                                    bounding_box = Some(bbox);
                                } else {
                                    println!("Clearing bounding box");
                                    bounding_box = None;
                                }
                            }
                        } else {
                            // Echo back unrecognized messages
                            if socket.send(WsMessage::Text(format!("Echo: {}", text))).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) => break, // Client disconnected
                    Some(Err(e)) => {
                        println!("WebSocket error: {:?}", e);
                        break;
                    }
                    None => break, // Connection closed
                    _ => {} // Ignore other message types
                }
            }
            // Forward AIS data from the broadcast channel to the client
            ais_data_result = ais_rx.recv() => {
                match ais_data_result {
                    Ok(data) => {
                        // Apply bounding box filter if it exists
                        let should_send = bounding_box.as_ref()
                            .map(|bbox| is_within_bounding_box(&data, bbox))
                            .unwrap_or(true); // Send if no bbox is set

                        if should_send {
                            if let Ok(json_data) = serde_json::to_string(&data) {
                                if socket.send(WsMessage::Text(json_data)).await.is_err() {
                                    // Client is likely disconnected
                                    break;
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        println!("WebSocket client lagged behind by {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // This happens if the sender is dropped, e.g., during stream shutdown.
                        break;
                    }
                }
            }
        }
    }
}



fn print_detailed_ais_message(ais_message: &Value) {
    println!("\n=== AIS MESSAGE DETAILS ===");

    // Print message type
    if let Some(msg_type) = ais_message.get("MessageType") {
        println!("Message Type: {}", msg_type);
    }

    // Print metadata information
    if let Some(metadata) = ais_message.get("MetaData") {
        if let Some(timestamp) = metadata.get("time_utc") {
            println!("Timestamp: {}", timestamp);
        }

        if let Some(mmsi) = metadata.get("MMSI") {
            println!("MMSI: {}", mmsi);
        }

        if let Some(ship_name) = metadata.get("ShipName") {
            println!("Ship Name: {}", ship_name.as_str().unwrap_or("N/A").trim());
        }

        if let Some(lat) = metadata.get("latitude") {
            println!("Latitude: {}", lat);
        }

        if let Some(lon) = metadata.get("longitude") {
            println!("Longitude: {}", lon);
        }
    }

    // Parse message content based on type
    if let Some(message) = ais_message.get("Message") {
        // Handle Position Report messages
        if let Some(pos_report) = message.get("PositionReport") {
            println!("\n--- Position Report Details ---");

            if let Some(sog) = pos_report.get("Sog") {
                println!("Speed Over Ground: {} knots", sog);
            }

            if let Some(cog) = pos_report.get("Cog") {
                println!("Course Over Ground: {}°", cog);
            }

            if let Some(heading) = pos_report.get("TrueHeading") {
                println!("True Heading: {}°", heading);
            }

            if let Some(nav_status) = pos_report.get("NavigationalStatus") {
                let status_text = match nav_status.as_u64().unwrap_or(15) {
                    0 => "Under way using engine",
                    1 => "At anchor",
                    2 => "Not under command",
                    3 => "Restricted manoeuvrability",
                    4 => "Constrained by her draught",
                    5 => "Moored",
                    6 => "Aground",
                    7 => "Engaged in fishing",
                    8 => "Under way sailing",
                    9 => "Reserved for HSC",
                    10 => "Reserved for WIG",
                    11 => "Power-driven vessel towing astern",
                    12 => "Power-driven vessel pushing ahead",
                    13 => "Reserved for future use",
                    14 => "AIS-SART, MOB-AIS, EPIRB-AIS",
                    _ => "Not defined (default)"
                };
                println!("Navigation Status: {} ({})", nav_status, status_text);
            }

            if let Some(rot) = pos_report.get("RateOfTurn") {
                if rot.as_i64().unwrap_or(127) != 127 {
                    println!("Rate of Turn: {}°/min", rot);
                } else {
                    println!("Rate of Turn: Not available");
                }
            }

            if let Some(accuracy) = pos_report.get("PositionAccuracy") {
                println!("Position Accuracy: {}", if accuracy.as_bool().unwrap_or(false) { "High (< 10m)" } else { "Low (> 10m)" });
            }

            if let Some(raim) = pos_report.get("Raim") {
                println!("RAIM: {}", if raim.as_bool().unwrap_or(false) { "In use" } else { "Not in use" });
            }
        }

        // Handle Static Data Report messages
        if let Some(static_report) = message.get("StaticDataReport") {
            println!("\n--- Static Data Report Details ---");

            if let Some(report_a) = static_report.get("ReportA") {
                if let Some(name) = report_a.get("Name") {
                    println!("Vessel Name: {}", name.as_str().unwrap_or("N/A").trim());
                }
            }

            if let Some(report_b) = static_report.get("ReportB") {
                if let Some(call_sign) = report_b.get("CallSign") {
                    let call_sign_str = call_sign.as_str().unwrap_or("").trim();
                    if !call_sign_str.is_empty() {
                        println!("Call Sign: {}", call_sign_str);
                    }
                }

                if let Some(ship_type) = report_b.get("ShipType") {
                    let ship_type_num = ship_type.as_u64().unwrap_or(0);
                    if ship_type_num > 0 {
                        println!("Ship Type: {} ({})", ship_type_num, get_ship_type_description(ship_type_num));
                    }
                }

                if let Some(dimension) = report_b.get("Dimension") {
                    let a = dimension.get("A").and_then(|v| v.as_u64()).unwrap_or(0);
                    let b = dimension.get("B").and_then(|v| v.as_u64()).unwrap_or(0);
                    let c = dimension.get("C").and_then(|v| v.as_u64()).unwrap_or(0);
                    let d = dimension.get("D").and_then(|v| v.as_u64()).unwrap_or(0);

                    if a > 0 || b > 0 || c > 0 || d > 0 {
                        println!("Dimensions: Length {}m ({}m to bow, {}m to stern), Width {}m ({}m to port, {}m to starboard)",
                                 a + b, a, b, c + d, c, d);
                    }
                }
            }
        }

        // Handle Voyage Data messages
        if let Some(voyage_data) = message.get("VoyageData") {
            println!("\n--- Voyage Data Details ---");

            if let Some(destination) = voyage_data.get("Destination") {
                println!("Destination: {}", destination.as_str().unwrap_or("N/A").trim());
            }

            if let Some(eta) = voyage_data.get("Eta") {
                println!("ETA: {:?}", eta);
            }

            if let Some(draught) = voyage_data.get("MaximumStaticDraught") {
                println!("Maximum Draught: {} meters", draught);
            }
        }
    }

    // Print raw message for debugging
    println!("\nRaw JSON: {}", ais_message);
    println!("========================\n");
}


fn get_ship_type_description(ship_type: u64) -> &'static str {
    match ship_type {
        20..=29 => "Wing in ground (WIG)",
        30 => "Fishing",
        31 => "Towing",
        32 => "Towing: length exceeds 200m or breadth exceeds 25m",
        33 => "Dredging or underwater ops",
        34 => "Diving ops",
        35 => "Military ops",
        36 => "Sailing",
        37 => "Pleasure Craft",
        40..=49 => "High speed craft (HSC)",
        50 => "Pilot Vessel",
        51 => "Search and Rescue vessel",
        52 => "Tug",
        53 => "Port Tender",
        54 => "Anti-pollution equipment",
        55 => "Law Enforcement",
        58 => "Medical Transport",
        59 => "Noncombatant ship according to RR Resolution No. 18",
        60..=69 => "Passenger",
        70..=79 => "Cargo",
        80..=89 => "Tanker",
        90..=99 => "Other Type",
        _ => "Unknown"
    }
}


// Connects to the AIS stream and broadcasts messages.
// Shuts down when the cancellation_token is triggered.
async fn connect_to_ais_stream_with_broadcast(
    tx: broadcast::Sender<AisResponse>,
    cancellation_token: CancellationToken,
) {
    loop {
        tokio::select! {
            // Check if the task has been cancelled.
            _ = cancellation_token.cancelled() => {
                println!("Cancellation signal received. Shutting down AIS stream connection.");
                return;
            }
            // Try to connect and process messages.
            result = connect_and_process_ais_stream(&tx, &cancellation_token) => {
                if let Err(e) = result {
                    eprintln!("AIS stream error: {}. Reconnecting in 5 seconds...", e);
                }
                 // If the connection drops, wait before retrying, but still listen for cancellation.
                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {},
                    _ = cancellation_token.cancelled() => {
                         println!("Cancellation signal received during reconnect wait. Shutting down.");
                        return;
                    }
                }
            }
        }
    }
}


async fn connect_and_process_ais_stream(
    tx: &broadcast::Sender<AisResponse>,
    cancellation_token: &CancellationToken
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { // <--- THE FIX IS HERE

    let url = Url::parse("wss://stream.aisstream.io/v0/stream")?;
    let (ws_stream, _) = connect_async(url).await.map_err(|e| format!("WebSocket connection failed: {}", e))?;
    println!("Upstream WebSocket connection to aisstream.io opened.");

    let (mut sender, mut receiver) = ws_stream.split();

    let key = "MDc4YzY5NTdkMGUwM2UzMzQ1Zjc5NDFmOTA1ODg4ZTMyOGQ0MjM0MA==";
    let subscription_message = SubscriptionMessage {
        apikey: STANDARD.decode(key)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default(),
        bounding_boxes: vec![vec![[-90.0, -180.0], [90.0, 180.0]]], // Global coverage
        filters_ship_mmsi: vec![],
    };

    let message_json = serde_json::to_string(&subscription_message)?;
    sender.send(Message::Text(message_json)).await?;
    println!("Upstream subscription message sent.");

    loop {
        tokio::select! {
            // Forward messages from upstream
            message = receiver.next() => {
                match message {
                    Some(Ok(msg)) => {
                        if process_upstream_message(msg, tx).is_err() {
                            // If there's a critical error processing, break to reconnect
                            break;
                        }
                    },
                    Some(Err(e)) => {
                        eprintln!("Upstream WebSocket error: {}", e);
                        return Err(e.into());
                    },
                    None => {
                        println!("Upstream WebSocket connection closed.");
                        return Ok(()); // Connection closed normally
                    }
                }
            }
            // Listen for the shutdown signal
            _ = cancellation_token.cancelled() => {
                println!("Closing upstream WebSocket connection due to cancellation.");
                 let _ = sender.send(Message::Close(None)).await;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn process_upstream_message(
    msg: Message,
    tx: &broadcast::Sender<AisResponse>,
) -> Result<(), ()> {
    let text = match msg {
        Message::Text(text) => text,
        Message::Binary(data) => String::from_utf8_lossy(&data).to_string(),
        Message::Ping(_) | Message::Pong(_) | Message::Close(_) => return Ok(()),
        Message::Frame(_) => return Ok(()),
    };

    if let Ok(ais_message) = serde_json::from_str::<Value>(&text) {
        let parsed_message = parse_ais_message(&ais_message);
        // The broadcast send will fail if there are no receivers, which is fine.
        let _ = tx.send(parsed_message);
    } else {
        eprintln!("Failed to parse JSON from upstream: {}", text);
    }
    Ok(())
}


// Graceful shutdown signal handler
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Signal received, starting graceful shutdown");
}





#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_router;
    use axum_test::TestServer;
    use serde_json::json;

    #[test]
    fn test_get_ship_type_description() {
        assert_eq!(get_ship_type_description(30), "Fishing");
        assert_eq!(get_ship_type_description(31), "Towing");
        assert_eq!(get_ship_type_description(36), "Sailing");
        assert_eq!(get_ship_type_description(37), "Pleasure Craft");
        assert_eq!(get_ship_type_description(60), "Passenger");
        assert_eq!(get_ship_type_description(70), "Cargo");
        assert_eq!(get_ship_type_description(80), "Tanker");
        assert_eq!(get_ship_type_description(999), "Unknown");
    }

    #[test]
    fn test_parse_ais_message_position_report() {
        let ais_message = json!({
            "MessageType": "PositionReport",
            "MetaData": {
                "MMSI": "123456789",
                "ShipName": "TEST VESSEL",
                "latitude": 33.7,
                "longitude": -118.3,
                "time_utc": "2023-01-01T12:00:00Z"
            },
            "Message": {
                "PositionReport": {
                    "Sog": 12.5,
                    "Cog": 180.0,
                    "TrueHeading": 175.0,
                    "NavigationalStatus": 0
                }
            }
        });

        let result = parse_ais_message(&ais_message);

        assert_eq!(result.message_type, Some("PositionReport".to_string()));
        assert_eq!(result.mmsi, Some("123456789".to_string()));
        assert_eq!(result.ship_name, Some("TEST VESSEL".to_string()));
        assert_eq!(result.latitude, Some(33.7));
        assert_eq!(result.longitude, Some(-118.3));
        assert_eq!(result.timestamp, Some("2023-01-01T12:00:00Z".to_string()));
        assert_eq!(result.speed_over_ground, Some(12.5));
        assert_eq!(result.course_over_ground, Some(180.0));
        assert_eq!(result.heading, Some(175.0));
        assert_eq!(result.navigation_status, Some("Under way using engine".to_string()));
    }

    #[test]
    fn test_parse_ais_message_static_data() {
        let ais_message = json!({
            "MessageType": "StaticDataReport",
            "MetaData": {
                "MMSI": "987654321",
                "ShipName": "CARGO SHIP",
                "latitude": 34.0,
                "longitude": -118.0,
                "time_utc": "2023-01-01T13:00:00Z"
            },
            "Message": {
                "StaticDataReport": {
                    "ReportB": {
                        "ShipType": 70
                    }
                }
            }
        });

        let result = parse_ais_message(&ais_message);

        assert_eq!(result.message_type, Some("StaticDataReport".to_string()));
        assert_eq!(result.mmsi, Some("987654321".to_string()));
        assert_eq!(result.ship_name, Some("CARGO SHIP".to_string()));
        assert_eq!(result.ship_type, Some("Cargo".to_string()));
    }

    #[test]
    fn test_parse_ais_message_empty() {
        let ais_message = json!({});
        let result = parse_ais_message(&ais_message);

        assert_eq!(result.message_type, None);
        assert_eq!(result.mmsi, None);
        assert_eq!(result.ship_name, None);
        assert_eq!(result.latitude, None);
        assert_eq!(result.longitude, None);
    }

    #[tokio::test]
    async fn test_get_ais_data_endpoint() {
        // Create test state
        let state = AppState {
            ais_stream_manager: Arc::new(AisStreamManager::new()),
        };

        // Create test server
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();

        // Test valid bounding box request
        let response = server
            .get("/ais")
            .add_query_param("sw_lat", "33.6")
            .add_query_param("sw_lon", "-118.5")
            .add_query_param("ne_lat", "33.9")
            .add_query_param("ne_lon", "-118.0")
            .await;

        response.assert_status_ok();

        let json_response: Vec<AisResponse> = response.json();
        assert_eq!(json_response.len(), 1);
        assert_eq!(json_response[0].ship_name, Some("Bounding Box Query Received".to_string()));
        assert_eq!(json_response[0].latitude, Some(33.75)); // Average of sw_lat and ne_lat
        assert_eq!(json_response[0].longitude, Some(-118.25)); // Average of sw_lon and ne_lon
    }

    #[tokio::test]
    async fn test_get_ais_data_endpoint_missing_params() {
        // Create test state
        let state = AppState {
            ais_stream_manager: Arc::new(AisStreamManager::new()),
        };

        // Create test server
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();

        // Test request with missing parameters
        let response = server
            .get("/ais")
            .add_query_param("sw_lat", "33.6")
            .add_query_param("sw_lon", "-118.5")
            // Missing ne_lat and ne_lon
            .await;

        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_get_ais_data_endpoint_invalid_params() {
        // Create test state
        let state = AppState {
            ais_stream_manager: Arc::new(AisStreamManager::new()),
        };

        // Create test server
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();

        // Test request with invalid parameter types
        let response = server
            .get("/ais")
            .add_query_param("sw_lat", "invalid")
            .add_query_param("sw_lon", "-118.5")
            .add_query_param("ne_lat", "33.9")
            .add_query_param("ne_lon", "-118.0")
            .await;

        response.assert_status_bad_request();
    }

    #[test]
    fn test_bounding_box_query_validation() {
        // Test valid bounding box
        let valid_query = BoundingBoxQuery {
            sw_lat: 33.6,
            sw_lon: -118.5,
            ne_lat: 33.9,
            ne_lon: -118.0,
        };

        // Basic validation - northeast should be greater than southwest
        assert!(valid_query.ne_lat > valid_query.sw_lat);
        assert!(valid_query.ne_lon > valid_query.sw_lon);
    }

    #[test]
    fn test_ais_response_serialization() {
        let response = AisResponse {
            message_type: Some("PositionReport".to_string()),
            mmsi: Some("123456789".to_string()),
            ship_name: Some("Test Ship".to_string()),
            latitude: Some(33.7),
            longitude: Some(-118.3),
            timestamp: Some("2023-01-01T12:00:00Z".to_string()),
            speed_over_ground: Some(10.5),
            course_over_ground: Some(90.0),
            heading: Some(85.0),
            navigation_status: Some("Under way using engine".to_string()),
            ship_type: Some("Cargo".to_string()),
            raw_message: json!({"test": "data"}),
        };

        // Test that the response can be serialized to JSON
        let json_result = serde_json::to_string(&response);
        assert!(json_result.is_ok());

        let json_string = json_result.unwrap();
        assert!(json_string.contains("PositionReport"));
        assert!(json_string.contains("123456789"));
        assert!(json_string.contains("Test Ship"));
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState {
            ais_stream_manager: Arc::new(AisStreamManager::new()),
        };
        // Test that the manager is accessible.
        assert_eq!(state.ais_stream_manager.state.lock().await.client_count, 0);
    }

    #[test]
    fn test_subscription_message_serialization() {
        let subscription = SubscriptionMessage {
            apikey: "test_key".to_string(),
            bounding_boxes: vec![vec![
                [33.6, -118.5],
                [33.9, -118.0]
            ]],
            filters_ship_mmsi: vec!["123456789".to_string()],
        };

        let json_result = serde_json::to_string(&subscription);
        assert!(json_result.is_ok());

        let json_string = json_result.unwrap();
        assert!(json_string.contains("Apikey"));
        assert!(json_string.contains("BoundingBoxes"));
        assert!(json_string.contains("FiltersShipMMSI"));
    }

    #[tokio::test]
    async fn test_websocket_endpoint_exists() {
        // Create test state
        let state = AppState {
            ais_stream_manager: Arc::new(AisStreamManager::new()),
        };

        // Create test server
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();

        // The websocket endpoint should return 400 Bad Request
        // when accessed via HTTP GET without proper websocket headers
        let response = server.get("/ws").await;
        response.assert_status(axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_is_within_bounding_box() {
        let bbox = WebSocketBoundingBox {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0,
        };

        // Test point within bounding box
        let ais_within = AisResponse {
            message_type: Some("PositionReport".to_string()),
            mmsi: Some("123456789".to_string()),
            ship_name: Some("Test Ship".to_string()),
            latitude: Some(33.5),
            longitude: Some(-118.5),
            timestamp: Some("2023-01-01T12:00:00Z".to_string()),
            speed_over_ground: Some(10.0),
            course_over_ground: Some(90.0),
            heading: Some(85.0),
            navigation_status: Some("Under way using engine".to_string()),
            ship_type: Some("Cargo".to_string()),
            raw_message: serde_json::json!({"test": "data"}),
        };

        assert!(is_within_bounding_box(&ais_within, &bbox));

        // Test point outside bounding box (latitude too high)
        let ais_outside_lat = AisResponse {
            latitude: Some(35.0),
            longitude: Some(-118.5),
            ..ais_within.clone()
        };

        assert!(!is_within_bounding_box(&ais_outside_lat, &bbox));
    }
}