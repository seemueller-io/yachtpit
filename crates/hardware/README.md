# Virtual Hardware Abstraction Layer - Integration Guide

This document provides detailed instructions on how to integrate the virtual hardware abstraction layer into yachtpit systems.

## Overview

The virtual hardware abstraction layer consists of three main components:

1. **Hardware Bus** - Communication infrastructure for virtual devices
2. **System Device** - Interface and base implementation for virtual hardware devices
3. **Discovery Protocol** - Device discovery and capability advertisement

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Yachtpit Application                     │
├─────────────────────────────────────────────────────────────┤
│                    Systems Crate                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ GPS System  │  │Radar System │  │ AIS System  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                Hardware Abstraction Layer                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │Hardware Bus │  │System Device│  │Discovery    │        │
│  │             │  │Interface    │  │Protocol     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## Integration Steps

### Step 1: Add Hardware Dependency

Add the hardware crate as a dependency to the systems crate:

```toml
# crates/systems/Cargo.toml
[dependencies]
hardware = { path = "../hardware" }
```

### Step 2: Create Hardware-Aware System Implementations

Modify existing systems to implement the `SystemDevice` trait:

```rust
// crates/systems/src/gps/gps_system.rs
use hardware::prelude::*;

pub struct GpsSystemDevice {
    base: BaseSystemDevice,
    // GPS-specific fields
    position: Option<Position>,
    satellites: u8,
}

#[async_trait::async_trait]
impl SystemDevice for GpsSystemDevice {
    async fn initialize(&mut self) -> Result<()> {
        self.base.initialize().await?;
        // GPS-specific initialization
        self.satellites = 0;
        Ok(())
    }

    async fn process(&mut self) -> Result<Vec<BusMessage>> {
        // Generate GPS data messages
        let mut messages = Vec::new();

        if let Some(position) = &self.position {
            let payload = serde_json::to_vec(&position)?;
            let message = BusMessage::Data {
                from: self.base.info.address.clone(),
                to: BusAddress::new("navigation_system"), // Example target
                payload,
                message_id: Uuid::new_v4(),
            };
            messages.push(message);
        }

        Ok(messages)
    }

    // Implement other required methods...
}
```

### Step 3: Set Up Hardware Bus

Create a central hardware bus manager:

```rust
// crates/systems/src/hardware_manager.rs
use hardware::prelude::*;
use std::sync::Arc;

pub struct HardwareManager {
    bus: Arc<HardwareBus>,
    device_manager: DeviceManager,
    discovery_protocol: DiscoveryProtocol,
}

impl HardwareManager {
    pub async fn new() -> Result<Self> {
        let bus = Arc::new(HardwareBus::new());
        let device_manager = DeviceManager::new();

        // Create discovery protocol for the manager itself
        let manager_info = DeviceInfo {
            address: BusAddress::new("hardware_manager"),
            config: DeviceConfig {
                name: "Hardware Manager".to_string(),
                capabilities: vec![DeviceCapability::Communication],
                ..Default::default()
            },
            status: DeviceStatus::Online,
            last_seen: SystemTime::now(),
            version: "1.0.0".to_string(),
            manufacturer: "Yachtpit".to_string(),
        };

        let discovery_protocol = DiscoveryProtocol::new(
            manager_info,
            DiscoveryConfig::default(),
        );

        Ok(Self {
            bus,
            device_manager,
            discovery_protocol,
        })
    }

    pub async fn add_system_device(&mut self, device: Box<dyn SystemDevice>) -> Result<()> {
        let address = device.get_info().address.clone();

        // Connect device to bus
        let connection = self.bus.connect_device(address.clone()).await?;

        // Add to device manager
        self.device_manager.add_device(device);

        Ok(())
    }

    pub async fn start_all_systems(&mut self) -> Result<()> {
        self.device_manager.start_all().await?;
        self.discovery_protocol.start().await?;
        Ok(())
    }
}
```

### Step 4: Integrate with Existing Systems

Modify the existing vessel systems to use the hardware abstraction:

```rust
// crates/systems/src/vessel/vessel_systems.rs
use crate::hardware_manager::HardwareManager;

pub async fn create_vessel_systems_with_hardware() -> Result<HardwareManager> {
    let mut hardware_manager = HardwareManager::new().await?;

    // Create GPS system
    let gps_config = DeviceConfig {
        name: "GPS System".to_string(),
        capabilities: vec![DeviceCapability::Gps],
        update_interval_ms: 1000,
        ..Default::default()
    };
    let gps_device = Box::new(GpsSystemDevice::new(gps_config));
    hardware_manager.add_system_device(gps_device).await?;

    // Create Radar system
    let radar_config = DeviceConfig {
        name: "Radar System".to_string(),
        capabilities: vec![DeviceCapability::Radar],
        update_interval_ms: 500,
        ..Default::default()
    };
    let radar_device = Box::new(RadarSystemDevice::new(radar_config));
    hardware_manager.add_system_device(radar_device).await?;

    // Create AIS system
    let ais_config = DeviceConfig {
        name: "AIS System".to_string(),
        capabilities: vec![DeviceCapability::Ais],
        update_interval_ms: 2000,
        ..Default::default()
    };
    let ais_device = Box::new(AisSystemDevice::new(ais_config));
    hardware_manager.add_system_device(ais_device).await?;

    hardware_manager.start_all_systems().await?;

    Ok(hardware_manager)
}
```

### Step 5: Update Main Application

Integrate the hardware manager into the main yachtpit application:

```rust
// crates/yachtpit/src/core/system_manager.rs
use systems::vessel::vessel_systems::create_vessel_systems_with_hardware;

pub struct SystemManager {
    hardware_manager: Option<HardwareManager>,
}

impl SystemManager {
    pub async fn initialize_with_hardware(&mut self) -> Result<()> {
        let hardware_manager = create_vessel_systems_with_hardware().await?;
        self.hardware_manager = Some(hardware_manager);
        Ok(())
    }

    pub async fn discover_devices(&self) -> Result<Vec<DeviceInfo>> {
        if let Some(ref manager) = self.hardware_manager {
            // Use discovery protocol to find devices
            manager.discovery_protocol.discover_devices(None).await?;
            tokio::time::sleep(Duration::from_millis(100)).await; // Wait for responses
            Ok(manager.discovery_protocol.get_known_devices().await)
        } else {
            Ok(vec![])
        }
    }
}
```

## Message Flow Examples

### GPS Data Flow
```
GPS Device → Hardware Bus → Navigation System
           → Discovery Protocol (heartbeat)
           → Other interested devices
```

### Device Discovery Flow
```
New Device → Announce Message → Hardware Bus → All Devices
Discovery Request → Hardware Bus → Matching Devices → Response
```

## Configuration

### Device Configuration
Each device can be configured with:
- Update intervals
- Capabilities
- Custom configuration parameters
- Message queue sizes

### Discovery Configuration
- Heartbeat intervals
- Device timeout periods
- Cleanup intervals
- Maximum tracked devices

## Testing Integration

### Unit Tests
Run tests for individual components:
```bash
cargo test -p hardware
cargo test -p systems
```

### Integration Tests
Create integration tests that verify the complete flow:

```rust
#[tokio::test]
async fn test_complete_hardware_integration() {
    let mut hardware_manager = HardwareManager::new().await.unwrap();

    // Add test devices
    let gps_device = Box::new(create_test_gps_device());
    hardware_manager.add_system_device(gps_device).await.unwrap();

    // Start systems
    hardware_manager.start_all_systems().await.unwrap();

    // Verify device discovery
    let devices = hardware_manager.discovery_protocol.get_known_devices().await;
    assert!(!devices.is_empty());

    // Test message passing
    // ... additional test logic
}
```

## Performance Considerations

1. **Message Throughput**: The hardware bus uses unbounded channels for high throughput
2. **Device Limits**: Configure maximum device limits based on system resources
3. **Update Intervals**: Balance between data freshness and system load
4. **Memory Usage**: Monitor device registry size and message history

## Error Handling

The hardware abstraction layer provides comprehensive error handling:

- **Device Errors**: Automatic retry and fallback mechanisms
- **Bus Errors**: Connection recovery and message queuing
- **Discovery Errors**: Timeout handling and device cleanup

## Migration Strategy

### Phase 1: Parallel Implementation
- Keep existing systems running
- Implement hardware abstraction alongside
- Gradual migration of individual systems

### Phase 2: Feature Parity
- Ensure all existing functionality is available
- Add comprehensive testing
- Performance validation

### Phase 3: Full Migration
- Switch to hardware abstraction as primary
- Remove legacy system implementations
- Optimize performance

## Troubleshooting

### Common Issues

1. **Device Not Found**: Check device registration and bus connection
2. **Message Delivery Failures**: Verify device addresses and bus connectivity
3. **Discovery Timeouts**: Adjust discovery configuration parameters
4. **Performance Issues**: Monitor message queue sizes and update intervals

### Debugging Tools

```rust
// Enable debug logging
use tracing::{info, debug, warn};

// Check device status
// let device_info = hardware_manager.get_device_info(&address).await;
// debug!("Device status: {:?}", device_info.status);

// Monitor message history
// let messages = hardware_bus.get_message_history().await;
// info!("Recent messages: {}", messages.len());
```

## Future Enhancements

1. **Network Discovery**: Extend discovery protocol to work across network boundaries
2. **Device Simulation**: Add comprehensive device simulators for testing
3. **Hot-Plugging**: Support for dynamic device addition/removal
4. **Load Balancing**: Distribute device processing across multiple threads
5. **Persistence**: Save and restore device configurations and state

## Conclusion

The virtual hardware abstraction layer provides a robust foundation for managing yacht systems. By following this integration guide, you can gradually migrate existing systems while maintaining full functionality and adding new capabilities for device discovery and communication.

For questions or issues during integration, refer to the individual module documentation in the hardware crate or create an issue in the project repository.
