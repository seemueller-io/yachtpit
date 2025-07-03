//! Virtual Hardware Abstraction Layer
//! 
//! This crate provides a common abstraction for virtual hardware components
//! including a hardware bus, system devices, and discovery protocols.

#![allow(clippy::type_complexity)]

pub mod bus;
pub mod device;
pub mod discovery_protocol;
pub mod error;

// Re-export main types
pub use bus::{HardwareBus, BusMessage, BusAddress};
pub use device::{SystemDevice, DeviceCapability, DeviceStatus, DeviceInfo, DeviceConfig};
pub use discovery_protocol::{DiscoveryProtocol, DiscoveryMessage};
pub use error::{HardwareError, Result};

/// Common traits and types used throughout the hardware abstraction layer
pub mod prelude {
    pub use crate::{
        HardwareBus, BusMessage, BusAddress,
        SystemDevice, DeviceCapability, DeviceStatus, DeviceInfo, DeviceConfig,
        DiscoveryProtocol, DiscoveryMessage,
        HardwareError, Result,
    };
}
