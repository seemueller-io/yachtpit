//! Virtual Data-Link Abstraction
//! 
//! This crate provides a common abstraction for data communication links
//! that can be used by various vessel systems like AIS, GPS, Radar, etc.
//! 
//! The abstraction allows systems to receive and transmit data through
//! different transport mechanisms (serial, network, simulation, etc.)
//! without being tightly coupled to the specific implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Errors that can occur in the data-link layer
#[derive(Error, Debug)]
pub enum DataLinkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Data parsing error: {0}")]
    ParseError(String),
    #[error("Timeout occurred")]
    Timeout,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Transport error: {0}")]
    TransportError(String),
}

/// Result type for data-link operations
pub type DataLinkResult<T> = Result<T, DataLinkError>;

/// Represents a generic data message that can be transmitted over the data-link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMessage {
    /// Unique identifier for the message type
    pub message_type: String,
    /// Source identifier (e.g., MMSI for AIS, device ID for GPS)
    pub source_id: String,
    /// Timestamp when the message was created/received
    pub timestamp: SystemTime,
    /// Raw message payload
    pub payload: Vec<u8>,
    /// Parsed message data as key-value pairs
    pub data: HashMap<String, String>,
    /// Signal strength or quality indicator (0-100)
    pub signal_quality: Option<u8>,
}

impl DataMessage {
    /// Create a new data message
    pub fn new(message_type: String, source_id: String, payload: Vec<u8>) -> Self {
        Self {
            message_type,
            source_id,
            timestamp: SystemTime::now(),
            payload,
            data: HashMap::new(),
            signal_quality: None,
        }
    }

    /// Add parsed data to the message
    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }

    /// Set signal quality
    pub fn with_signal_quality(mut self, quality: u8) -> Self {
        self.signal_quality = Some(quality.min(100));
        self
    }

    /// Get a data value by key
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Configuration for a data-link connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLinkConfig {
    /// Connection type (e.g., "serial", "tcp", "udp", "simulation")
    pub connection_type: String,
    /// Connection parameters (port, baud rate, IP address, etc.)
    pub parameters: HashMap<String, String>,
    /// Timeout for operations
    pub timeout: Duration,
    /// Whether to auto-reconnect on failure
    pub auto_reconnect: bool,
}

impl DataLinkConfig {
    /// Create a new configuration
    pub fn new(connection_type: String) -> Self {
        Self {
            connection_type,
            parameters: HashMap::new(),
            timeout: Duration::from_secs(5),
            auto_reconnect: true,
        }
    }

    /// Add a parameter to the configuration
    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Status of a data-link connection
#[derive(Debug, Clone, PartialEq)]
pub enum DataLinkStatus {
    /// Connection is active and receiving data
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is disconnected
    Disconnected,
    /// Connection has an error
    Error(String),
}

/// Trait for data-link receivers that can receive messages
pub trait DataLinkReceiver: Send + Sync {
    /// Get the current status of the data-link
    fn status(&self) -> DataLinkStatus;

    /// Receive the next available message, if any
    fn receive_message(&mut self) -> DataLinkResult<Option<DataMessage>>;

    /// Receive all available messages
    fn receive_all_messages(&mut self) -> DataLinkResult<Vec<DataMessage>> {
        let mut messages = Vec::new();
        while let Some(message) = self.receive_message()? {
            messages.push(message);
        }
        Ok(messages)
    }

    /// Connect to the data source
    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()>;

    /// Disconnect from the data source
    fn disconnect(&mut self) -> DataLinkResult<()>;

    /// Check if the connection is active
    fn is_connected(&self) -> bool {
        matches!(self.status(), DataLinkStatus::Connected)
    }
}

/// Trait for data-link transmitters that can send messages
pub trait DataLinkTransmitter: Send + Sync {
    /// Get the current status of the data-link
    fn status(&self) -> DataLinkStatus;

    /// Send a message through the data-link
    fn send_message(&mut self, message: &DataMessage) -> DataLinkResult<()>;

    /// Connect to the data destination
    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()>;

    /// Disconnect from the data destination
    fn disconnect(&mut self) -> DataLinkResult<()>;

    /// Check if the connection is active
    fn is_connected(&self) -> bool {
        matches!(self.status(), DataLinkStatus::Connected)
    }
}

/// Combined trait for bidirectional data-links
pub trait DataLink: DataLinkReceiver + DataLinkTransmitter {}

/// Automatic implementation for types that implement both receiver and transmitter
impl<T> DataLink for T where T: DataLinkReceiver + DataLinkTransmitter {}

/// A simulation data-link for testing and demonstration purposes
pub struct SimulationDataLink {
    status: DataLinkStatus,
    config: Option<DataLinkConfig>,
    message_queue: Vec<DataMessage>,
}

impl SimulationDataLink {
    /// Create a new simulation data-link
    pub fn new() -> Self {
        Self {
            status: DataLinkStatus::Disconnected,
            config: None,
            message_queue: Vec::new(),
        }
    }

    /// Add a simulated message to the queue
    pub fn add_simulated_message(&mut self, message: DataMessage) {
        self.message_queue.push(message);
    }

    /// Generate sample AIS messages for testing
    pub fn generate_sample_ais_messages(&mut self) {
        let messages = vec![
            DataMessage::new(
                "AIS_POSITION".to_string(),
                "987654321".to_string(),
                b"!AIVDM,1,1,,A,15M8J7001G?UJH@E=4R0S>0@0<0M,0*7B".to_vec(),
            )
            .with_data("vessel_name".to_string(), "M/Y SERENITY".to_string())
            .with_data("mmsi".to_string(), "987654321".to_string())
            .with_data("latitude".to_string(), "37.7749".to_string())
            .with_data("longitude".to_string(), "-122.4194".to_string())
            .with_data("speed".to_string(), "12.5".to_string())
            .with_data("course".to_string(), "180".to_string())
            .with_signal_quality(85),

            DataMessage::new(
                "AIS_POSITION".to_string(),
                "456789123".to_string(),
                b"!AIVDM,1,1,,A,15M8J7001G?UJH@E=4R0S>0@0<0M,0*7B".to_vec(),
            )
            .with_data("vessel_name".to_string(), "CARGO VESSEL ATLANTIS".to_string())
            .with_data("mmsi".to_string(), "456789123".to_string())
            .with_data("latitude".to_string(), "37.7849".to_string())
            .with_data("longitude".to_string(), "-122.4094".to_string())
            .with_data("speed".to_string(), "18.2".to_string())
            .with_data("course".to_string(), "090".to_string())
            .with_signal_quality(92),

            DataMessage::new(
                "AIS_POSITION".to_string(),
                "789123456".to_string(),
                b"!AIVDM,1,1,,A,15M8J7001G?UJH@E=4R0S>0@0M,0*7B".to_vec(),
            )
            .with_data("vessel_name".to_string(), "S/Y WIND DANCER".to_string())
            .with_data("mmsi".to_string(), "789123456".to_string())
            .with_data("latitude".to_string(), "37.7649".to_string())
            .with_data("longitude".to_string(), "-122.4294".to_string())
            .with_data("speed".to_string(), "6.8".to_string())
            .with_data("course".to_string(), "225".to_string())
            .with_signal_quality(78),
        ];

        for message in messages {
            self.message_queue.push(message);
        }
    }
}

impl Default for SimulationDataLink {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLinkReceiver for SimulationDataLink {
    fn status(&self) -> DataLinkStatus {
        self.status.clone()
    }

    fn receive_message(&mut self) -> DataLinkResult<Option<DataMessage>> {
        if matches!(self.status, DataLinkStatus::Connected) && !self.message_queue.is_empty() {
            Ok(Some(self.message_queue.remove(0)))
        } else {
            Ok(None)
        }
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        if config.connection_type == "simulation" {
            self.config = Some(config.clone());
            self.status = DataLinkStatus::Connected;
            // Generate some sample messages when connecting
            self.generate_sample_ais_messages();
            Ok(())
        } else {
            Err(DataLinkError::InvalidConfig(
                "SimulationDataLink only supports 'simulation' connection type".to_string(),
            ))
        }
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        self.status = DataLinkStatus::Disconnected;
        self.config = None;
        self.message_queue.clear();
        Ok(())
    }
}

impl DataLinkTransmitter for SimulationDataLink {
    fn status(&self) -> DataLinkStatus {
        // Delegate to the receiver implementation
        <Self as DataLinkReceiver>::status(self)
    }

    fn send_message(&mut self, _message: &DataMessage) -> DataLinkResult<()> {
        if <Self as DataLinkReceiver>::is_connected(self) {
            // In simulation mode, we just acknowledge the send
            Ok(())
        } else {
            Err(DataLinkError::ConnectionFailed(
                "Not connected".to_string(),
            ))
        }
    }

    fn connect(&mut self, config: &DataLinkConfig) -> DataLinkResult<()> {
        <Self as DataLinkReceiver>::connect(self, config)
    }

    fn disconnect(&mut self) -> DataLinkResult<()> {
        <Self as DataLinkReceiver>::disconnect(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_message_creation() {
        let message = DataMessage::new(
            "TEST".to_string(),
            "123".to_string(),
            b"test payload".to_vec(),
        )
        .with_data("key1".to_string(), "value1".to_string())
        .with_signal_quality(75);

        assert_eq!(message.message_type, "TEST");
        assert_eq!(message.source_id, "123");
        assert_eq!(message.get_data("key1"), Some(&"value1".to_string()));
        assert_eq!(message.signal_quality, Some(75));
    }

    #[test]
    fn test_simulation_datalink() {
        let mut datalink = SimulationDataLink::new();
        let config = DataLinkConfig::new("simulation".to_string());

        assert_eq!(<SimulationDataLink as DataLinkReceiver>::status(&datalink), DataLinkStatus::Disconnected);
        assert!(!<SimulationDataLink as DataLinkReceiver>::is_connected(&datalink));

        <SimulationDataLink as DataLinkReceiver>::connect(&mut datalink, &config).unwrap();
        assert_eq!(<SimulationDataLink as DataLinkReceiver>::status(&datalink), DataLinkStatus::Connected);
        assert!(<SimulationDataLink as DataLinkReceiver>::is_connected(&datalink));

        // Should have sample messages after connecting
        let messages = <SimulationDataLink as DataLinkReceiver>::receive_all_messages(&mut datalink).unwrap();
        assert!(!messages.is_empty());
        assert!(messages.iter().any(|m| m.message_type == "AIS_POSITION"));

        <SimulationDataLink as DataLinkReceiver>::disconnect(&mut datalink).unwrap();
        assert_eq!(<SimulationDataLink as DataLinkReceiver>::status(&datalink), DataLinkStatus::Disconnected);
    }

    #[test]
    fn test_datalink_config() {
        let config = DataLinkConfig::new("tcp".to_string())
            .with_parameter("host".to_string(), "localhost".to_string())
            .with_parameter("port".to_string(), "4001".to_string())
            .with_timeout(Duration::from_secs(10));

        assert_eq!(config.connection_type, "tcp");
        assert_eq!(config.parameters.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(10));
    }
}
