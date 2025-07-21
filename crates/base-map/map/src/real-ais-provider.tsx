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

// Convert AIS service response to VesselData format
const convertAisResponseToVesselData = (aisResponse: AisResponse): any | null => {
    // Skip responses that don't have essential vessel data

    console.log({aisResponse})
    // return aisResponse.raw_message;
    return {
        id: aisResponse.mmsi,
        name: aisResponse.ship_name || `Vessel ${aisResponse.mmsi}`,
        type: aisResponse.ship_type || 'Unknown',
        latitude: aisResponse.latitude,
        longitude: aisResponse.longitude,
        heading: aisResponse.heading || 0,
        speed: aisResponse.speed_over_ground || 0,
        length: 100, // Default length, could be extracted from raw_message if available
        width: 20,   // Default width
        mmsi: aisResponse.mmsi,
        callSign: '', // Could be extracted from raw_message if available
        destination: '', // Could be extracted from raw_message if available
        eta: '', // Could be extracted from raw_message if available
        lastUpdate: new Date()
    };
};

// WebSocket message types for communication with the backend
interface WebSocketMessage {
    type: string;
    bounding_box?: BoundingBox;
}

// Hook to provide real AIS data from the service via WebSocket
export const useRealAISProvider = (boundingBox?: BoundingBox, userLocationLoaded?: boolean, mapFocused?: boolean) => {
    const [vessels, setVessels] = useState<VesselData[]>([]);
    const [isActive, setIsActive] = useState(true);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isConnected, setIsConnected] = useState(false);
    const [aisStreamStarted, setAisStreamStarted] = useState(false);
    
    const wsRef = useRef<WebSocket | null>(null);
    const lastBoundingBoxRef = useRef<BoundingBox | undefined>(undefined);
    const reconnectTimeoutRef = useRef<any | null>(null);
    const vesselMapRef = useRef<Map<string, VesselData>>(new Map());

    // Connect to WebSocket
    const connectWebSocket = useCallback(() => {
        // Prevent multiple connections
        if (!isActive) return;
        
        // Check if we already have an active or connecting WebSocket
        if (wsRef.current && 
            (wsRef.current.readyState === WebSocket.OPEN || 
             wsRef.current.readyState === WebSocket.CONNECTING)) {
            console.log('WebSocket already connected or connecting, skipping...');
            return;
        }

        // Close any existing connection before creating a new one
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }

        setIsLoading(true);
        setError(null);

        try {
            console.log('Creating new WebSocket connection...');
            const ws = new WebSocket('ws://localhost:3000/ws');
            wsRef.current = ws;

            ws.onopen = () => {
                console.log('Connected to AIS WebSocket');
                setIsConnected(true);
                setIsLoading(false);
                setError(null);

                // Send bounding box configuration if available
                // Note: We'll send the bounding box separately to avoid connection recreation
                const currentBoundingBox = lastBoundingBoxRef.current;
                if (currentBoundingBox) {
                    const message: WebSocketMessage = {
                        type: 'set_bounding_box',
                        bounding_box: currentBoundingBox
                    };
                    ws.send(JSON.stringify(message));
                    console.log('Sent initial bounding box configuration:', currentBoundingBox);
                }
            };

            ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    
                    // Handle connection confirmation and bounding box confirmations
                    if (typeof data === 'string' || data.type) {
                        console.log('Received WebSocket message:', data);
                        return;
                    }
                    const vesselData = convertAisResponseToVesselData(data);
                    if (vesselData) {
                        // Update vessel map for efficient updates
                        vesselMapRef.current.set(vesselData.mmsi, vesselData);

                        // Update vessels state with current map values
                        setVessels(Array.from(vesselMapRef.current.values()));
                    }
                } catch (err) {
                    console.error('Error parsing WebSocket message:', err);
                }
            };

            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                setError('WebSocket connection error');
                setIsConnected(false);
            };

            ws.onclose = (event) => {
                console.log('WebSocket connection closed:', event.code, event.reason);
                setIsConnected(false);
                setIsLoading(false);
                
                // Attempt to reconnect if the connection was active
                if (isActive && !event.wasClean) {
                    setError('Connection lost, attempting to reconnect...');
                    reconnectTimeoutRef.current = setTimeout(() => {
                        connectWebSocket();
                    }, 3000); // Reconnect after 3 seconds
                }
            };

        } catch (err) {
            console.error('Error creating WebSocket connection:', err);
            setError(err instanceof Error ? err.message : 'Unknown WebSocket error');
            setIsLoading(false);
        }
    }, [isActive]); // Removed boundingBox dependency to prevent reconnections

    // Send bounding box update to WebSocket
    const updateBoundingBox = useCallback((bbox: BoundingBox) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            const message: WebSocketMessage = {
                type: 'set_bounding_box',
                bounding_box: bbox
            };
            wsRef.current.send(JSON.stringify(message));
            console.log('Updated bounding box:', bbox);
        }
    }, []);

    // Send start AIS stream message to WebSocket
    const startAisStream = useCallback(() => {
        if (wsRef.current?.readyState === WebSocket.OPEN && !aisStreamStarted) {
            const message: WebSocketMessage = {
                type: 'start_ais_stream'
            };
            wsRef.current.send(JSON.stringify(message));
            console.log('Sent start AIS stream request');
            setAisStreamStarted(true);
        }
    }, [aisStreamStarted]);

    // Connect to WebSocket when component mounts or becomes active
    useEffect(() => {
        if (isActive) {
            connectWebSocket();
        }

        return () => {
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
            if (wsRef.current) {
                wsRef.current.close();
                wsRef.current = null;
            }
        };
    }, [isActive, connectWebSocket]);

    // Handle bounding box changes
    useEffect(() => {
        if (!boundingBox || !isActive) return;

        // Check if bounding box actually changed to avoid unnecessary updates
        const lastBbox = lastBoundingBoxRef.current;
        if (lastBbox && 
            lastBbox.sw_lat === boundingBox.sw_lat &&
            lastBbox.sw_lon === boundingBox.sw_lon &&
            lastBbox.ne_lat === boundingBox.ne_lat &&
            lastBbox.ne_lon === boundingBox.ne_lon) {
            return;
        }

        lastBoundingBoxRef.current = boundingBox;
        
        // Clear existing vessels when bounding box changes
        vesselMapRef.current.clear();
        setVessels([]);
        
        // Send new bounding box to WebSocket
        updateBoundingBox(boundingBox);
    }, [boundingBox, updateBoundingBox, isActive]);

    // Handle active state changes
    useEffect(() => {
        if (!isActive) {
            // Close WebSocket connection when inactive
            if (wsRef.current) {
                wsRef.current.close();
                wsRef.current = null;
            }
            setIsConnected(false);
            setError(null);
            
            // Clear reconnection timeout
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
                reconnectTimeoutRef.current = null;
            }
        }
    }, [isActive]);

    // Start AIS stream when user location is loaded and map is focused
    useEffect(() => {
        if (userLocationLoaded && mapFocused && isConnected && !aisStreamStarted) {
            console.log('User location loaded and map focused, starting AIS stream...');
            startAisStream();
        }
    }, [userLocationLoaded, mapFocused, isConnected, aisStreamStarted, startAisStream]);

    return {
        vessels,
        isActive,
        setIsActive,
        isLoading,
        error,
        isConnected,
        refreshVessels: () => {
            // For WebSocket, we can trigger a reconnection to refresh data
            if (wsRef.current) {
                wsRef.current.close();
            }
            connectWebSocket();
        }
    };
};