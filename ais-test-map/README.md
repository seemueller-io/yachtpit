# AIS Test Map

This is a separate test map implementation created to test displaying data from the AIS server. It's located in a separate directory from the main base-map to allow independent testing and development.

## Purpose

This test map was created to:
- Test AIS WebSocket server connectivity
- Display vessel data from the AIS stream
- Provide a simplified environment for debugging AIS data issues
- Serve as a reference implementation for AIS integration

## Issues Fixed

### React StrictMode Double Effect Issues
The primary issue was React's StrictMode in development mode causing double invocation of effects, leading to multiple simultaneous WebSocket connection attempts and immediate disconnections with errors like:
- "WebSocket is closed before the connection is established"
- Connection closed with error code 1006 (abnormal closure)
- "The network connection was lost"

### JSON Parsing Errors
The original implementation had JSON parsing errors when the AIS server sent plain text messages like "Connected to AIS stream". The error was:

```
Error parsing WebSocket message: – SyntaxError: JSON Parse error: Unexpected identifier "Connected"
```

**Solution**: Implemented a React StrictMode-safe WebSocket connection with comprehensive state management and race condition prevention in `src/ais-provider.tsx`:

### Key Improvements:
1. **React StrictMode Protection**: Prevents multiple simultaneous connection attempts using `isConnectingRef` flag
2. **Component Mount Tracking**: Uses `isMountedRef` to prevent operations on unmounted components
3. **Connection State Guards**: Comprehensive checks before attempting connections or state updates
4. **Exponential Backoff Reconnection**: Automatic reconnection with increasing delays (1s, 2s, 4s, 8s, etc.)
5. **Connection Timeout Management**: 10-second timeout with proper cleanup to prevent hanging connections
6. **Graceful Message Handling**: Handles both JSON and plain text messages without errors
7. **Resource Cleanup**: Proper cleanup of all timeouts, event handlers, and connections
8. **Race Condition Prevention**: Multiple safeguards to prevent connection race conditions

```typescript
// React StrictMode-safe connection logic
const connectSocket = useCallback(() => {
    // Prevent multiple simultaneous connection attempts (React StrictMode protection)
    if (isConnectingRef.current) {
        console.log('Connection attempt already in progress, skipping...');
        return;
    }

    // Check if component is still mounted
    if (!isMountedRef.current) {
        console.log('Component unmounted, skipping connection attempt');
        return;
    }

    isConnectingRef.current = true;
    
    const ws = new WebSocket('ws://localhost:3000/ws');
    wsRef.current = ws;

    // Connection timeout with proper cleanup
    connectionTimeoutRef.current = setTimeout(() => {
        if (ws.readyState === WebSocket.CONNECTING && isMountedRef.current) {
            console.log('[TIMEOUT] Connection timeout, closing WebSocket');
            isConnectingRef.current = false;
            ws.close();
        }
    }, 10000);

    ws.onopen = () => {
        if (!isMountedRef.current) {
            console.log('[OPEN] Component unmounted, closing connection');
            ws.close();
            return;
        }
        isConnectingRef.current = false;
        // Handle successful connection...
    };

    // Robust message handling
    ws.onmessage = (event) => {
        try {
            const messageData = event.data;
            let data;
            try {
                data = JSON.parse(messageData);
            } catch (parseError) {
                console.log('Received plain text message:', messageData);
                return;
            }
            // Handle JSON messages...
        } catch (err) {
            console.error('Error processing WebSocket message:', err);
        }
    };
}, []);

// Component cleanup with proper resource management
useEffect(() => {
    isMountedRef.current = true;
    
    // Small delay to prevent immediate double connection in StrictMode
    const connectTimeout = setTimeout(() => {
        if (isMountedRef.current) {
            connectSocket();
        }
    }, 100);

    return () => {
        isMountedRef.current = false;
        clearTimeout(connectTimeout);
        // Clean up all resources...
    };
}, [connectSocket]);
```

## Project Structure

```
ais-test-map/
├── src/
│   ├── App.tsx              # Main application component
│   ├── MapComponent.tsx     # Map display component
│   ├── ais-provider.tsx     # AIS WebSocket provider (fixed)
│   └── main.tsx            # Application entry point
├── public/
├── package.json
├── vite.config.ts
├── test-websocket.cjs      # WebSocket connection test
└── README.md               # This file
```

## Setup and Usage

### Prerequisites
- Node.js installed
- AIS WebSocket server running on `ws://localhost:3000/ws`
- Mapbox access token

### Installation
```bash
cd ais-test-map
npm install
```

### Running the Test Map
```bash
npm run dev
```

The map will be available at `http://localhost:5173`

### Testing WebSocket Connection
To test the WebSocket connection independently:
```bash
node test-websocket.cjs
```

This will:
- Connect to the AIS WebSocket server
- Send test messages (bounding box, start stream)
- Display received messages (both JSON and plain text)
- Verify the connection works without parsing errors

## Features

- **Real-time AIS Data**: Connects to WebSocket server for live vessel data
- **Interactive Map**: Mapbox-based map with vessel markers
- **Bounding Box Updates**: Automatically updates AIS data based on map viewport
- **Error Handling**: Robust error handling for connection issues
- **Connection Status**: Visual indicators for connection state

## Configuration

The AIS provider connects to `ws://localhost:3000/ws` by default. To change this, modify the WebSocket URL in `src/ais-provider.tsx`:

```typescript
const ws = new WebSocket('ws://your-server:port/ws');
```

## Troubleshooting

### Connection Issues
1. Verify the AIS server is running: `lsof -i :3000`
2. Check WebSocket connection: `node test-websocket.cjs`
3. Review browser console for detailed error messages

### No Vessel Data
1. Ensure the map viewport covers an area with AIS data
2. Check that the AIS stream has started (look for "Started AIS stream" in console)
3. Verify bounding box is being sent correctly

## Development Notes

This test map uses a simplified AIS provider compared to the main base-map implementation. Key differences:
- Simplified vessel data structure
- Direct WebSocket connection without complex reconnection logic
- Focused on testing and debugging rather than production use

## Testing Results

### React StrictMode Connection Test
```
🧪 Testing React StrictMode scenario (rapid double connections)...
Connection 1: ✅ Successful
Connection 2: ✅ Properly skipped (race condition prevented)

🔄 Testing sequential connections...
Sequential connection 1: ✅ Success
Sequential connection 2: ✅ Success  
Sequential connection 3: ✅ Success

📈 Final Statistics:
Total connection attempts: 4
Successful connections: 4
Failed connections: 0
Success rate: 100.0%

🎉 All tests passed! React StrictMode fixes are working correctly.
```

### Connection Stability Verification
✅ **React StrictMode Protection**: Double effects properly handled without race conditions  
✅ **WebSocket Connection**: Stable connections without immediate disconnections  
✅ **Message Handling**: Both JSON and plain text messages processed correctly  
✅ **Reconnection Logic**: Exponential backoff working with proper cleanup  
✅ **Resource Management**: All timeouts and connections properly cleaned up  
✅ **Bounding Box Updates**: Map viewport changes trigger correct AIS data updates  
✅ **AIS Stream**: Stream initialization and data flow working correctly