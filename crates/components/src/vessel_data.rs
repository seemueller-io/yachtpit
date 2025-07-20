use bevy::prelude::*;
use super::speed_gauge::SpeedGauge;
use super::depth_gauge::DepthGauge;
use super::compass_gauge::CompassGauge;

/// Yacht data resource containing all sensor readings
#[derive(Resource)]
pub struct VesselData {
    pub speed: f32,           // knots
    pub depth: f32,           // meters
    pub heading: f32,         // degrees
    pub engine_temp: f32,     // celsius
    pub fuel_level: f32,      // percentage
    pub battery_level: f32,   // percentage
    pub wind_speed: f32,      // knots
    pub wind_direction: f32,  // degrees
}

impl Default for VesselData {
    fn default() -> Self {
        Self {
            speed: 12.5,
            depth: 15.2,
            heading: 045.0,
            engine_temp: 82.0,
            fuel_level: 75.0,
            battery_level: 88.0,
            wind_speed: 8.3,
            wind_direction: 120.0,
        }
    }
}

/// Updates yacht data with sensor readings, using real GPS data when available
pub fn update_vessel_data(mut vessel_data: ResMut<VesselData>, time: Res<Time>) {
    update_vessel_data_with_gps(vessel_data, time, None);
}

/// Updates yacht data with sensor readings, optionally using real GPS data
pub fn update_vessel_data_with_gps(
    mut vessel_data: ResMut<VesselData>, 
    time: Res<Time>, 
    gps_data: Option<(f64, f64)> // (speed, heading)
) {
    let t = time.elapsed_secs();

    // Use real GPS data if available, otherwise simulate
    if let Some((gps_speed, gps_heading)) = gps_data {
        vessel_data.speed = gps_speed as f32;
        vessel_data.heading = gps_heading as f32;
    } else {
        // Simulate realistic yacht data with some variation
        vessel_data.speed = 12.5 + (t * 0.3).sin() * 2.0;
        vessel_data.heading = (vessel_data.heading + time.delta_secs() * 5.0) % 360.0;
    }

    // Continue simulating other sensor data
    vessel_data.depth = 15.2 + (t * 0.1).sin() * 3.0;
    vessel_data.engine_temp = 82.0 + (t * 0.2).sin() * 3.0;
    vessel_data.wind_speed = 8.3 + (t * 0.4).sin() * 1.5;
    vessel_data.wind_direction = (vessel_data.wind_direction + time.delta_secs() * 10.0) % 360.0;

    // Slowly drain fuel and battery (very slowly for demo purposes)
    vessel_data.fuel_level = (vessel_data.fuel_level - time.delta_secs() * 0.01).max(0.0);
    vessel_data.battery_level = (vessel_data.battery_level - time.delta_secs() * 0.005).max(0.0);
}

/// Updates the display values for all instrument gauges
pub fn update_instrument_displays(
    vessel_data: Res<VesselData>,
    mut speed_query: Query<&mut Text, (With<SpeedGauge>, Without<DepthGauge>, Without<CompassGauge>)>,
    mut depth_query: Query<&mut Text, (With<DepthGauge>, Without<SpeedGauge>, Without<CompassGauge>)>,
    mut compass_query: Query<&mut Text, (With<CompassGauge>, Without<SpeedGauge>, Without<DepthGauge>)>,
) {
    // Update speed display
    for mut text in speed_query.iter_mut() {
        if text.0.contains('.') {
            text.0 = format!("{:.1}", vessel_data.speed);
        }
    }

    // Update depth display
    for mut text in depth_query.iter_mut() {
        if text.0.contains('.') {
            text.0 = format!("{:.1}", vessel_data.depth);
        }
    }

    // Update compass display
    for mut text in compass_query.iter_mut() {
            text.0 = format!("{:03.0}", vessel_data.heading);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vessel_data_default() {
        let vessel_data = VesselData::default();
        assert_eq!(vessel_data.speed, 12.5);
        assert_eq!(vessel_data.depth, 15.2);
        assert_eq!(vessel_data.heading, 45.0);
        assert_eq!(vessel_data.fuel_level, 75.0);
        assert_eq!(vessel_data.battery_level, 88.0);
    }
}