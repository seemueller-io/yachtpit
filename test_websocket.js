#!/usr/bin/env node

// Simple WebSocket test client for AIS data
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:3000/ws');

ws.on('open', function open() {
    console.log('Connected to AIS WebSocket server');
    
    // Send a test message
    ws.send('Hello from test client');
});

ws.on('message', function message(data) {
    const message = data.toString();
    
    if (message.startsWith('Connected to AIS stream')) {
        console.log('✓ Received connection confirmation:', message);
    } else if (message.startsWith('Echo:')) {
        console.log('✓ Received echo response:', message);
    } else {
        try {
            const aisData = JSON.parse(message);
            console.log('✓ Received AIS data:', {
                mmsi: aisData.mmsi,
                ship_name: aisData.ship_name,
                latitude: aisData.latitude,
                longitude: aisData.longitude,
                timestamp: aisData.timestamp
            });
        } catch (e) {
            console.log('✓ Received message:', message);
        }
    }
});

ws.on('error', function error(err) {
    console.error('WebSocket error:', err);
});

ws.on('close', function close() {
    console.log('WebSocket connection closed');
});

// Keep the script running for 30 seconds to receive some data
setTimeout(() => {
    console.log('Closing connection after 30 seconds...');
    ws.close();
    process.exit(0);
}, 30000);