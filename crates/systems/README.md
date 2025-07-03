# YachtPit Systems Crate

The `systems` crate provides the core game systems, vessel logic, and maritime system integrations for the YachtPit virtual yacht cockpit application.

## Overview

This crate contains all the business logic and system implementations built on top of Bevy's ECS (Entity-Component-System) architecture. It bridges the visual components with real maritime system behaviors, data processing, and vessel simulation logic.

## System Modules

### World Management
- **`world::player`** - Player entity management and vessel ownership
  - `PlayerPlugin` - Bevy plugin for player system integration
  - `get_vessel_systems()` - Retrieves available vessel systems
  - `setup_instrument_cluster_system()` - Initializes instrument displays

### Vessel Systems
- **`vessel::vessel_systems`** - Core vessel system implementations
  - `VesselSystem` - Base trait for all vessel systems
  - `SystemStatus` - System operational status tracking
  - `SystemInteraction` - User interaction handling
  - `create_vessel_systems()` - Factory for vessel system creation

### Maritime Systems
- **`AisSystem`** - Automatic Identification System integration
  - Vessel tracking and identification
  - Collision avoidance data
  - Maritime traffic monitoring

- **`GpsSystem`** - Global Positioning System implementation
  - Position tracking and navigation
  - Waypoint management
  - Course and speed calculations

- **`RadarSystem`** - Radar detection and display system
  - Object detection and tracking
  - Range and bearing calculations
  - Weather and obstacle detection

## Key Features

### System Integration
- **Real-time Data Processing** - Continuous updates of vessel and environmental data
- **System Interconnectivity** - Maritime systems work together for comprehensive navigation
- **Status Monitoring** - Track operational status of all vessel systems
- **User Interaction** - Handle user inputs and system controls

### Vessel Simulation
- **Realistic Behavior** - Systems behave like real maritime equipment
- **Data Validation** - Ensure data integrity and realistic ranges
- **System Dependencies** - Model real-world system interdependencies
- **Performance Optimization** - Efficient processing for real-time operation

### Plugin Architecture
- **Modular Design** - Each system can be enabled/disabled independently
- **Bevy Integration** - Full integration with Bevy's plugin system
- **Event-Driven** - Responsive to system events and state changes
- **Extensible** - Easy to add new maritime systems

## Usage

This crate is designed for internal use within the YachtPit project and is not published to crates.io. It provides the systems layer that connects components with application logic.

```rust
use systems::{
    PlayerPlugin, VesselSystem, SystemStatus,
    AisSystem, GpsSystem, RadarSystem,
    create_vessel_systems, get_vessel_systems
};
```

## Dependencies

- **Bevy 0.16** - Core ECS engine and plugin system
- **`components`** - UI components and instrument widgets
- **Rust Standard Library** - Core functionality and data structures

## Architecture

The systems crate follows these design principles:

### ECS Integration
- **Systems** - Process vessel data and update component states
- **Components** - Store vessel and system state data
- **Resources** - Manage shared data like vessel configuration
- **Events** - Handle system interactions and state changes

### Data Flow
1. **Input Processing** - Handle user interactions and external data
2. **System Updates** - Process maritime system logic and calculations  
3. **Component Updates** - Update visual components with new data
4. **Event Dispatch** - Notify other systems of state changes

### Performance Considerations
- **Efficient Queries** - Optimized ECS queries for real-time performance
- **Batch Processing** - Group similar operations for better performance
- **Memory Management** - Careful resource management for smooth operation
- **Platform Optimization** - Optimized for desktop, web, and mobile platforms
