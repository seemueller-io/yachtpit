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

/// Configuration for different types of AIS data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AisSourceConfig {
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

/// Real AIS Datalink Provider
pub struct AisDataLinkProvider {
    status: DataLinkStatus,
    config: Option<DataLinkConfig>,
    source_config: Option<AisSourceConfig>,
    message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
    receiver_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl AisDataLinkProvider {
    /// Create a new AIS datalink provider
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

    /// Parse AIS source configuration from DataLinkConfig
    pub fn parse_source_config(config: &DataLinkConfig) -> DataLinkResult<AisSourceConfig> {
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

                Ok(AisSourceConfig::Serial {
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

                Ok(AisSourceConfig::Tcp {
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

                Ok(AisSourceConfig::Udp {
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

                Ok(AisSourceConfig::File {
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
            AisSourceConfig::Serial { port, baud_rate } => {
                let port = port.clone();
                let baud_rate = *baud_rate;

                tokio::spawn(async move {
                    if let Err(e) = Self::serial_receiver(port, baud_rate, message_queue, &mut shutdown_rx).await {
                        error!("Serial receiver error: {}", e);
                    }
                })
            }
            AisSourceConfig::Tcp { host, port } => {
                let host = host.clone();
                let port = *port;

                tokio::spawn(async move {
                    if let Err(e) = Self::tcp_receiver(host, port, message_queue, &mut shutdown_rx).await {
                        error!("TCP receiver error: {}", e);
                    }
                })
            }
            AisSourceConfig::Udp { bind_addr, port } => {
                let bind_addr = bind_addr.clone();
                let port = *port;

                tokio::spawn(async move {
                    if let Err(e) = Self::udp_receiver(bind_addr, port, message_queue, &mut shutdown_rx).await {
                        error!("UDP receiver error: {}", e);
                    }
                })
            }
            AisSourceConfig::File { path, replay_speed } => {
                let path = path.clone();
                let replay_speed = *replay_speed;

                tokio::spawn(async move {
                    if let Err(e) = Self::file_receiver(path, replay_speed, message_queue, &mut shutdown_rx).await {
                        error!("File receiver error: {}", e);
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
        info!("Starting serial receiver on port {} at {} baud", port, baud_rate);

        let serial_port = tokio_serial::new(&port, baud_rate)
            .open_native_async()?;

        let mut reader = BufReader::new(serial_port);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Serial receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            warn!("Serial port closed");
                            break;
                        }
                        Ok(_) => {
                            if let Some(message) = Self::parse_ais_sentence(&line.trim()) {
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
                            error!("Serial read error: {}", e);
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
        info!("Starting TCP receiver connecting to {}:{}", host, port);

        let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("TCP receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            warn!("TCP connection closed");
                            break;
                        }
                        Ok(_) => {
                            if let Some(message) = Self::parse_ais_sentence(&line.trim()) {
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
                            error!("TCP read error: {}", e);
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
        info!("Starting UDP receiver on {}:{}", bind_addr, port);

        let socket = UdpSocket::bind(format!("{}:{}", bind_addr, port)).await?;
        let mut buf = [0; 1024];

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("UDP receiver shutdown requested");
                    break;
                }
                result = socket.recv(&mut buf) => {
                    match result {
                        Ok(len) => {
                            let data = String::from_utf8_lossy(&buf[..len]);
                            for line in data.lines() {
                                if let Some(message) = Self::parse_ais_sentence(line.trim()) {
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
                            error!("UDP receive error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// File receiver implementation for replaying AIS data
    async fn file_receiver(
        path: String,
        replay_speed: f64,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting file receiver for {} at {}x speed", path, replay_speed);

        let file = tokio::fs::File::open(&path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let delay_duration = Duration::from_millis((1000.0 / replay_speed) as u64);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("File receiver shutdown requested");
                    break;
                }
                result = lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if let Some(message) = Self::parse_ais_sentence(&line.trim()) {
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
                            info!("End of file reached");
                            break;
                        }
                        Err(e) => {
                            error!("File read error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse an AIS sentence into a DataMessage
    pub fn parse_ais_sentence(sentence: &str) -> Option<DataMessage> {
        if !sentence.starts_with('!') && !sentence.starts_with('$') {
            return None;
        }

        // Basic NMEA sentence validation
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() < 6 {
            return None;
        }

        // Extract basic information from AIS sentence
        let sentence_type = parts[0];
        if !sentence_type.contains("AIVDM") && !sentence_type.contains("AIVDO") {
            return None;
        }

        // Create a DataMessage from the AIS sentence
        let mut message = DataMessage::new(
            "AIS_SENTENCE".to_string(),
            "AIS_RECEIVER".to_string(),
            sentence.as_bytes().to_vec(),
        );

        // Add parsed data if available
        if parts.len() >= 6 {
            message = message.with_data("sentence_type".to_string(), sentence_type.to_string());
            message = message.with_data("fragment_count".to_string(), parts[1].to_string());
            message = message.with_data("fragment_number".to_string(), parts[2].to_string());
            message = message.with_data("message_id".to_string(), parts[3].to_string());
            message = message.with_data("channel".to_string(), parts[4].to_string());
            message = message.with_data("payload".to_string(), parts[5].to_string());
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

        // Set signal quality based on sentence completeness
        let quality = if sentence.contains('*') { 90 } else { 70 };
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

impl Default for AisDataLinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLinkReceiver for AisDataLinkProvider {
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
        info!("Connecting AIS datalink provider");

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
        info!("AIS datalink provider connected successfully");

        Ok(())
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        info!("Disconnecting AIS datalink provider");

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DataLinkError::TransportError(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            self.stop_receiver().await;
        });

        self.status = DataLinkStatus::Disconnected;
        self.config = None;
        self.source_config = None;

        info!("AIS datalink provider disconnected");
        Ok(())
    }
}

impl DataLinkTransmitter for AisDataLinkProvider {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn send_message(&mut self, _message: &DataMessage) -> DataLinkResult<()> {
        // For now, AIS transmission is not implemented as it requires special equipment
        // and licensing. This could be extended in the future for AIS transponders.
        Err(DataLinkError::TransportError("AIS transmission not supported".to_string()))
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
