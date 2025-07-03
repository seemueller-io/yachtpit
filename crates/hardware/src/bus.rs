//! Virtual Hardware Bus Module
//! 
//! Provides a communication infrastructure for virtual hardware devices

use crate::{HardwareError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Unique address for devices on the hardware bus
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BusAddress {
    pub id: Uuid,
    pub name: String,
}

impl BusAddress {
    /// Create a new bus address with a generated UUID
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
        }
    }

    /// Create a bus address with a specific UUID
    pub fn with_id(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}

/// Message types that can be sent over the hardware bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusMessage {
    /// Data message with payload
    Data {
        from: BusAddress,
        to: BusAddress,
        payload: Vec<u8>,
        message_id: Uuid,
    },
    /// Control message for bus management
    Control {
        from: BusAddress,
        command: ControlCommand,
        message_id: Uuid,
    },
    /// Broadcast message to all devices
    Broadcast {
        from: BusAddress,
        payload: Vec<u8>,
        message_id: Uuid,
    },
    /// Acknowledgment message
    Ack {
        to: BusAddress,
        original_message_id: Uuid,
        message_id: Uuid,
    },
}

/// Control commands for bus management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    /// Register a device on the bus
    Register { address: BusAddress },
    /// Unregister a device from the bus
    Unregister { address: BusAddress },
    /// Ping a device
    Ping { target: BusAddress },
    /// Pong response to ping
    Pong { from: BusAddress },
    /// Request device list
    ListDevices,
    /// Response with device list
    DeviceList { devices: Vec<BusAddress> },
}

impl BusMessage {
    /// Get the message ID
    pub fn message_id(&self) -> Uuid {
        match self {
            BusMessage::Data { message_id, .. } => *message_id,
            BusMessage::Control { message_id, .. } => *message_id,
            BusMessage::Broadcast { message_id, .. } => *message_id,
            BusMessage::Ack { message_id, .. } => *message_id,
        }
    }

    /// Get the sender address if available
    pub fn from(&self) -> Option<&BusAddress> {
        match self {
            BusMessage::Data { from, .. } => Some(from),
            BusMessage::Control { from, .. } => Some(from),
            BusMessage::Broadcast { from, .. } => Some(from),
            BusMessage::Ack { .. } => None,
        }
    }
}

/// Device connection handle for the hardware bus
pub struct DeviceConnection {
    pub address: BusAddress,
    pub sender: mpsc::UnboundedSender<BusMessage>,
    pub receiver: mpsc::UnboundedReceiver<BusMessage>,
}

/// Virtual Hardware Bus implementation
pub struct HardwareBus {
    devices: Arc<RwLock<HashMap<BusAddress, mpsc::UnboundedSender<BusMessage>>>>,
    message_log: Arc<RwLock<Vec<BusMessage>>>,
}

impl Default for HardwareBus {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareBus {
    /// Create a new hardware bus
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            message_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Connect a device to the bus
    pub async fn connect_device(&self, address: BusAddress) -> Result<DeviceConnection> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        {
            let mut devices = self.devices.write().await;
            if devices.contains_key(&address) {
                return Err(HardwareError::generic(format!(
                    "Device {} already connected", address.name
                )));
            }
            devices.insert(address.clone(), tx.clone());
        }

        info!("Device {} connected to bus", address.name);

        // Send registration message to all other devices
        let register_msg = BusMessage::Control {
            from: address.clone(),
            command: ControlCommand::Register {
                address: address.clone(),
            },
            message_id: Uuid::new_v4(),
        };

        self.broadcast_message(register_msg).await?;

        Ok(DeviceConnection {
            address,
            sender: tx,
            receiver: rx,
        })
    }

    /// Disconnect a device from the bus
    pub async fn disconnect_device(&self, address: &BusAddress) -> Result<()> {
        {
            let mut devices = self.devices.write().await;
            devices.remove(address);
        }

        info!("Device {} disconnected from bus", address.name);

        // Send unregistration message to all other devices
        let unregister_msg = BusMessage::Control {
            from: address.clone(),
            command: ControlCommand::Unregister {
                address: address.clone(),
            },
            message_id: Uuid::new_v4(),
        };

        self.broadcast_message(unregister_msg).await?;

        Ok(())
    }

    /// Send a message to a specific device
    pub async fn send_message(&self, message: BusMessage) -> Result<()> {
        // Log the message
        {
            let mut log = self.message_log.write().await;
            log.push(message.clone());
        }

        match &message {
            BusMessage::Data { to, .. } => {
                let devices = self.devices.read().await;
                if let Some(sender) = devices.get(to) {
                    sender.send(message).map_err(|_| {
                        HardwareError::bus_communication("Failed to send message to device")
                    })?;
                } else {
                    return Err(HardwareError::device_not_found(&to.name));
                }
            }
            BusMessage::Broadcast { .. } => {
                self.broadcast_message(message).await?;
            }
            BusMessage::Control { .. } => {
                self.broadcast_message(message).await?;
            }
            BusMessage::Ack { to, .. } => {
                let devices = self.devices.read().await;
                if let Some(sender) = devices.get(to) {
                    sender.send(message).map_err(|_| {
                        HardwareError::bus_communication("Failed to send ACK to device")
                    })?;
                } else {
                    warn!("Attempted to send ACK to unknown device: {}", to.name);
                }
            }
        }

        Ok(())
    }

    /// Broadcast a message to all connected devices
    async fn broadcast_message(&self, message: BusMessage) -> Result<()> {
        let devices = self.devices.read().await;
        let sender_address = message.from();

        for (address, sender) in devices.iter() {
            // Don't send message back to sender
            if let Some(from) = sender_address {
                if address == from {
                    continue;
                }
            }

            if let Err(_) = sender.send(message.clone()) {
                error!("Failed to broadcast message to device: {}", address.name);
            }
        }

        Ok(())
    }

    /// Get list of connected devices
    pub async fn get_connected_devices(&self) -> Vec<BusAddress> {
        let devices = self.devices.read().await;
        devices.keys().cloned().collect()
    }

    /// Get message history
    pub async fn get_message_history(&self) -> Vec<BusMessage> {
        let log = self.message_log.read().await;
        log.clone()
    }

    /// Clear message history
    pub async fn clear_message_history(&self) {
        let mut log = self.message_log.write().await;
        log.clear();
    }

    /// Check if a device is connected
    pub async fn is_device_connected(&self, address: &BusAddress) -> bool {
        let devices = self.devices.read().await;
        devices.contains_key(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_bus_creation() {
        let bus = HardwareBus::new();
        assert_eq!(bus.get_connected_devices().await.len(), 0);
    }

    #[tokio::test]
    async fn test_device_connection() {
        let bus = HardwareBus::new();
        let address = BusAddress::new("test_device");
        
        let connection = bus.connect_device(address.clone()).await.unwrap();
        assert_eq!(connection.address, address);
        assert!(bus.is_device_connected(&address).await);
    }

    #[tokio::test]
    async fn test_device_disconnection() {
        let bus = HardwareBus::new();
        let address = BusAddress::new("test_device");
        
        let _connection = bus.connect_device(address.clone()).await.unwrap();
        assert!(bus.is_device_connected(&address).await);
        
        bus.disconnect_device(&address).await.unwrap();
        assert!(!bus.is_device_connected(&address).await);
    }

    #[tokio::test]
    async fn test_message_sending() {
        let bus = HardwareBus::new();
        let addr1 = BusAddress::new("device1");
        let addr2 = BusAddress::new("device2");
        
        let mut conn1 = bus.connect_device(addr1.clone()).await.unwrap();
        let _conn2 = bus.connect_device(addr2.clone()).await.unwrap();
        
        let message = BusMessage::Data {
            from: addr2.clone(),
            to: addr1.clone(),
            payload: b"test data".to_vec(),
            message_id: Uuid::new_v4(),
        };
        
        bus.send_message(message.clone()).await.unwrap();
        
        // Check if message was received
        let received = conn1.receiver.recv().await.unwrap();
        match received {
            BusMessage::Data { payload, .. } => {
                assert_eq!(payload, b"test data");
            }
            _ => panic!("Expected data message"),
        }
    }
}