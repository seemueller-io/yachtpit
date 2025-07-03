use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_serial::SerialPortBuilderExt;
use datalink::{DataLinkConfig, DataLinkError, DataLinkReceiver, DataLinkResult, DataLinkStatus, DataLinkTransmitter, DataMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RadarSourceConfig {
    /// Serial port connection for radar data
    Serial {
        port: String,
        baud_rate: u32,
    },
    /// TCP connection for networked radar data
    Tcp {
        host: String,
        port: u16,
    },
    /// UDP connection for radar data
    Udp {
        bind_addr: String,
        port: u16,
    },
    /// File-based radar data replay
    File {
        path: String,
        replay_speed: f64,
    },
}

pub struct RadarDataLinkProvider {
    status: DataLinkStatus,
    config: Option<RadarSourceConfig>,
    message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    receiver_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RadarDataLinkProvider {
    pub fn new() -> Self {
        Self {
            status: DataLinkStatus::Disconnected,
            config: None,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            shutdown_tx: None,
            receiver_handle: None,
        }
    }

    pub fn parse_source_config(config: &DataLinkConfig) -> DataLinkResult<RadarSourceConfig> {
        let connection_type = config.parameters.get("connection_type")
            .ok_or_else(|| DataLinkError::InvalidConfig("Missing connection_type parameter".to_string()))?;

        match connection_type.as_str() {
            "serial" => {
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port parameter for serial connection".to_string()))?
                    .clone();
                let baud_rate = config.parameters.get("baud_rate")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing baud_rate parameter for serial connection".to_string()))?
                    .parse::<u32>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid baud_rate parameter".to_string()))?;

                Ok(RadarSourceConfig::Serial { port, baud_rate })
            }
            "tcp" => {
                let host = config.parameters.get("host")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing host parameter for TCP connection".to_string()))?
                    .clone();
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port parameter for TCP connection".to_string()))?
                    .parse::<u16>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid port parameter".to_string()))?;

                Ok(RadarSourceConfig::Tcp { host, port })
            }
            "udp" => {
                let bind_addr = config.parameters.get("bind_addr")
                    .unwrap_or(&"0.0.0.0".to_string())
                    .clone();
                let port = config.parameters.get("port")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing port parameter for UDP connection".to_string()))?
                    .parse::<u16>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid port parameter".to_string()))?;

                Ok(RadarSourceConfig::Udp { bind_addr, port })
            }
            "file" => {
                let path = config.parameters.get("path")
                    .ok_or_else(|| DataLinkError::InvalidConfig("Missing path parameter for file connection".to_string()))?
                    .clone();
                let replay_speed = config.parameters.get("replay_speed")
                    .unwrap_or(&"1.0".to_string())
                    .parse::<f64>()
                    .map_err(|_| DataLinkError::InvalidConfig("Invalid replay_speed parameter".to_string()))?;

                Ok(RadarSourceConfig::File { path, replay_speed })
            }
            _ => Err(DataLinkError::InvalidConfig(format!("Unsupported connection type: {}", connection_type))),
        }
    }

    fn start_receiver(&mut self) -> DataLinkResult<()> {
        if let Some(config) = &self.config {
            let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
            let message_queue = Arc::clone(&self.message_queue);

            let handle = match config {
                RadarSourceConfig::Serial { port, baud_rate } => {
                    let port = port.clone();
                    let baud_rate = *baud_rate;
                    tokio::spawn(async move {
                        if let Err(e) = Self::serial_receiver(port, baud_rate, message_queue, &mut shutdown_rx).await {
                            error!("Radar serial receiver error: {}", e);
                        }
                    })
                }
                RadarSourceConfig::Tcp { host, port } => {
                    let host = host.clone();
                    let port = *port;
                    tokio::spawn(async move {
                        if let Err(e) = Self::tcp_receiver(host, port, message_queue, &mut shutdown_rx).await {
                            error!("Radar TCP receiver error: {}", e);
                        }
                    })
                }
                RadarSourceConfig::Udp { bind_addr, port } => {
                    let bind_addr = bind_addr.clone();
                    let port = *port;
                    tokio::spawn(async move {
                        if let Err(e) = Self::udp_receiver(bind_addr, port, message_queue, &mut shutdown_rx).await {
                            error!("Radar UDP receiver error: {}", e);
                        }
                    })
                }
                RadarSourceConfig::File { path, replay_speed } => {
                    let path = path.clone();
                    let replay_speed = *replay_speed;
                    tokio::spawn(async move {
                        if let Err(e) = Self::file_receiver(path, replay_speed, message_queue, &mut shutdown_rx).await {
                            error!("Radar file receiver error: {}", e);
                        }
                    })
                }
            };

            self.shutdown_tx = Some(shutdown_tx);
            self.receiver_handle = Some(handle);
            self.status = DataLinkStatus::Connected;
            Ok(())
        } else {
            Err(DataLinkError::InvalidConfig("No configuration set".to_string()))
        }
    }

    async fn serial_receiver(
        port: String,
        baud_rate: u32,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting radar serial receiver on {} at {} baud", port, baud_rate);

        let serial_stream = tokio_serial::new(&port, baud_rate)
            .open_native_async()?;

        let mut reader = BufReader::new(serial_stream);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Radar serial receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trimmed = line.trim();
                            if let Some(message) = Self::parse_radar_sentence(trimmed) {
                                if let Ok(mut queue) = message_queue.lock() {
                                    queue.push_back(message);
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("Error reading from radar serial port: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn tcp_receiver(
        host: String,
        port: u16,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting radar TCP receiver on {}:{}", host, port);

        let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Radar TCP receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trimmed = line.trim();
                            if let Some(message) = Self::parse_radar_sentence(trimmed) {
                                if let Ok(mut queue) = message_queue.lock() {
                                    queue.push_back(message);
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("Error reading from radar TCP connection: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn udp_receiver(
        bind_addr: String,
        port: u16,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting radar UDP receiver on {}:{}", bind_addr, port);

        let socket = UdpSocket::bind(format!("{}:{}", bind_addr, port)).await?;
        let mut buf = [0; 1024];

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Radar UDP receiver shutdown requested");
                    break;
                }
                result = socket.recv(&mut buf) => {
                    match result {
                        Ok(len) => {
                            let data = String::from_utf8_lossy(&buf[..len]);
                            for line in data.lines() {
                                if let Some(message) = Self::parse_radar_sentence(line.trim()) {
                                    if let Ok(mut queue) = message_queue.lock() {
                                        queue.push_back(message);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error reading from radar UDP socket: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn file_receiver(
        path: String,
        replay_speed: f64,
        message_queue: Arc<Mutex<VecDeque<DataMessage>>>,
        shutdown_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting radar file receiver from {} at {}x speed", path, replay_speed);

        let file = tokio::fs::File::open(&path).await?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();

        let delay_duration = Duration::from_millis((1000.0 / replay_speed) as u64);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Radar file receiver shutdown requested");
                    break;
                }
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            info!("Radar file replay completed");
                            break;
                        }
                        Ok(_) => {
                            let trimmed = line.trim();
                            if let Some(message) = Self::parse_radar_sentence(trimmed) {
                                if let Ok(mut queue) = message_queue.lock() {
                                    queue.push_back(message);
                                }
                            }
                            line.clear();
                            tokio::time::sleep(delay_duration).await;
                        }
                        Err(e) => {
                            error!("Error reading from radar file: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn parse_radar_sentence(sentence: &str) -> Option<DataMessage> {
        // Parse various radar sentence formats
        if sentence.starts_with("$RADTG") {
            // Radar Target message
            Self::parse_radar_target(sentence)
        } else if sentence.starts_with("$RADSC") {
            // Radar Scan message
            Self::parse_radar_scan(sentence)
        } else if sentence.starts_with("$RADCF") {
            // Radar Configuration message
            Self::parse_radar_config(sentence)
        } else if sentence.starts_with("$RADST") {
            // Radar Status message
            Self::parse_radar_status(sentence)
        } else {
            None
        }
    }

    fn parse_radar_target(sentence: &str) -> Option<DataMessage> {
        // Example: $RADTG,123.45,67.89,12.3,045,15.2*7A
        // Format: $RADTG,range_nm,bearing_deg,speed_kts,course_deg,cpa_nm*checksum
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() >= 6 && parts[0] == "$RADTG" {
            let mut message = DataMessage::new(
                "RADAR_TARGET".to_string(),
                "RADAR_RECEIVER".to_string(),
                sentence.as_bytes().to_vec(),
            );

            if let Ok(range) = parts[1].parse::<f32>() {
                message = message.with_data("range_nm".to_string(), range.to_string());
            }
            if let Ok(bearing) = parts[2].parse::<f32>() {
                message = message.with_data("bearing_deg".to_string(), bearing.to_string());
            }
            if let Ok(speed) = parts[3].parse::<f32>() {
                message = message.with_data("speed_kts".to_string(), speed.to_string());
            }
            if let Ok(course) = parts[4].parse::<f32>() {
                message = message.with_data("course_deg".to_string(), course.to_string());
            }
            if let Ok(cpa) = parts[5].split('*').next().unwrap_or("").parse::<f32>() {
                message = message.with_data("cpa_nm".to_string(), cpa.to_string());
            }

            message = message.with_data("sentence_type".to_string(), "$RADTG".to_string());
            Some(message)
        } else {
            None
        }
    }

    fn parse_radar_scan(sentence: &str) -> Option<DataMessage> {
        // Example: $RADSC,123.45,12.0,AUTO,-15,OFF*7A
        // Format: $RADSC,sweep_angle,range_nm,gain,sea_clutter_db,rain_clutter*checksum
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() >= 6 && parts[0] == "$RADSC" {
            let mut message = DataMessage::new(
                "RADAR_SCAN".to_string(),
                "RADAR_RECEIVER".to_string(),
                sentence.as_bytes().to_vec(),
            );

            if let Ok(sweep_angle) = parts[1].parse::<f32>() {
                message = message.with_data("sweep_angle".to_string(), sweep_angle.to_string());
            }
            if let Ok(range) = parts[2].parse::<f32>() {
                message = message.with_data("range_nm".to_string(), range.to_string());
            }
            message = message.with_data("gain".to_string(), parts[3].to_string());
            if let Ok(sea_clutter) = parts[4].parse::<i8>() {
                message = message.with_data("sea_clutter_db".to_string(), sea_clutter.to_string());
            }
            message = message.with_data("rain_clutter".to_string(), parts[5].split('*').next().unwrap_or("").to_string());

            message = message.with_data("sentence_type".to_string(), "$RADSC".to_string());
            Some(message)
        } else {
            None
        }
    }

    fn parse_radar_config(sentence: &str) -> Option<DataMessage> {
        // Example: $RADCF,12.0,AUTO,-15,OFF*7A
        // Format: $RADCF,range_nm,gain,sea_clutter_db,rain_clutter*checksum
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() >= 5 && parts[0] == "$RADCF" {
            let mut message = DataMessage::new(
                "RADAR_CONFIG".to_string(),
                "RADAR_RECEIVER".to_string(),
                sentence.as_bytes().to_vec(),
            );

            if let Ok(range) = parts[1].parse::<f32>() {
                message = message.with_data("range_nm".to_string(), range.to_string());
            }
            message = message.with_data("gain".to_string(), parts[2].to_string());
            if let Ok(sea_clutter) = parts[3].parse::<i8>() {
                message = message.with_data("sea_clutter_db".to_string(), sea_clutter.to_string());
            }
            message = message.with_data("rain_clutter".to_string(), parts[4].split('*').next().unwrap_or("").to_string());

            message = message.with_data("sentence_type".to_string(), "$RADCF".to_string());
            Some(message)
        } else {
            None
        }
    }

    fn parse_radar_status(sentence: &str) -> Option<DataMessage> {
        // Example: $RADST,ACTIVE,OK*7A
        // Format: $RADST,status,health*checksum
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() >= 3 && parts[0] == "$RADST" {
            let mut message = DataMessage::new(
                "RADAR_STATUS".to_string(),
                "RADAR_RECEIVER".to_string(),
                sentence.as_bytes().to_vec(),
            );

            message = message.with_data("status".to_string(), parts[1].to_string());
            message = message.with_data("health".to_string(), parts[2].split('*').next().unwrap_or("").to_string());
            message = message.with_data("sentence_type".to_string(), "$RADST".to_string());
            Some(message)
        } else {
            None
        }
    }

    fn stop_receiver(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.try_send(());
        }
        if let Some(handle) = self.receiver_handle.take() {
            handle.abort();
        }
        self.status = DataLinkStatus::Disconnected;
    }
}

impl Default for RadarDataLinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLinkReceiver for RadarDataLinkProvider {
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
        info!("Connecting radar datalink with config: {:?}", config);

        let source_config = Self::parse_source_config(config)?;
        self.config = Some(source_config);
        self.status = DataLinkStatus::Connecting;

        match self.start_receiver() {
            Ok(()) => {
                info!("Radar datalink connected successfully");
                Ok(())
            }
            Err(e) => {
                self.status = DataLinkStatus::Error(format!("Connection failed: {}", e));
                Err(e)
            }
        }
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        info!("Disconnecting radar datalink");
        self.stop_receiver();
        self.config = None;

        // Clear message queue
        if let Ok(mut queue) = self.message_queue.lock() {
            queue.clear();
        }

        info!("Radar datalink disconnected");
        Ok(())
    }
}

impl DataLinkTransmitter for RadarDataLinkProvider {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn send_message(&mut self, _message: &DataMessage) -> DataLinkResult<()> {
        // Radar is typically receive-only, but we could implement radar control commands here
        Err(DataLinkError::TransportError("Radar transmission not supported".to_string()))
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        DataLinkReceiver::connect(self, config)
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        DataLinkReceiver::disconnect(self)
    }
}
