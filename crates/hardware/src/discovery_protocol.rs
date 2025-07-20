//! Discovery Protocol Module
//! 
//! Provides device discovery and capability advertisement functionality

use crate::{BusAddress, BusMessage, DeviceCapability, DeviceInfo, HardwareError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;
use std::sync::Arc;

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    /// Announce device presence and capabilities
    Announce {
        device_info: DeviceInfo,
        timestamp: SystemTime,
    },
    /// Request device information
    Discover {
        requester: BusAddress,
        filter: Option<DiscoveryFilter>,
        timestamp: SystemTime,
    },
    /// Response to discovery request
    DiscoverResponse {
        devices: Vec<DeviceInfo>,
        responder: BusAddress,
        timestamp: SystemTime,
    },
    /// Heartbeat to maintain presence
    Heartbeat {
        device: BusAddress,
        timestamp: SystemTime,
    },
    /// Device going offline notification
    Goodbye {
        device: BusAddress,
        timestamp: SystemTime,
    },
}

/// Filter criteria for device discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryFilter {
    /// Filter by device capabilities
    pub capabilities: Option<Vec<DeviceCapability>>,
    /// Filter by device name pattern
    pub name_pattern: Option<String>,
    /// Filter by manufacturer
    pub manufacturer: Option<String>,
    /// Filter by minimum version
    pub min_version: Option<String>,
}

impl DiscoveryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self {
            capabilities: None,
            name_pattern: None,
            manufacturer: None,
            min_version: None,
        }
    }

    /// Filter by capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<DeviceCapability>) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Filter by name pattern
    pub fn with_name_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    /// Filter by manufacturer
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }

    /// Check if device matches this filter
    pub fn matches(&self, device_info: &DeviceInfo) -> bool {
        // Check capabilities
        if let Some(required_caps) = &self.capabilities {
            let device_caps = &device_info.config.capabilities;
            if !required_caps.iter().all(|cap| device_caps.contains(cap)) {
                return false;
            }
        }

        // Check name pattern (simple substring match)
        if let Some(pattern) = &self.name_pattern {
            if !device_info.config.name.contains(pattern) {
                return false;
            }
        }

        // Check manufacturer
        if let Some(manufacturer) = &self.manufacturer {
            if device_info.manufacturer != *manufacturer {
                return false;
            }
        }

        // Check version (simple string comparison for now)
        if let Some(min_version) = &self.min_version {
            if device_info.version < *min_version {
                return false;
            }
        }

        true
    }
}

impl Default for DiscoveryFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Discovery protocol configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// How often to send heartbeat messages (in seconds)
    pub heartbeat_interval: Duration,
    /// How long to wait before considering a device offline (in seconds)
    pub device_timeout: Duration,
    /// How often to clean up expired devices (in seconds)
    pub cleanup_interval: Duration,
    /// Maximum number of devices to track
    pub max_devices: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            device_timeout: Duration::from_secs(90),
            cleanup_interval: Duration::from_secs(60),
            max_devices: 1000,
        }
    }
}

/// Discovery protocol implementation
pub struct DiscoveryProtocol {
    /// Local device information
    local_device: DeviceInfo,
    /// Known devices registry
    known_devices: Arc<RwLock<HashMap<BusAddress, DeviceInfo>>>,
    /// Configuration
    config: DiscoveryConfig,
    /// Message sender for bus communication
    message_sender: Option<mpsc::UnboundedSender<BusMessage>>,
    /// Discovery message receiver
    discovery_receiver: Option<mpsc::UnboundedReceiver<DiscoveryMessage>>,
    /// Running state
    is_running: bool,
}

impl DiscoveryProtocol {
    /// Create a new discovery protocol instance
    pub fn new(local_device: DeviceInfo, config: DiscoveryConfig) -> Self {
        Self {
            local_device,
            known_devices: Arc::new(RwLock::new(HashMap::new())),
            config,
            message_sender: None,
            discovery_receiver: None,
            is_running: false,
        }
    }

    /// Set the message sender for bus communication
    pub fn set_message_sender(&mut self, sender: mpsc::UnboundedSender<BusMessage>) {
        self.message_sender = Some(sender);
    }

    /// Set the discovery message receiver
    pub fn set_discovery_receiver(&mut self, receiver: mpsc::UnboundedReceiver<DiscoveryMessage>) {
        self.discovery_receiver = Some(receiver);
    }

    /// Start the discovery protocol
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting discovery protocol for device: {}", self.local_device.config.name);
        self.is_running = true;

        // Send initial announcement
        self.announce_device().await?;

        Ok(())
    }

    /// Stop the discovery protocol
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping discovery protocol for device: {}", self.local_device.config.name);
        self.is_running = false;

        // Send goodbye message
        self.send_goodbye().await?;

        Ok(())
    }

    /// Announce this device to the network
    pub async fn announce_device(&self) -> Result<()> {
        let announcement = DiscoveryMessage::Announce {
            device_info: self.local_device.clone(),
            timestamp: SystemTime::now(),
        };

        self.send_discovery_message(announcement).await
    }

    /// Send heartbeat to maintain presence
    pub async fn send_heartbeat(&self) -> Result<()> {
        let heartbeat = DiscoveryMessage::Heartbeat {
            device: self.local_device.address.clone(),
            timestamp: SystemTime::now(),
        };

        self.send_discovery_message(heartbeat).await
    }

    /// Send goodbye message when going offline
    pub async fn send_goodbye(&self) -> Result<()> {
        let goodbye = DiscoveryMessage::Goodbye {
            device: self.local_device.address.clone(),
            timestamp: SystemTime::now(),
        };

        self.send_discovery_message(goodbye).await
    }

    /// Discover devices on the network
    pub async fn discover_devices(&self, filter: Option<DiscoveryFilter>) -> Result<()> {
        let discover_msg = DiscoveryMessage::Discover {
            requester: self.local_device.address.clone(),
            filter,
            timestamp: SystemTime::now(),
        };

        self.send_discovery_message(discover_msg).await
    }

    /// Get all known devices
    pub async fn get_known_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.known_devices.read().await;
        devices.values().cloned().collect()
    }

    /// Get devices matching a filter
    pub async fn get_devices_by_filter(&self, filter: &DiscoveryFilter) -> Vec<DeviceInfo> {
        let devices = self.known_devices.read().await;
        devices
            .values()
            .filter(|device| filter.matches(device))
            .cloned()
            .collect()
    }

    /// Get device by address
    pub async fn get_device(&self, address: &BusAddress) -> Option<DeviceInfo> {
        let devices = self.known_devices.read().await;
        devices.get(address).cloned()
    }

    /// Handle incoming discovery message
    pub async fn handle_discovery_message(&self, message: DiscoveryMessage) -> Result<()> {
        match message {
            DiscoveryMessage::Announce { device_info, .. } => {
                self.handle_device_announcement(device_info).await
            }
            DiscoveryMessage::Discover { requester, filter, .. } => {
                self.handle_discovery_request(requester, filter).await
            }
            DiscoveryMessage::DiscoverResponse { devices, .. } => {
                self.handle_discovery_response(devices).await
            }
            DiscoveryMessage::Heartbeat { device, timestamp } => {
                self.handle_heartbeat(device, timestamp).await
            }
            DiscoveryMessage::Goodbye { device, .. } => {
                self.handle_goodbye(device).await
            }
        }
    }

    /// Handle device announcement
    async fn handle_device_announcement(&self, device_info: DeviceInfo) -> Result<()> {
        info!("Device announced: {}", device_info.config.name);
        
        let mut devices = self.known_devices.write().await;
        devices.insert(device_info.address.clone(), device_info);
        
        Ok(())
    }

    /// Handle discovery request
    async fn handle_discovery_request(
        &self,
        requester: BusAddress,
        filter: Option<DiscoveryFilter>,
    ) -> Result<()> {
        debug!("Discovery request from: {}", requester.name);

        let devices = self.known_devices.read().await;
        let mut matching_devices = vec![self.local_device.clone()]; // Include self

        // Add matching known devices
        for device in devices.values() {
            if let Some(ref filter) = filter {
                if filter.matches(device) {
                    matching_devices.push(device.clone());
                }
            } else {
                matching_devices.push(device.clone());
            }
        }

        drop(devices); // Release the lock

        let response = DiscoveryMessage::DiscoverResponse {
            devices: matching_devices,
            responder: self.local_device.address.clone(),
            timestamp: SystemTime::now(),
        };

        self.send_discovery_message(response).await
    }

    /// Handle discovery response
    async fn handle_discovery_response(&self, devices: Vec<DeviceInfo>) -> Result<()> {
        debug!("Received discovery response with {} devices", devices.len());

        let mut known_devices = self.known_devices.write().await;
        for device in devices {
            // Don't add ourselves
            if device.address != self.local_device.address {
                known_devices.insert(device.address.clone(), device);
            }
        }

        Ok(())
    }

    /// Handle heartbeat message
    async fn handle_heartbeat(&self, device: BusAddress, timestamp: SystemTime) -> Result<()> {
        debug!("Heartbeat from device: {}", device.name);

        let mut devices = self.known_devices.write().await;
        if let Some(device_info) = devices.get_mut(&device) {
            device_info.last_seen = timestamp;
        }

        Ok(())
    }

    /// Handle goodbye message
    async fn handle_goodbye(&self, device: BusAddress) -> Result<()> {
        info!("Device going offline: {}", device.name);

        let mut devices = self.known_devices.write().await;
        devices.remove(&device);

        Ok(())
    }

    /// Clean up expired devices
    pub async fn cleanup_expired_devices(&self) -> Result<()> {
        let now = SystemTime::now();
        let timeout = self.config.device_timeout;

        let mut devices = self.known_devices.write().await;
        let mut expired_devices = Vec::new();

        for (address, device_info) in devices.iter() {
            if let Ok(elapsed) = now.duration_since(device_info.last_seen) {
                if elapsed > timeout {
                    expired_devices.push(address.clone());
                }
            }
        }

        for address in expired_devices {
            warn!("Removing expired device: {}", address.name);
            devices.remove(&address);
        }

        Ok(())
    }

    /// Send discovery message over the bus
    async fn send_discovery_message(&self, discovery_msg: DiscoveryMessage) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            let payload = serde_json::to_vec(&discovery_msg)?;
            
            let bus_message = BusMessage::Broadcast {
                from: self.local_device.address.clone(),
                payload,
                message_id: Uuid::new_v4(),
            };

            sender.send(bus_message).map_err(|_| {
                HardwareError::bus_communication("Failed to send discovery message")
            })?;
        } else {
            return Err(HardwareError::generic("Discovery protocol not connected to bus"));
        }

        Ok(())
    }

    /// Run the discovery protocol main loop
    pub async fn run(&mut self) -> Result<()> {
        let mut heartbeat_timer = tokio::time::interval(self.config.heartbeat_interval);
        let mut cleanup_timer = tokio::time::interval(self.config.cleanup_interval);

        while self.is_running {
            tokio::select! {
                _ = heartbeat_timer.tick() => {
                    if let Err(e) = self.send_heartbeat().await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }
                _ = cleanup_timer.tick() => {
                    if let Err(e) = self.cleanup_expired_devices().await {
                        warn!("Failed to cleanup expired devices: {}", e);
                    }
                }
                // Handle incoming discovery messages if receiver is set
                msg = async {
                    if let Some(ref mut receiver) = self.discovery_receiver {
                        receiver.recv().await
                    } else {
                        std::future::pending().await
                    }
                } => {
                    if let Some(discovery_msg) = msg {
                        if let Err(e) = self.handle_discovery_message(discovery_msg).await {
                            warn!("Failed to handle discovery message: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if the protocol is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the number of known devices
    pub async fn device_count(&self) -> usize {
        let devices = self.known_devices.read().await;
        devices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceConfig, DeviceStatus};

    fn create_test_device_info(name: &str) -> DeviceInfo {
        DeviceInfo {
            address: BusAddress::new(name),
            config: DeviceConfig {
                name: name.to_string(),
                capabilities: vec![DeviceCapability::Gps],
                ..Default::default()
            },
            status: DeviceStatus::Online,
            last_seen: SystemTime::now(),
            version: "1.0.0".to_string(),
            manufacturer: "Test Manufacturer".to_string(),
        }
    }

    #[tokio::test]
    async fn test_discovery_protocol_creation() {
        let device_info = create_test_device_info("test_device");
        let config = DiscoveryConfig::default();
        let protocol = DiscoveryProtocol::new(device_info, config);
        
        assert!(!protocol.is_running());
        assert_eq!(protocol.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_device_announcement() {
        let device_info = create_test_device_info("test_device");
        let config = DiscoveryConfig::default();
        let protocol = DiscoveryProtocol::new(device_info.clone(), config);

        let other_device = create_test_device_info("other_device");
        protocol.handle_device_announcement(other_device.clone()).await.unwrap();

        let known_devices = protocol.get_known_devices().await;
        assert_eq!(known_devices.len(), 1);
        assert_eq!(known_devices[0].config.name, "other_device");
    }

    #[tokio::test]
    async fn test_discovery_filter() {
        let filter = DiscoveryFilter::new()
            .with_capabilities(vec![DeviceCapability::Gps])
            .with_name_pattern("test");

        let device_info = create_test_device_info("test_device");
        assert!(filter.matches(&device_info));

        let other_device = DeviceInfo {
            address: BusAddress::new("other"),
            config: DeviceConfig {
                name: "other".to_string(),
                capabilities: vec![DeviceCapability::Radar],
                ..Default::default()
            },
            status: DeviceStatus::Online,
            last_seen: SystemTime::now(),
            version: "1.0.0".to_string(),
            manufacturer: "Test".to_string(),
        };
        assert!(!filter.matches(&other_device));
    }

    #[tokio::test]
    async fn test_device_cleanup() {
        let device_info = create_test_device_info("test_device");
        let mut config = DiscoveryConfig::default();
        config.device_timeout = Duration::from_millis(100);
        
        let protocol = DiscoveryProtocol::new(device_info, config);

        // Add a device with old timestamp
        let mut old_device = create_test_device_info("old_device");
        old_device.last_seen = SystemTime::now() - Duration::from_secs(200);
        
        protocol.handle_device_announcement(old_device).await.unwrap();
        assert_eq!(protocol.device_count().await, 1);

        // Wait and cleanup
        tokio::time::sleep(Duration::from_millis(150)).await;
        protocol.cleanup_expired_devices().await.unwrap();
        
        assert_eq!(protocol.device_count().await, 0);
    }
}