import { useState, useEffect, useCallback, useRef } from 'react';

// Vessel data interface
export interface VesselData {
    id: string;
    name: string;
    type: string;
    latitude: number;
    longitude: number;
    heading: number; // degrees 0-359
    speed: number; // knots
    length: number; // meters
    width: number; // meters
    mmsi: string; // Maritime Mobile Service Identity
    callSign: string;
    destination?: string;
    eta?: string;
    lastUpdate: Date;
}

// AIS service response structure (matching Rust AisResponse)
interface AisResponse {
    message_type?: string;
    mmsi?: string;
    ship_name?: string;
    latitude?: number;
    longitude?: number;
    timestamp?: string;
    speed_over_ground?: number;
    course_over_ground?: number;
    heading?: number;
    navigation_status?: string;
    ship_type?: string;
    raw_message: any;
}

// Bounding box for AIS queries
interface BoundingBox {
    sw_lat: number;
    sw_lon: number;
    ne_lat: number;
    ne_lon: number;
}

// WebSocket message types for communication with the backend
interface WebSocketMessage {
    type: string;
    bounding_box?: BoundingBox;
}

// Convert AIS service response to VesselData format
const convertAisResponseToVesselData = (aisResponse: AisResponse): VesselData | null => {
    if ((!aisResponse.raw_message?.MetaData?.MMSI) || !aisResponse.latitude || !aisResponse.longitude) {
        console.log('Skipping vessel with missing data:', {
            mmsi: aisResponse.mmsi,
            metadataMSSI: aisResponse.raw_message?.MetaData?.MSSI,
            latitude: aisResponse.latitude,
            longitude: aisResponse.longitude,
            raw: aisResponse.raw_message
        });
        return null;
    }

    return {
        id: aisResponse.mmsi ?? !aisResponse.raw_message?.MetaData?.MSSI,
        name: aisResponse.ship_name || `Vessel ${aisResponse.mmsi}`,
        type: aisResponse.ship_type || 'Unknown',
        latitude: aisResponse.latitude,
        longitude: aisResponse.longitude,
        heading: aisResponse.heading || 0,
        speed: aisResponse.speed_over_ground || 0,
        length: 100, // Default length
        width: 20,   // Default width
        mmsi: aisResponse.mmsi,
        callSign: '', 
        destination: '', 
        eta: '', 
        lastUpdate: new Date()
    };
};

// Simplified AIS provider hook for testing
export const useAISProvider = (boundingBox?: BoundingBox) => {
    const [vessels, setVessels] = useState<VesselData[]>([]);
    const [isConnected, setIsConnected] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [connectionStatus, setConnectionStatus] = useState<string>('Disconnected');
    
    const wsRef = useRef<WebSocket | null>(null);
    const vesselMapRef = useRef<Map<string, VesselData>>(new Map());
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const reconnectAttemptsRef = useRef<number>(0);
    const connectionTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const isConnectingRef = useRef<boolean>(false);
    const isMountedRef = useRef<boolean>(true);
    const maxReconnectAttempts = 10;
    const baseReconnectDelay = 1000; // 1 second

    // Calculate exponential backoff delay
    const getReconnectDelay = useCallback(() => {
        const delay = baseReconnectDelay * Math.pow(2, reconnectAttemptsRef.current);
        return Math.min(delay, 30000); // Cap at 30 seconds
    }, []);

    // Connect to WebSocket with React StrictMode-safe logic
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

        // Clear any existing reconnection timeout
        if (reconnectTimeoutRef.current) {
            clearTimeout(reconnectTimeoutRef.current);
            reconnectTimeoutRef.current = null;
        }

        // Clear any existing connection timeout
        if (connectionTimeoutRef.current) {
            clearTimeout(connectionTimeoutRef.current);
            connectionTimeoutRef.current = null;
        }

        // Check if already connected or connecting
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            console.log('WebSocket already connected');
            return;
        }

        if (wsRef.current?.readyState === WebSocket.CONNECTING) {
            console.log('WebSocket already connecting');
            return;
        }

        // Check reconnection attempts
        if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            setError('Failed to connect after multiple attempts');
            setConnectionStatus('Failed');
            return;
        }

        // Set connecting flag to prevent race conditions
        isConnectingRef.current = true;

        setConnectionStatus(reconnectAttemptsRef.current > 0 ? 
            `Reconnecting... (${reconnectAttemptsRef.current + 1}/${maxReconnectAttempts})` : 
            'Connecting...');
        setError(null);

        try {
            console.log(`[CONNECT] Attempting WebSocket connection (attempt ${reconnectAttemptsRef.current + 1})`);
            
            // Close any existing connection properly
            if (wsRef.current) {
                wsRef.current.onopen = null;
                wsRef.current.onmessage = null;
                wsRef.current.onerror = null;
                wsRef.current.onclose = null;
                wsRef.current.close();
                wsRef.current = null;
            }

            const ws = new WebSocket('ws://localhost:3000/ws');
            wsRef.current = ws;

            // Set connection timeout with proper cleanup
            connectionTimeoutRef.current = setTimeout(() => {
                if (ws.readyState === WebSocket.CONNECTING && isMountedRef.current) {
                    console.log('[TIMEOUT] Connection timeout, closing WebSocket');
                    isConnectingRef.current = false;
                    ws.close();
                }
            }, 10000); // 10 second timeout

            ws.onopen = () => {
                // Clear connection timeout
                if (connectionTimeoutRef.current) {
                    clearTimeout(connectionTimeoutRef.current);
                    connectionTimeoutRef.current = null;
                }
                
                // Check if component is still mounted
                if (!isMountedRef.current) {
                    console.log('[OPEN] Component unmounted, closing connection');
                    ws.close();
                    return;
                }

                console.log('[OPEN] Connected to AIS WebSocket');
                isConnectingRef.current = false; // Clear connecting flag
                setIsConnected(true);
                setConnectionStatus('Connected');
                setError(null);
                reconnectAttemptsRef.current = 0; // Reset reconnection attempts

                // Send bounding box if available
                if (boundingBox && isMountedRef.current) {
                    const message: WebSocketMessage = {
                        type: 'set_bounding_box',
                        bounding_box: boundingBox
                    };
                    ws.send(JSON.stringify(message));
                    console.log('[OPEN] Sent bounding box:', boundingBox);
                }

                // Start AIS stream
                if (isMountedRef.current) {
                    const startMessage: WebSocketMessage = {
                        type: 'start_ais_stream'
                    };
                    ws.send(JSON.stringify(startMessage));
                    console.log('[OPEN] Started AIS stream');
                }
            };

            ws.onmessage = (event) => {
                try {
                    const messageData = event.data;
                    
                    // Try to parse as JSON, but handle plain text messages gracefully
                    let data;
                    try {
                        data = JSON.parse(messageData);
                    } catch (parseError) {
                        console.log('Received plain text message:', messageData);
                        return;
                    }
                    
                    // Handle JSON status messages
                    if (typeof data === 'string' || data.type) {
                        console.log('Received message:', data);
                        return;
                    }

                    // Process vessel data
                    const vesselData = convertAisResponseToVesselData(data);
                    if (vesselData) {
                        console.log('Received vessel data:', vesselData);
                        vesselMapRef.current.set(vesselData.mmsi, vesselData);
                        setVessels(Array.from(vesselMapRef.current.values()));
                    }
                } catch (err) {
                    console.error('Error processing WebSocket message:', err);
                }
            };

            ws.onerror = (error) => {
                // Clear connection timeout
                if (connectionTimeoutRef.current) {
                    clearTimeout(connectionTimeoutRef.current);
                    connectionTimeoutRef.current = null;
                }
                
                console.error('[ERROR] WebSocket error:', error);
                isConnectingRef.current = false; // Clear connecting flag
                
                // Only update state if component is still mounted
                if (isMountedRef.current) {
                    setError('WebSocket connection error');
                    setIsConnected(false);
                }
            };

            ws.onclose = (event) => {
                // Clear connection timeout
                if (connectionTimeoutRef.current) {
                    clearTimeout(connectionTimeoutRef.current);
                    connectionTimeoutRef.current = null;
                }
                
                console.log(`[CLOSE] WebSocket connection closed: ${event.code} - ${event.reason}`);
                isConnectingRef.current = false; // Clear connecting flag
                
                // Only update state if component is still mounted
                if (isMountedRef.current) {
                    setIsConnected(false);
                }
                
                // Only attempt reconnection if component is mounted, wasn't a clean close, and we haven't exceeded max attempts
                if (isMountedRef.current && !event.wasClean && reconnectAttemptsRef.current < maxReconnectAttempts) {
                    reconnectAttemptsRef.current++;
                    const delay = getReconnectDelay();
                    
                    console.log(`[CLOSE] Scheduling reconnection in ${delay}ms (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})`);
                    setError(`Connection lost, reconnecting in ${Math.round(delay/1000)}s...`);
                    setConnectionStatus('Reconnecting...');
                    
                    reconnectTimeoutRef.current = setTimeout(() => {
                        if (isMountedRef.current) {
                            connectSocket();
                        }
                    }, delay);
                } else {
                    if (isMountedRef.current) {
                        if (event.wasClean) {
                            setConnectionStatus('Disconnected');
                            setError(null);
                        } else {
                            setConnectionStatus('Failed');
                            setError('Connection failed after multiple attempts');
                        }
                    }
                }
            };

        } catch (err) {
            console.error('Error creating WebSocket connection:', err);
            setError(err instanceof Error ? err.message : 'Unknown WebSocket error');
            setConnectionStatus('Error');
            
            // Schedule reconnection attempt
            if (reconnectAttemptsRef.current < maxReconnectAttempts) {
                reconnectAttemptsRef.current++;
                const delay = getReconnectDelay();
                reconnectTimeoutRef.current = setTimeout(() => {
                    connectSocket();
                }, delay);
            }
        }
    }, [boundingBox, getReconnectDelay]);

    // Update bounding box
    const updateBoundingBox = useCallback((bbox: BoundingBox) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            const message: WebSocketMessage = {
                type: 'set_bounding_box',
                bounding_box: bbox
            };
            wsRef.current.send(JSON.stringify(message));
            console.log('Updated bounding box:', bbox);
            
            // Clear existing vessels when bounding box changes
            vesselMapRef.current.clear();
            setVessels([]);
        }
    }, []);

    // Connect on mount with React StrictMode protection
    useEffect(() => {
        // Set mounted flag
        isMountedRef.current = true;
        
        // Small delay to prevent immediate double connection in StrictMode
        const connectTimeout = setTimeout(() => {
            if (isMountedRef.current) {
                connectSocket();
            }
        }, 100);

        return () => {
            // Mark component as unmounted
            isMountedRef.current = false;
            
            // Clear connect timeout
            clearTimeout(connectTimeout);
            
            // Clear reconnection timeout
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
                reconnectTimeoutRef.current = null;
            }
            
            // Clear connection timeout
            if (connectionTimeoutRef.current) {
                clearTimeout(connectionTimeoutRef.current);
                connectionTimeoutRef.current = null;
            }
            
            // Reset connection flags
            isConnectingRef.current = false;
            
            // Close WebSocket connection properly
            if (wsRef.current) {
                console.log('[CLEANUP] Closing WebSocket connection');
                wsRef.current.onopen = null;
                wsRef.current.onmessage = null;
                wsRef.current.onerror = null;
                wsRef.current.onclose = null;
                wsRef.current.close();
                wsRef.current = null;
            }
            
            // Reset reconnection attempts
            reconnectAttemptsRef.current = 0;
        };
    }, [connectSocket]);

    return {
        vessels,
        isConnected,
        error,
        connectionStatus,
        connectSocket,
        updateBoundingBox
    };
};