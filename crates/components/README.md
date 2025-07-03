# YachtPit Components Crate

The `components` crate provides reusable UI components and maritime instrument widgets for the YachtPit virtual yacht cockpit application.

## Overview

This crate contains all the visual components and rendering primitives built on top of Bevy's UI system. It provides a comprehensive set of maritime instruments and displays that simulate real yacht cockpit equipment.

## Available Components

### Maritime Instruments
- **`SpeedGauge`** - Displays vessel speed in knots with analog gauge visualization
- **`DepthGauge`** - Shows water depth measurements with sonar-style display
- **`CompassGauge`** - Magnetic compass with heading display and cardinal directions
- **`WindDisplay`** - Wind speed and direction indicator with graphical representation

### Engine & Systems
- **`EngineStatus`** - Engine monitoring display showing RPM, temperature, and status
- **`SystemDisplay`** - General system status and monitoring interface

### Navigation Equipment
- **`NavigationDisplay`** - Chart plotter and navigation information display
- **`GpsIndicator`** - GPS status and position information
- **`RadarIndicator`** - Radar system status and basic display
- **`AisIndicator`** - AIS (Automatic Identification System) status and nearby vessels

### Core Components
- **`InstrumentCluster`** - Container for organizing multiple instruments
- **`VesselData`** - Data structures for vessel state and measurements

### UI Framework
- **`ui`** - Base UI utilities and common interface elements
- **`theme`** - Consistent theming and styling for maritime aesthetics
- **`composition`** - Layout and composition utilities for instrument arrangements

## Features

- **Realistic Maritime Aesthetics** - Components designed to mimic real yacht instruments
- **Bevy Integration** - Built on Bevy's ECS architecture for performance
- **Modular Design** - Each component can be used independently or combined
- **Responsive Layout** - Adapts to different screen sizes and orientations
- **Theme Consistency** - Unified visual design across all components

## Usage

This crate is designed for internal use within the YachtPit project and is not published to crates.io. Components are imported and used by the `systems` and main `yachtpit` crates.

```rust
use components::{
    SpeedGauge, DepthGauge, CompassGauge, 
    InstrumentCluster, VesselData
};
```

## Dependencies

- **Bevy 0.16** - Core engine and UI framework
- Built for cross-platform compatibility (Desktop, Web, Mobile)

## Architecture

Components follow Bevy's ECS (Entity-Component-System) pattern and are designed to be:
- **Composable** - Can be combined in different arrangements
- **Data-driven** - Respond to changes in vessel data automatically  
- **Performance-oriented** - Optimized for real-time updates
- **Platform-agnostic** - Work across all supported platforms
