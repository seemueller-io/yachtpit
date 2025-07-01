# Icons Required for Yacht yachtpit Application

This document lists all the icons that need to be generated for the yacht yachtpit application based on the current UI implementation.

## Navigation & Compass Icons

### Primary Navigation
- **Compass Rose Icon** - For the central navigation display
- **North Arrow Icon** - Directional indicator
- **GPS Satellite Icon** - GPS status indicator
- **Waypoint Icon** - Navigation waypoints
- **Route Line Icon** - Planned route visualization

## Instrument Gauge Icons

### Speed Gauge
- **Speedometer Icon** - Circular gauge background
- **Speed Needle Icon** - Gauge pointer/needle
- **Knots Unit Icon** - "KTS" stylized icon

### Depth Gauge  
- **Depth Sounder Icon** - Sonar/depth measurement icon
- **Water Depth Icon** - Underwater depth visualization
- **Meter Unit Icon** - "M" stylized icon

## Engine & Systems Icons

### Engine Status
- **Engine Icon** - Marine engine representation
- **Temperature Gauge Icon** - Engine temperature indicator
- **Cooling System Icon** - Engine cooling status
- **Engine Alert Icon** - Warning/alert indicator

### Fuel System
- **Fuel Tank Icon** - Fuel level indicator
- **Fuel Pump Icon** - Fuel system status
- **Fuel Drop Icon** - Fuel consumption indicator

### Electrical System
- **Battery Icon** - Battery level indicator
- **Charging Icon** - Battery charging status
- **Power Icon** - Electrical system status
- **Voltage Meter Icon** - Electrical measurement

## Communication & Navigation Systems

### GPS System
- **GPS Icon** - Global positioning system
- **Satellite Signal Icon** - Signal strength indicator
- **Location Pin Icon** - Current position marker

### Radar System
- **Radar Dish Icon** - Radar antenna representation
- **Radar Sweep Icon** - Radar scanning animation
- **Target Blip Icon** - Radar contact indicator

### AIS (Automatic Identification System)
- **AIS Icon** - Ship identification system
- **Ship Icon** - Other vessel representation
- **Radio Wave Icon** - Communication signal

## Weather & Environmental Icons

### Wind Information
- **Wind Vane Icon** - Wind direction indicator
- **Wind Speed Icon** - Anemometer representation
- **Wind Arrow Icon** - Directional wind indicator
- **Beaufort Scale Icon** - Wind force scale

### Weather Conditions
- **Barometer Icon** - Atmospheric pressure
- **Temperature Icon** - Air temperature
- **Humidity Icon** - Relative humidity indicator

## Status & Alert Icons

### System Status Indicators
- **Green Status Dot** - System operational
- **Red Status Dot** - System fault/offline
- **Yellow Status Dot** - System warning
- **Blue Status Dot** - System standby

### Alert Icons
- **Warning Triangle** - General warning
- **Critical Alert** - Emergency situation
- **Information Icon** - General information
- **Maintenance Icon** - Service required

## UI Control Icons

### Navigation Controls
- **Menu Icon** - Main menu access
- **Settings Icon** - Configuration access
- **Home Icon** - Return to main display
- **Back Arrow** - Navigation back

### Display Controls
- **Brightness Icon** - Screen brightness control
- **Contrast Icon** - Display contrast
- **Night Mode Icon** - Low-light display mode
- **Full Screen Icon** - Display mode toggle

## Chart & Mapping Icons

### Chart Elements
- **Chart Icon** - Nautical chart representation
- **Depth Contour Icon** - Underwater topography
- **Buoy Icon** - Navigation aids
- **Harbor Icon** - Port/marina indicator
- **Anchor Icon** - Anchorage areas

### Measurement Tools
- **Ruler Icon** - Distance measurement
- **Protractor Icon** - Bearing measurement
- **Scale Icon** - Chart scale indicator

## Safety & Emergency Icons

### Safety Equipment
- **Life Ring Icon** - Safety equipment
- **Fire Extinguisher Icon** - Emergency equipment
- **First Aid Icon** - Medical supplies
- **Emergency Radio Icon** - Distress communication

### Emergency Procedures
- **SOS Icon** - Distress signal
- **Mayday Icon** - Emergency call
- **Coast Guard Icon** - Emergency services
- **Evacuation Icon** - Emergency procedures

## File Formats Required

All icons should be generated in the following formats:
- **PNG**: 16x16, 24x24, 32x32, 48x48, 64x64, 128x128, 256x256 pixels
- **SVG**: Scalable vector format for high-DPI displays
- **ICO**: Windows icon format (for desktop application)

## Design Guidelines

### Style Requirements
- **Nautical Theme**: Maritime-inspired design language
- **High Contrast**: Suitable for marine lighting conditions
- **Monochromatic**: Primary colors should be cyan/blue theme
- **Clean Lines**: Minimalist, professional appearance
- **Scalable**: Must remain legible at small sizes

### Color Palette
- **Primary**: Cyan (#00CCFF) - Main UI elements
- **Secondary**: Green (#00FF80) - Operational status
- **Warning**: Orange (#FF8000) - Caution states  
- **Alert**: Red (#FF0040) - Critical alerts
- **Neutral**: Gray (#999999) - Inactive elements

## Implementation Notes

These icons will replace the current text-based placeholders in:
- `src/player.rs` - Main instrument cluster
- `src/menu.rs` - Menu system icons
- `src/loading.rs` - Loading screen elements

The icons should be placed in the `assets/textures/icons/` directory and loaded through the existing `TextureAssets` resource system.