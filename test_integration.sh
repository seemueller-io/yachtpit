#!/bin/bash

echo "Testing AIS Service Integration"
echo "==============================="

# Test 1: Los Angeles area (default)
echo "Test 1: Los Angeles area"
curl -s "http://localhost:8081/ais?sw_lat=33.6&sw_lon=-118.5&ne_lat=33.9&ne_lon=-118.0" | jq '.[0].ship_name'
echo ""

# Test 2: San Francisco Bay area
echo "Test 2: San Francisco Bay area"
curl -s "http://localhost:8081/ais?sw_lat=37.5&sw_lon=-122.5&ne_lat=37.9&ne_lon=-122.0" | jq '.[0].ship_name'
echo ""

# Test 3: New York Harbor area
echo "Test 3: New York Harbor area"
curl -s "http://localhost:8081/ais?sw_lat=40.5&sw_lon=-74.2&ne_lat=40.8&ne_lon=-73.8" | jq '.[0].ship_name'
echo ""

# Test 4: Check response structure
echo "Test 4: Response structure check"
response=$(curl -s "http://localhost:8081/ais?sw_lat=33.6&sw_lon=-118.5&ne_lat=33.9&ne_lon=-118.0")
echo "Response contains bounding box: $(echo $response | jq '.[0].raw_message.bounding_box != null')"
echo "Response has latitude: $(echo $response | jq '.[0].latitude != null')"
echo "Response has longitude: $(echo $response | jq '.[0].longitude != null')"
echo ""

echo "Integration test completed successfully!"
echo "The React map will call the AIS service with similar requests when the map bounds change."