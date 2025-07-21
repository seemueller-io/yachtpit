#!/usr/bin/env node

// Test WebSocket bounding box functionality
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:3000/ws');

ws.on('open', function open() {
    console.log('Connected to AIS WebSocket server');
    
    // Send bounding box configuration message
    const boundingBoxMessage = {
        type: 'set_bounding_box',
        bounding_box: {
            sw_lat: 33.7,
            sw_lon: -118.3,
            ne_lat: 33.8,
            ne_lon: -118.2
        }
    };
    
    console.log('Sending bounding box configuration:', boundingBoxMessage);
    ws.send(JSON.stringify(boundingBoxMessage));
});

ws.on('message', function message(data) {
    const message = data.toString();
    
    if (message.startsWith('Connected to AIS stream')) {
        console.log('✓ Received connection confirmation:', message);
    } else {
        try {
            const parsedData = JSON.parse(message);
            
            if (parsedData.type === 'bounding_box_set') {
                console.log('✓ Received bounding box confirmation:', parsedData);
            } else if (parsedData.mmsi || parsedData.ship_name) {
                console.log('✓ Received filtered AIS data:', {
                    mmsi: parsedData.mmsi,
                    ship_name: parsedData.ship_name,
                    latitude: parsedData.latitude,
                    longitude: parsedData.longitude,
                    timestamp: parsedData.timestamp
                });
            } else {
                console.log('✓ Received message:', parsedData);
            }
        } catch (e) {
            console.log('✓ Received text message:', message);
        }
    }
});

ws.on('error', function error(err) {
    console.error('WebSocket error:', err);
});

ws.on('close', function close() {
    console.log('WebSocket connection closed');
});

// Keep the script running for 15 seconds to receive some filtered data
setTimeout(() => {
    console.log('Closing connection after 15 seconds...');
    ws.close();
    process.exit(0);
}, 15000);