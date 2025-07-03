//! System Device Module
//! 
//! Defines the interface and behavior for virtual hardware devices

use crate::{BusAddress, BusMessage, HardwareError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Device capabilities that can be advertised
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceCapability {
    /// GPS positioning capability
    Gps,
    /// Radar detection capability
    Radar,
    /// AIS (Automatic Identification System) capability
    Ais,
    /// Engine monitoring capability
    Engine,
    /// Navigation capability
    Navigation,
    /// Communication capability
    Communication,
    /// Sensor data capability
    Sensor,
    /// Custom capability with name
    Custom(String),
}

impl DeviceCapability {
    /// Get the capability name as a string
    pub fn name(&self) -> &str {
        match self {
            DeviceCapability::Gps => "GPS",
            DeviceCapability::Radar => "Radar",
            DeviceCapability::Ais => "AIS",
            DeviceCapability::Engine => "Engine",
            DeviceCapability::Navigation => "Navigation",
            DeviceCapability::Communication => "Communication",
            DeviceCapability::Sensor => "Sensor",
            DeviceCapability::Custom(name) => name,
        }
    }
}

/// Current status of a device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Device is initializing
    Initializing,
    /// Device is online and operational
    Online,
    /// Device is offline
    Offline,
    /// Device has encountered an error
    Error { message: String },
    /// Device is in maintenance mode
    Maintenance,
}

/// Device configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Device name
    pub name: String,
    /// Device capabilities
    pub capabilities: Vec<DeviceCapability>,
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Maximum message queue size
    pub max_queue_size: usize,
    /// Device-specific configuration
    pub custom_config: HashMap<String, String>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            name: "Unknown Device".to_string(),
            capabilities: vec![],
            update_interval_ms: 1000,
            max_queue_size: 100,
            custom_config: HashMap::new(),
        }
    }
}

/// Device information for discovery and identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device address
    pub address: BusAddress,
    /// Device configuration
    pub config: DeviceConfig,
    /// Current status
    pub status: DeviceStatus,
    /// Last seen timestamp
    pub last_seen: SystemTime,
    /// Device version
    pub version: String,
    /// Manufacturer information
    pub manufacturer: String,
}

/// Trait for implementing system devices
#[async_trait::async_trait]
pub trait SystemDevice: Send + Sync {
    /// Initialize the device
    async fn initialize(&mut self) -> Result<()>;

    /// Start the device operation
    async fn start(&mut self) -> Result<()>;

    /// Stop the device operation
    async fn stop(&mut self) -> Result<()>;

    /// Get device information
    fn get_info(&self) -> DeviceInfo;

    /// Get current device status
    fn get_status(&self) -> DeviceStatus;

    /// Handle incoming bus message
    async fn handle_message(&mut self, message: BusMessage) -> Result<Option<BusMessage>>;

    /// Process device-specific logic (called periodically)
    async fn process(&mut self) -> Result<Vec<BusMessage>>;

    /// Get device capabilities
    fn get_capabilities(&self) -> Vec<DeviceCapability>;

    /// Update device configuration
    async fn update_config(&mut self, config: DeviceConfig) -> Result<()>;
}

/// Base implementation for system devices
pub struct BaseSystemDevice {
    pub info: DeviceInfo,
    pub message_sender: Option<mpsc::UnboundedSender<BusMessage>>,
    pub message_receiver: Option<mpsc::UnboundedReceiver<BusMessage>>,
    pub is_running: bool,
}

impl BaseSystemDevice {
    /// Create a new base system device
    pub fn new(config: DeviceConfig) -> Self {
        let address = BusAddress::new(&config.name);
        let info = DeviceInfo {
            address,
            config,
            status: DeviceStatus::Initializing,
            last_seen: SystemTime::now(),
            version: "1.0.0".to_string(),
            manufacturer: "Virtual Hardware".to_string(),
        };

        Self {
            info,
            message_sender: None,
            message_receiver: None,
            is_running: false,
        }
    }

    /// Set the message channels
    pub fn set_message_channels(
        &mut self,
        sender: mpsc::UnboundedSender<BusMessage>,
        receiver: mpsc::UnboundedReceiver<BusMessage>,
    ) {
        self.message_sender = Some(sender);
        self.message_receiver = Some(receiver);
    }

    /// Send a message through the bus
    pub async fn send_message(&self, message: BusMessage) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message).map_err(|_| {
                HardwareError::bus_communication("Failed to send message from device")
            })?;
        } else {
            return Err(HardwareError::generic("Device not connected to bus"));
        }
        Ok(())
    }

    /// Update device status
    pub fn set_status(&mut self, status: DeviceStatus) {
        self.info.status = status;
        self.info.last_seen = SystemTime::now();
    }

    /// Check if device is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

#[async_trait::async_trait]
impl SystemDevice for BaseSystemDevice {
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing device: {}", self.info.config.name);
        self.set_status(DeviceStatus::Initializing);

        // Simulate initialization delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.set_status(DeviceStatus::Online);
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting device: {}", self.info.config.name);
        self.is_running = true;
        self.set_status(DeviceStatus::Online);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping device: {}", self.info.config.name);
        self.is_running = false;
        self.set_status(DeviceStatus::Offline);
        Ok(())
    }

    fn get_info(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn get_status(&self) -> DeviceStatus {
        self.info.status.clone()
    }

    async fn handle_message(&mut self, message: BusMessage) -> Result<Option<BusMessage>> {
        debug!("Device {} received message: {:?}", self.info.config.name, message);

        match message {
            BusMessage::Control { command, .. } => {
                match command {
                    crate::bus::ControlCommand::Ping { target } => {
                        if target == self.info.address {
                            let pong = BusMessage::Control {
                                from: self.info.address.clone(),
                                command: crate::bus::ControlCommand::Pong {
                                    from: self.info.address.clone(),
                                },
                                message_id: Uuid::new_v4(),
                            };
                            return Ok(Some(pong));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(None)
    }

    async fn process(&mut self) -> Result<Vec<BusMessage>> {
        // Base implementation does nothing
        Ok(vec![])
    }

    fn get_capabilities(&self) -> Vec<DeviceCapability> {
        self.info.config.capabilities.clone()
    }

    async fn update_config(&mut self, config: DeviceConfig) -> Result<()> {
        info!("Updating config for device: {}", self.info.config.name);
        self.info.config = config;
        Ok(())
    }
}

/// Device manager for handling multiple devices
pub struct DeviceManager {
    devices: HashMap<BusAddress, Box<dyn SystemDevice>>,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Add a device to the manager
    pub fn add_device(&mut self, device: Box<dyn SystemDevice>) {
        let address = device.get_info().address.clone();
        self.devices.insert(address, device);
    }

    /// Remove a device from the manager
    pub fn remove_device(&mut self, address: &BusAddress) -> Option<Box<dyn SystemDevice>> {
        self.devices.remove(address)
    }

    /// Get a device by address
    pub fn get_device(&self, address: &BusAddress) -> Option<&dyn SystemDevice> {
        self.devices.get(address).map(|d| d.as_ref())
    }

    /// Get a mutable device by address
    pub fn get_device_mut(&mut self, address: &BusAddress) -> Option<&mut Box<dyn SystemDevice>> {
        self.devices.get_mut(address)
    }

    /// Initialize all devices
    pub async fn initialize_all(&mut self) -> Result<()> {
        for device in self.devices.values_mut() {
            device.initialize().await?;
        }
        Ok(())
    }

    /// Start all devices
    pub async fn start_all(&mut self) -> Result<()> {
        for device in self.devices.values_mut() {
            device.start().await?;
        }
        Ok(())
    }

    /// Stop all devices
    pub async fn stop_all(&mut self) -> Result<()> {
        for device in self.devices.values_mut() {
            device.stop().await?;
        }
        Ok(())
    }

    /// Get all device information
    pub fn get_all_device_info(&self) -> Vec<DeviceInfo> {
        self.devices.values().map(|d| d.get_info()).collect()
    }

    /// Process all devices
    pub async fn process_all(&mut self) -> Result<Vec<BusMessage>> {
        let mut messages = Vec::new();
        for device in self.devices.values_mut() {
            let device_messages = device.process().await?;
            messages.extend(device_messages);
        }
        Ok(messages)
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_creation() {
        let config = DeviceConfig {
            name: "Test Device".to_string(),
            capabilities: vec![DeviceCapability::Gps],
            ..Default::default()
        };

        let device = BaseSystemDevice::new(config);
        assert_eq!(device.info.config.name, "Test Device");
        assert_eq!(device.info.status, DeviceStatus::Initializing);
    }

    #[tokio::test]
    async fn test_device_initialization() {
        let config = DeviceConfig {
            name: "Test Device".to_string(),
            ..Default::default()
        };

        let mut device = BaseSystemDevice::new(config);
        device.initialize().await.unwrap();
        assert_eq!(device.get_status(), DeviceStatus::Online);
    }

    #[tokio::test]
    async fn test_device_start_stop() {
        let config = DeviceConfig {
            name: "Test Device".to_string(),
            ..Default::default()
        };

        let mut device = BaseSystemDevice::new(config);

        device.start().await.unwrap();
        assert!(device.is_running());
        assert_eq!(device.get_status(), DeviceStatus::Online);

        device.stop().await.unwrap();
        assert!(!device.is_running());
        assert_eq!(device.get_status(), DeviceStatus::Offline);
    }

    #[tokio::test]
    async fn test_device_manager() {
        let mut manager = DeviceManager::new();

        let config = DeviceConfig {
            name: "Test Device".to_string(),
            ..Default::default()
        };

        let device = Box::new(BaseSystemDevice::new(config));
        let address = device.get_info().address.clone();

        manager.add_device(device);
        assert!(manager.get_device(&address).is_some());

        manager.initialize_all().await.unwrap();
        manager.start_all().await.unwrap();

        let info = manager.get_all_device_info();
        assert_eq!(info.len(), 1);
        assert_eq!(info[0].status, DeviceStatus::Online);
    }
}
