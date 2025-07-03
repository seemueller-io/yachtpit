use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_serial::SerialPortBuilderExt;
use datalink::{DataLinkConfig, DataLinkError, DataLinkReceiver, DataLinkResult, DataLinkStatus, DataLinkTransmitter, DataMessage};

/// Configuration for different types of GPS data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpsSourceConfig {
    /// Serial port configuration
    Serial {
        port: String,
        baud_rate: u32,
    },
    /// TCP connection configuration
    Tcp {
        host: String,
        port: u16,
    },
    /// UDP connection configuration
    Udp {
        bind_addr: String,
        port: u16,
    },
    /// File replay configuration
    File {
        path: String,
        replay_speed: f64, // 1.0 = real-time, 2.0 = 2x speed, etc.
    },
}

/// Real GPS Datalink Provider
pub struct GpsDataLinkProvider {
    status: DataLinkStatus,
    config: Option<DataLinkConfig>,
    source_config: Option<GpsSourceConfig>,
    message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
    receiver_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl GpsDataLinkProvider {
    /// Create a new GPS datalink provider
    pub fn new() -> Self {
        Self {
            status: DataLinkStatus::Disconnected,
            config: None,
            source_config: None,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            receiver_handle: None,
            shutdown_tx: None,
        }
    }

    /// Parse GPS source configuration from DataLinkConfig
    pub fn parse_source_config(config: &DataLinkConfig) -> DataLinkResult<GpsSourceConfig> {
        let connection_type = config.parameters.get("connection_type")
            .ok_or_else(|| DataLinkError::InvalidConfig("Missing connection_type".to_string()))?;

        match connection_type.as_str() {
            "serial" => {
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port for serial connection".to_string()))?;
                let baud_rate = config.parameters.get("baud_rate")
                    .unwrap_or(&"4800".to_string())
                    .parse::<u32>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid baud_rate".to_string()))?;

                Ok(GpsSourceConfig::Serial {
                    port: port.clone(),
                    baud_rate,
                })
            }
            "tcp" => {
                let host = config.parameters.get("host")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing host for TCP connection".to_string()))?;
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port for TCP connection".to_string()))?
                    .parse::<u16>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid port number".to_string()))?;

                Ok(GpsSourceConfig::Tcp {
                    host: host.clone(),
                    port,
                })
            }
            "udp" => {
                let bind_addr = config.parameters.get("bind_addr")
                    .unwrap_or(&"0.0.0.0".to_string())
                    .clone();
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port for UDP connection".to_string()))?
                    .parse::<u16>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid port number".to_string()))?;

                Ok(GpsSourceConfig::Udp {
                    bind_addr,
                    port,
                })
            }
            "file" => {
                let path = config.parameters.get("path")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing path for file replay".to_string()))?;
                let replay_speed = config.parameters.get("replay_speed")
                    .unwrap_or(&"1.0".to_string())
                    .parse::<f64>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid replay_speed".to_string()))?;

                Ok(GpsSourceConfig::File {
                    path: path.clone(),
                    replay_speed,
                })
            }
            _ => Err(DataLinkError::InvalidConfig(format!("Unsupported connection type: {}", connection_type))),
        }
    }

    /// Start the data receiver task based on the source configuration
    async fn start_receiver(&mut self) -> DataLinkResult<()> {
        let source_config = self.source_config.as_ref()
            .ok_or_else(|| DataLinkError::InvalidConfig("No source configuration".to_string()))?;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let message_queue = Arc::clone(&self.message_queue);

        let receiver_handle = match source_config {
            GpsSourceConfig::Serial { port, baud_rate } => {
                let port = port.clone();
                let baud_rate = *baud_rate;

                tokio::spawn(async move {
                    if let Err(e) = Self::serial_receiver(port, baud_rate, message_queue, &mut shutdown_rx).await {
                        error!("GPS Serial receiver error: {}", e);
                    }
                })
            }
            GpsSourceConfig::Tcp { host, port } => {
                let host = host.clone();
                let port = *port;

                tokio::spawn(async move {
                    if let Err(e) = Self::tcp_receiver(host, port, message_queue, &mut shutdown_rx).await {
                        error!("GPS TCP receiver error: {}", e);
                    }
                })
            }
            GpsSourceConfig::Udp { bind_addr, port } => {
                let bind_addr = bind_addr.clone();
                let port = *port;

                tokio::spawn(async move {
                    if let Err(e) = Self::udp_receiver(bind_addr, port, message_queue, &mut shutdown_rx).await {
                        error!("GPS UDP receiver error: {}", e);
                    }
                })
            }
            GpsSourceConfig::File { path, replay_speed } => {
                let path = path.clone();
                let replay_speed = *replay_speed;

                tokio::spawn(async move {
                    if let Err(e) = Self::file_receiver(path, replay_speed, message_queue, &mut shutdown_rx).await {
                        error!("GPS File receiver error: {}", e);
                    }
                })
            }
        };

        self.receiver_handle = Some(receiver_handle);
        self.shutdown_tx = Some(shutdown_tx);

        Ok(())
    }

    /// Serial port receiver implementation
    async fn serial_receiver(
        port: String,
        baud_rate: u32,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting GPS serial receiver on port {} at {} baud", port, baud_rate);

        let serial_port = tokio_serial::new(&port, baud_rate)
            .open_native_async()?;

        let mut reader = BufReader::new(serial_port);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("GPS Serial receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            warn!("GPS Serial port closed");
                            break;
                        }
                        Ok(_) => {
                            if let Some(message) = Self::parse_gps_sentence(&line.trim()) {
                                if let Ok(mut queue) = message_queue.lock() {
                                    queue.push_back(message);
                                    // Limit queue size to prevent memory issues
                                    if queue.len() > 1000 {
                                        queue.pop_front();
                                    }
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("GPS Serial read error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// TCP receiver implementation
    async fn tcp_receiver(
        host: String,
        port: u16,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting GPS TCP receiver connecting to {}:{}", host, port);

        let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("GPS TCP receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            warn!("GPS TCP connection closed");
                            break;
                        }
                        Ok(_) => {
                            if let Some(message) = Self::parse_gps_sentence(&line.trim()) {
                                if let Ok(mut queue) = message_queue.lock() {
                                    queue.push_back(message);
                                    if queue.len() > 1000 {
                                        queue.pop_front();
                                    }
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("GPS TCP read error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// UDP receiver implementation
    async fn udp_receiver(
        bind_addr: String,
        port: u16,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting GPS UDP receiver on {}:{}", bind_addr, port);

        let socket = UdpSocket::bind(format!("{}:{}", bind_addr, port)).await?;
        let mut buf = [0; 1024];

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("GPS UDP receiver shutdown requested");
                    break;
                }
                result = socket.recv(&mut buf) => {
                    match result {
                        Ok(len) => {
                            let data = String::from_utf8_lossy(&buf[..len]);
                            for line in data.lines() {
                                if let Some(message) = Self::parse_gps_sentence(line.trim()) {
                                    if let Ok(mut queue) = message_queue.lock() {
                                        queue.push_back(message);
                                        if queue.len() > 1000 {
                                            queue.pop_front();
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("GPS UDP receive error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// File receiver implementation for replaying GPS data
    async fn file_receiver(
        path: String,
        replay_speed: f64,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting GPS file receiver for {} at {}x speed", path, replay_speed);

        let file = tokio::fs::File::open(&path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let delay_duration = Duration::from_millis((1000.0 / replay_speed) as u64);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("GPS File receiver shutdown requested");
                    break;
                }
                result = lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if let Some(message) = Self::parse_gps_sentence(&line.trim()) {
                                if let Ok(mut queue) = message_queue.lock() {
                                    queue.push_back(message);
                                    if queue.len() > 1000 {
                                        queue.pop_front();
                                    }
                                }
                            }
                            tokio::time::sleep(delay_duration).await;
                        }
                        Ok(None) => {
                            info!("GPS End of file reached");
                            break;
                        }
                        Err(e) => {
                            error!("GPS File read error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse a GPS NMEA sentence into a DataMessage
    pub fn parse_gps_sentence(sentence: &str) -> Option<DataMessage> {
        if !sentence.starts_with('$') {
            return None;
        }

        // Basic NMEA sentence validation
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() < 3 {
            return None;
        }

        // Extract sentence type (first part after $)
        let sentence_type = parts[0];

        // Check for common GPS sentence types
        if !sentence_type.contains("GPGGA") &&
            !sentence_type.contains("GPRMC") &&
            !sentence_type.contains("GPGLL") &&
            !sentence_type.contains("GPVTG") &&
            !sentence_type.contains("GPGSA") &&
            !sentence_type.contains("GPGSV") &&
            !sentence_type.contains("GNRMC") &&
            !sentence_type.contains("GNGGA") &&
            !sentence_type.contains("GNGLL") {
            return None;
        }

        // Create a DataMessage from the GPS sentence
        let mut message = DataMessage::new(
            "GPS_SENTENCE".to_string(),
            "GPS_RECEIVER".to_string(),
            sentence.as_bytes().to_vec(),
        );

        // Add parsed data based on sentence type
        message = message.with_data("sentence_type".to_string(), sentence_type.to_string());

        // Parse specific GPS sentence types
        match sentence_type {
            s if s.contains("GPGGA") || s.contains("GNGGA") => {
                // Global Positioning System Fix Data
                if parts.len() >= 15 {
                    message = message.with_data("time".to_string(), parts[1].to_string());
                    message = message.with_data("latitude".to_string(), parts[2].to_string());
                    message = message.with_data("lat_direction".to_string(), parts[3].to_string());
                    message = message.with_data("longitude".to_string(), parts[4].to_string());
                    message = message.with_data("lon_direction".to_string(), parts[5].to_string());
                    message = message.with_data("fix_quality".to_string(), parts[6].to_string());
                    message = message.with_data("satellites".to_string(), parts[7].to_string());
                    message = message.with_data("hdop".to_string(), parts[8].to_string());
                    message = message.with_data("altitude".to_string(), parts[9].to_string());
                    message = message.with_data("altitude_unit".to_string(), parts[10].to_string());
                }
            }
            s if s.contains("GPRMC") || s.contains("GNRMC") => {
                // Recommended Minimum Course
                if parts.len() >= 12 {
                    message = message.with_data("time".to_string(), parts[1].to_string());
                    message = message.with_data("status".to_string(), parts[2].to_string());
                    message = message.with_data("latitude".to_string(), parts[3].to_string());
                    message = message.with_data("lat_direction".to_string(), parts[4].to_string());
                    message = message.with_data("longitude".to_string(), parts[5].to_string());
                    message = message.with_data("lon_direction".to_string(), parts[6].to_string());
                    message = message.with_data("speed".to_string(), parts[7].to_string());
                    message = message.with_data("course".to_string(), parts[8].to_string());
                    message = message.with_data("date".to_string(), parts[9].to_string());
                }
            }
            s if s.contains("GPGLL") || s.contains("GNGLL") => {
                // Geographic Position - Latitude/Longitude
                if parts.len() >= 7 {
                    message = message.with_data("latitude".to_string(), parts[1].to_string());
                    message = message.with_data("lat_direction".to_string(), parts[2].to_string());
                    message = message.with_data("longitude".to_string(), parts[3].to_string());
                    message = message.with_data("lon_direction".to_string(), parts[4].to_string());
                    message = message.with_data("time".to_string(), parts[5].to_string());
                    message = message.with_data("status".to_string(), parts[6].to_string());
                }
            }
            _ => {
                // For other sentence types, just store the raw parts
                for (i, part) in parts.iter().enumerate() {
                    message = message.with_data(format!("field_{}", i), part.to_string());
                }
            }
        }

        // Add timestamp
        message = message.with_data(
            "timestamp".to_string(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );

        // Set signal quality based on sentence completeness and checksum
        let quality = if sentence.contains('*') { 95 } else { 75 };
        message = message.with_signal_quality(quality);

        Some(message)
    }

    /// Stop the receiver task
    async fn stop_receiver(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        if let Some(handle) = self.receiver_handle.take() {
            let _ = handle.await;
        }
    }
}

impl Default for GpsDataLinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLinkReceiver for GpsDataLinkProvider {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn receive_message(&mut self) -> DataLinkResult<Option<DataMessage>> {
        if let Ok(mut queue) = self.message_queue.lock() {
            Ok(queue.pop_front())
        } else {
            Err(DataLinkError::TransportError("Failed to access message queue".to_string()))
        }
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        info!("Connecting GPS datalink provider");

        self.status = DataLinkStatus::Connecting;
        self.config = Some(config.clone());

        // Parse source configuration
        self.source_config = Some(Self::parse_source_config(config)?);

        // Start the receiver in a blocking context
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DataLinkError::ConnectionFailed(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            self.start_receiver().await
        })?;

        self.status = DataLinkStatus::Connected;
        info!("GPS datalink provider connected successfully");

        Ok(())
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        info!("Disconnecting GPS datalink provider");

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DataLinkError::TransportError(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            self.stop_receiver().await;
        });

        self.status = DataLinkStatus::Disconnected;
        self.config = None;
        self.source_config = None;

        info!("GPS datalink provider disconnected");
        Ok(())
    }
}

impl DataLinkTransmitter for GpsDataLinkProvider {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn send_message(&mut self, _message: &DataMessage) -> DataLinkResult<()> {
        // GPS transmission is typically not supported for consumer devices
        // This could be extended in the future for specialized GPS equipment
        Err(DataLinkError::TransportError("GPS transmission not supported".to_string()))
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        // Use the same connection logic as receiver
        DataLinkReceiver::connect(self, config)
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        // Use the same disconnection logic as receiver
        DataLinkReceiver::disconnect(self)
    }
}
