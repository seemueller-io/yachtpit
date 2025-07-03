use bevy::prelude::*;
use super::speed_gauge::SpeedGauge;
use super::depth_gauge::DepthGauge;
use super::compass_gauge::CompassGauge;

/// Yacht data resource containing all sensor readings
#[derive(Resource)]
pub struct YachtData {
    pub speed: f32,           // knots
    pub depth: f32,           // meters
    pub heading: f32,         // degrees
    pub engine_temp: f32,     // celsius
    pub fuel_level: f32,      // percentage
    pub battery_level: f32,   // percentage
    pub wind_speed: f32,      // knots
    pub wind_direction: f32,  // degrees
}

impl Default for YachtData {
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

/// Updates yacht data with simulated sensor readings
pub fn update_yacht_data(mut yacht_data: ResMut<YachtData>, time: Res<Time>) {
    let t = time.elapsed_secs();

    // Simulate realistic yacht data with some variation
    yacht_data.speed = 12.5 + (t * 0.3).sin() * 2.0;
    yacht_data.depth = 15.2 + (t * 0.1).sin() * 3.0;
    yacht_data.heading = (yacht_data.heading + time.delta_secs() * 5.0) % 360.0;
    yacht_data.engine_temp = 82.0 + (t * 0.2).sin() * 3.0;
    yacht_data.wind_speed = 8.3 + (t * 0.4).sin() * 1.5;
    yacht_data.wind_direction = (yacht_data.wind_direction + time.delta_secs() * 10.0) % 360.0;

    // Slowly drain fuel and battery (very slowly for demo purposes)
    yacht_data.fuel_level = (yacht_data.fuel_level - time.delta_secs() * 0.01).max(0.0);
    yacht_data.battery_level = (yacht_data.battery_level - time.delta_secs() * 0.005).max(0.0);
}

/// Updates the display values for all instrument gauges
pub fn update_instrument_displays(
    yacht_data: Res<YachtData>,
    mut speed_query: Query<&mut Text, (With<SpeedGauge>, Without<DepthGauge>, Without<CompassGauge>)>,
    mut depth_query: Query<&mut Text, (With<DepthGauge>, Without<SpeedGauge>, Without<CompassGauge>)>,
    mut compass_query: Query<&mut Text, (With<CompassGauge>, Without<SpeedGauge>, Without<DepthGauge>)>,
) {
    // Update speed display
    for mut text in speed_query.iter_mut() {
        if text.0.contains('.') {
            text.0 = format!("{:.1}", yacht_data.speed);
        }
    }

    // Update depth display
    for mut text in depth_query.iter_mut() {
        if text.0.contains('.') {
            text.0 = format!("{:.1}", yacht_data.depth);
        }
    }

    // Update compass display
    for mut text in compass_query.iter_mut() {
            text.0 = format!("{:03.0}", yacht_data.heading);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yacht_data_default() {
        let yacht_data = YachtData::default();
        assert_eq!(yacht_data.speed, 12.5);
        assert_eq!(yacht_data.depth, 15.2);
        assert_eq!(yacht_data.heading, 45.0);
        assert_eq!(yacht_data.fuel_level, 75.0);
        assert_eq!(yacht_data.battery_level, 88.0);
    }
}