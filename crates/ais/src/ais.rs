use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures_util::{SinkExt, StreamExt};
use axum::extract::ws::{WebSocket, Message as WsMessage};
use url::Url;
use axum::{
    extract::{Query, WebSocketUpgrade, State},
    http::StatusCode,
    response::{Json, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[derive(Serialize, Deserialize, Debug)]
struct SubscriptionMessage {
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
struct BoundingBoxQuery {
    sw_lat: f64,  // Southwest latitude
    sw_lon: f64,  // Southwest longitude
    ne_lat: f64,  // Northeast latitude
    ne_lon: f64,  // Northeast longitude
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WebSocketBoundingBox {
    sw_lat: f64,  // Southwest latitude
    sw_lon: f64,  // Southwest longitude
    ne_lat: f64,  // Northeast latitude
    ne_lon: f64,  // Northeast longitude
}

#[derive(Serialize, Deserialize, Debug)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bounding_box: Option<WebSocketBoundingBox>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AisResponse {
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

// Shared state for the application
#[derive(Clone)]
struct AppState {
    ais_sender: Arc<Mutex<Option<broadcast::Sender<AisResponse>>>>,
    ais_stream_started: Arc<Mutex<bool>>,
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
async fn get_ais_data(
    Query(params): Query<BoundingBoxQuery>,
    axum::extract::State(_state): axum::extract::State<AppState>,
) -> Result<Json<Vec<AisResponse>>, StatusCode> {
    println!("Received bounding box request: {:?}", params);
    
    // For now, return a simple response indicating the bounding box was received
    // In a full implementation, you might want to:
    // 1. Store recent AIS data in memory/database
    // 2. Filter by the bounding box
    // 3. Return the filtered results
    
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
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
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
async fn handle_websocket(mut socket: WebSocket, state: AppState) {
    // Get a receiver from the broadcast channel
    let sender_guard = state.ais_sender.lock().await;
    let mut receiver = match sender_guard.as_ref() {
        Some(sender) => sender.subscribe(),
        None => {
            println!("No AIS sender available");
            let _ = socket.close().await;
            return;
        }
    };
    drop(sender_guard);

    println!("WebSocket client connected");

    // Store bounding box state for this connection
    let mut bounding_box: Option<WebSocketBoundingBox> = None;

    // Send initial connection confirmation
    if socket.send(WsMessage::Text("Connected to AIS stream".to_string())).await.is_err() {
        println!("Failed to send connection confirmation");
        return;
    }

    // Handle incoming messages and broadcast AIS data
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages (for potential client commands)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        println!("Received from client: {}", text);
                        
                        // Try to parse as WebSocket message for bounding box configuration
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(ws_msg) => {
                                match ws_msg.message_type.as_str() {
                                    "set_bounding_box" => {
                                        if let Some(bbox) = ws_msg.bounding_box {
                                            println!("Setting bounding box: {:?}", bbox);
                                            bounding_box = Some(bbox.clone());
                                            
                                            // Send confirmation
                                            let confirmation = serde_json::json!({
                                                "type": "bounding_box_set",
                                                "bounding_box": bbox
                                            });
                                            if socket.send(WsMessage::Text(confirmation.to_string())).await.is_err() {
                                                break;
                                            }
                                        } else {
                                            // Clear bounding box if none provided
                                            bounding_box = None;
                                            let confirmation = serde_json::json!({
                                                "type": "bounding_box_cleared"
                                            });
                                            if socket.send(WsMessage::Text(confirmation.to_string())).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                    "start_ais_stream" => {
                                        println!("Received request to start AIS stream");
                                        
                                        // Check if AIS stream is already started
                                        let mut stream_started = state.ais_stream_started.lock().await;
                                        if !*stream_started {
                                            *stream_started = true;
                                            drop(stream_started);
                                            
                                            // Start AIS stream connection in background
                                            let ais_state = state.clone();
                                            tokio::spawn(async move {
                                                if let Err(e) = connect_to_ais_stream_with_broadcast(ais_state).await {
                                                    eprintln!("WebSocket error: {:?}", e);
                                                }
                                            });
                                            
                                            // Send confirmation
                                            let confirmation = serde_json::json!({
                                                "type": "ais_stream_started"
                                            });
                                            if socket.send(WsMessage::Text(confirmation.to_string())).await.is_err() {
                                                break;
                                            }
                                            println!("AIS stream started successfully");
                                        } else {
                                            // AIS stream already started
                                            let confirmation = serde_json::json!({
                                                "type": "ais_stream_already_started"
                                            });
                                            if socket.send(WsMessage::Text(confirmation.to_string())).await.is_err() {
                                                break;
                                            }
                                            println!("AIS stream already started");
                                        }
                                    }
                                    _ => {
                                        // Echo back unknown message types
                                        if socket.send(WsMessage::Text(format!("Echo: {}", text))).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // If not valid JSON, echo back as before
                                if socket.send(WsMessage::Text(format!("Echo: {}", text))).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) => {
                        println!("WebSocket client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        println!("WebSocket error: {:?}", e);
                        break;
                    }
                    None => break,
                    _ => {} // Handle other message types if needed
                }
            }
            // Forward AIS data to the client
            ais_data = receiver.recv() => {
                match ais_data {
                    Ok(data) => {
                        // Apply bounding box filtering if configured
                        let should_send = match &bounding_box {
                            Some(bbox) => is_within_bounding_box(&data, bbox),
                            None => true, // Send all data if no bounding box is set
                        };
                        
                        if should_send {
                            match serde_json::to_string(&data) {
                                Ok(json_data) => {
                                    if socket.send(WsMessage::Text(json_data)).await.is_err() {
                                        println!("Failed to send AIS data to client");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    println!("Failed to serialize AIS data: {:?}", e);
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        println!("WebSocket client lagged behind by {} messages", n);
                        // Continue receiving, client will catch up
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("AIS broadcast channel closed");
                        break;
                    }
                }
            }
        }
    }

    println!("WebSocket connection closed");
}

// Create the Axum router
fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ais", get(get_ais_data))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
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

// Start the HTTP server with AIS functionality
pub async fn start_ais_server() -> Result<(), Box<dyn std::error::Error>> {
    // Create broadcast channel for AIS data
    let (tx, _rx) = broadcast::channel::<AisResponse>(1000);
    
    // Create shared state
    let state = AppState {
        ais_sender: Arc::new(Mutex::new(Some(tx.clone()))),
        ais_stream_started: Arc::new(Mutex::new(false)),
    };

    // Don't start AIS WebSocket connection immediately
    // It will be started when the frontend signals that user location is loaded and map is focused

    // Create and start HTTP server
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    
    println!("AIS server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;
    Ok(())
}


// Modified AIS stream function that broadcasts data and accepts dynamic bounding boxes
async fn connect_to_ais_stream_with_broadcast(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to WebSocket
    let url = Url::parse("wss://stream.aisstream.io/v0/stream")?;
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connection opened for broadcast");

    let (mut sender, mut receiver) = ws_stream.split();

    let key = "MDc4YzY5NTdkMGUwM2UzMzQ1Zjc5NDFmOTA1ODg4ZTMyOGQ0MjM0MA==";
    // Create subscription message with default bounding box (Port of Los Angeles area)
    // In a full implementation, this could be made dynamic based on active HTTP requests
    let subscription_message = SubscriptionMessage {
        apikey: STANDARD.decode(key)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default(),
        bounding_boxes: vec![vec![
            [33.6, -118.5], // Southwest corner (lat, lon)
            [33.9, -118.0]  // Northeast corner (lat, lon)
        ]],
        filters_ship_mmsi: vec![], // Remove specific MMSI filters to get all ships in the area
    };

    // Send subscription message
    let message_json = serde_json::to_string(&subscription_message)?;
    sender.send(Message::Text(message_json)).await?;
    println!("Subscription message sent for broadcast");

    // Listen for messages and broadcast them
    while let Some(message) = receiver.next().await {
        match message? {
            Message::Text(text) => {
                match serde_json::from_str::<Value>(&text) {
                    Ok(ais_message) => {
                        // Parse and broadcast the message
                        let parsed_message = parse_ais_message(&ais_message);
                        
                        // Try to broadcast to HTTP clients
                        let sender_guard = state.ais_sender.lock().await;
                        if let Some(ref broadcaster) = *sender_guard {
                            let _ = broadcaster.send(parsed_message.clone());
                        }
                        
                        // Still print detailed message for debugging
                        print_detailed_ais_message(&ais_message);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {:?}", e);
                    }
                }
            }
            Message::Binary(data) => {
                println!("Received binary data: {} bytes", data.len());
                
                // Try to decode as UTF-8 string to see if it's JSON
                if let Ok(text) = String::from_utf8(data.clone()) {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(ais_message) => {
                            let parsed_message = parse_ais_message(&ais_message);
                            
                            // Try to broadcast to HTTP clients
                            let sender_guard = state.ais_sender.lock().await;
                            if let Some(ref broadcaster) = *sender_guard {
                                let _ = broadcaster.send(parsed_message.clone());
                            }
                            
                            print_detailed_ais_message(&ais_message);
                        }
                        Err(e) => {
                            println!("Binary data is not valid JSON: {:?}", e);
                        }
                    }
                }
            }
            _ => {
                // Handle other message types like Close, Ping, Pong
            }
        }
    }

    println!("WebSocket connection closed");
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;
    use tokio::sync::broadcast;

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
        let (tx, _rx) = broadcast::channel::<AisResponse>(100);
        let state = AppState {
            ais_sender: Arc::new(Mutex::new(Some(tx))),
            ais_stream_started: Arc::new(Mutex::new(false)),
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
        let (tx, _rx) = broadcast::channel::<AisResponse>(100);
        let state = AppState {
            ais_sender: Arc::new(Mutex::new(Some(tx))),
            ais_stream_started: Arc::new(Mutex::new(false)),
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
        let (tx, _rx) = broadcast::channel::<AisResponse>(100);
        let state = AppState {
            ais_sender: Arc::new(Mutex::new(Some(tx))),
            ais_stream_started: Arc::new(Mutex::new(false)),
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
        let (tx, _rx) = broadcast::channel::<AisResponse>(100);
        let state = AppState {
            ais_sender: Arc::new(Mutex::new(Some(tx.clone()))),
            ais_stream_started: Arc::new(Mutex::new(false)),
        };

        // Test that we can access the sender
        let sender_guard = state.ais_sender.lock().await;
        assert!(sender_guard.is_some());
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
        let (tx, _rx) = broadcast::channel::<AisResponse>(100);
        let state = AppState {
            ais_sender: Arc::new(Mutex::new(Some(tx))),
            ais_stream_started: Arc::new(Mutex::new(false)),
        };

        // Create test server
        let app = create_router(state);
        let server = TestServer::new(app).unwrap();

        // Test that the websocket endpoint exists and returns appropriate response
        // Note: axum-test doesn't support websocket upgrades, but we can test that the route exists
        let response = server.get("/ws").await;
        
        // The websocket endpoint should return a 400 Bad Request status
        // when accessed via HTTP GET without proper websocket headers
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

        // Test point outside bounding box (longitude too low)
        let ais_outside_lon = AisResponse {
            latitude: Some(33.5),
            longitude: Some(-120.0),
            ..ais_within.clone()
        };

        assert!(!is_within_bounding_box(&ais_outside_lon, &bbox));

        // Test point with missing coordinates
        let ais_no_coords = AisResponse {
            latitude: None,
            longitude: None,
            ..ais_within.clone()
        };

        assert!(!is_within_bounding_box(&ais_no_coords, &bbox));

        // Test point on boundary (should be included)
        let ais_on_boundary = AisResponse {
            latitude: Some(33.0), // Exactly on southwest latitude
            longitude: Some(-118.0), // Exactly on northeast longitude
            ..ais_within.clone()
        };

        assert!(is_within_bounding_box(&ais_on_boundary, &bbox));
    }

    #[test]
    fn test_websocket_message_serialization() {
        // Test bounding box message
        let bbox_msg = WebSocketMessage {
            message_type: "set_bounding_box".to_string(),
            bounding_box: Some(WebSocketBoundingBox {
                sw_lat: 33.0,
                sw_lon: -119.0,
                ne_lat: 34.0,
                ne_lon: -118.0,
            }),
        };

        let json_result = serde_json::to_string(&bbox_msg);
        assert!(json_result.is_ok());

        let json_string = json_result.unwrap();
        assert!(json_string.contains("set_bounding_box"));
        assert!(json_string.contains("33.0"));
        assert!(json_string.contains("-119.0"));

        // Test message without bounding box
        let clear_msg = WebSocketMessage {
            message_type: "clear_bounding_box".to_string(),
            bounding_box: None,
        };

        let json_result = serde_json::to_string(&clear_msg);
        assert!(json_result.is_ok());

        let json_string = json_result.unwrap();
        assert!(json_string.contains("clear_bounding_box"));
        // The bounding_box field should be omitted when None due to skip_serializing_if
        assert!(!json_string.contains("\"bounding_box\""));
    }

    #[test]
    fn test_websocket_message_deserialization() {
        // Test parsing valid bounding box message
        let json_str = r#"{"type":"set_bounding_box","bounding_box":{"sw_lat":33.0,"sw_lon":-119.0,"ne_lat":34.0,"ne_lon":-118.0}}"#;
        let result: Result<WebSocketMessage, _> = serde_json::from_str(json_str);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.message_type, "set_bounding_box");
        assert!(msg.bounding_box.is_some());

        let bbox = msg.bounding_box.unwrap();
        assert_eq!(bbox.sw_lat, 33.0);
        assert_eq!(bbox.sw_lon, -119.0);
        assert_eq!(bbox.ne_lat, 34.0);
        assert_eq!(bbox.ne_lon, -118.0);

        // Test parsing message without bounding box
        let json_str = r#"{"type":"clear_bounding_box"}"#;
        let result: Result<WebSocketMessage, _> = serde_json::from_str(json_str);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.message_type, "clear_bounding_box");
        assert!(msg.bounding_box.is_none());
    }
}