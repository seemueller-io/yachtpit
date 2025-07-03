//! Error types for the hardware abstraction layer

use thiserror::Error;

/// Result type alias for hardware operations
pub type Result<T> = std::result::Result<T, HardwareError>;

/// Common error types for hardware operations
#[derive(Error, Debug)]
pub enum HardwareError {
    /// Device not found on the bus
    #[error("Device not found: {device_id}")]
    DeviceNotFound { device_id: String },

    /// Bus communication error
    #[error("Bus communication error: {message}")]
    BusCommunicationError { message: String },

    /// Device is not responding
    #[error("Device not responding: {device_id}")]
    DeviceNotResponding { device_id: String },

    /// Invalid device capability
    #[error("Invalid device capability: {capability}")]
    InvalidCapability { capability: String },

    /// Discovery protocol error
    #[error("Discovery protocol error: {message}")]
    DiscoveryError { message: String },

    /// Device initialization error
    #[error("Device initialization failed: {device_id}, reason: {reason}")]
    InitializationError { device_id: String, reason: String },

    /// Serialization/Deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Generic hardware error
    #[error("Hardware error: {message}")]
    Generic { message: String },
}

impl HardwareError {
    /// Create a new generic hardware error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create a new bus communication error
    pub fn bus_communication(message: impl Into<String>) -> Self {
        Self::BusCommunicationError {
            message: message.into(),
        }
    }

    /// Create a new device not found error
    pub fn device_not_found(device_id: impl Into<String>) -> Self {
        Self::DeviceNotFound {
            device_id: device_id.into(),
        }
    }

    /// Create a new discovery error
    pub fn discovery_error(message: impl Into<String>) -> Self {
        Self::DiscoveryError {
            message: message.into(),
        }
    }
}
