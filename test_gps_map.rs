fn main() {
    // Test coordinate conversion functions
    let lat = 43.6377;
    let lon = -1.4497;
    let zoom = 10;

    let (tile_x, tile_y) = lat_lon_to_tile(lat, lon, zoom);
    let (converted_lat, converted_lon) = tile_to_lat_lon(tile_x, tile_y, zoom);

    println!("Original: lat={}, lon={}", lat, lon);
    println!("Tile coordinates: x={}, y={}", tile_x, tile_y);
    println!("Converted back: lat={}, lon={}", converted_lat, converted_lon);
    println!("Difference: lat={}, lon={}", (lat - converted_lat).abs(), (lon - converted_lon).abs());

    // Check that the conversion is approximately correct (tiles are discrete, so some precision loss is expected)
    assert!((lat - converted_lat).abs() < 0.5);
    assert!((lon - converted_lon).abs() < 0.5);

    println!("✓ Coordinate conversion test passed!");
}

/// Convert latitude/longitude to tile coordinates
fn lat_lon_to_tile(lat: f64, lon: f64, zoom: u8) -> (i32, i32) {
    let n = 2.0_f64.powi(zoom as i32);
    let lat_rad = lat.to_radians();

    let x = ((lon + 180.0) / 360.0 * n) as i32;
    let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n) as i32;

    (x, y)
}

/// Convert tile coordinates to latitude/longitude
fn tile_to_lat_lon(x: i32, y: i32, zoom: u8) -> (f64, f64) {
    let n = 2.0_f64.powi(zoom as i32);

    let lon = x as f64 / n * 360.0 - 180.0;
    let lat_rad = (std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n)).sinh().atan();
    let lat = lat_rad.to_degrees();

    (lat, lon)
}
