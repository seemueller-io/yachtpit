import Map from 'react-map-gl/mapbox';          // ↔ v5+ uses this import path
import 'mapbox-gl/dist/mapbox-gl.css';
import {Box, HStack, Button, Input} from '@chakra-ui/react';
import {useState, useEffect, useCallback} from 'react';
import Cookies from 'js-cookie';

// Types for bevy_flurx_ipc communication
interface GpsPosition {
    latitude: number;
    longitude: number;
    zoom: number;
}

interface VesselStatus {
    latitude: number;
    longitude: number;
    heading: number;
    speed: number;
}

interface MapViewParams {
    latitude: number;
    longitude: number;
    zoom: number;
}

interface AuthParams {
    authenticated: boolean;
    token: string | null;
}

function App() {
    const [mapboxToken, setMapboxToken] = useState(() => Cookies.get('mapbox_token') || '');
    const [isTokenLoading, setIsTokenLoading] = useState(false);
    const [authenticated, setAuthenticated] = useState(false);

    // Map state that can be updated from Rust
    const [mapView, setMapView] = useState({
        longitude: -122.4,
        latitude: 37.8,
        zoom: 14
    });

    // Initialize map with data from Rust
    useEffect(() => {
        const initializeMap = async () => {
            if (typeof window !== 'undefined' && (window as any).__FLURX__) {
                try {
                    const mapInit: GpsPosition = await (window as any).__FLURX__.invoke("get_map_init");
                    console.log('Map initialization data:', mapInit);
                    setMapView({
                        latitude: mapInit.latitude,
                        longitude: mapInit.longitude,
                        zoom: mapInit.zoom
                    });
                } catch (error) {
                    console.error('Failed to get map initialization data:', error);
                }
            }
        };

        initializeMap();
    }, []);

    // Poll for vessel status updates
    useEffect(() => {
        const pollVesselStatus = async () => {
            if (typeof window !== 'undefined' && (window as any).__FLURX__) {
                try {
                    const vesselStatus: VesselStatus = await (window as any).__FLURX__.invoke("get_vessel_status");
                    console.log('Vessel status:', vesselStatus);
                    // You can update vessel position on map here if needed
                } catch (error) {
                    console.error('Failed to get vessel status:', error);
                }
            }
        };

        // Poll every 5 seconds
        const interval = setInterval(pollVesselStatus, 5000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        const checkAndLoadToken = async () => {
            const savedToken = Cookies.get('mapbox_token');
            if (savedToken) {
                setMapboxToken(savedToken);
            }
            setIsTokenLoading(false);
        };
        checkAndLoadToken();
    }, []);

    useEffect(() => {
        if (!isTokenLoading && mapboxToken) {
            Cookies.set('mapbox_token', mapboxToken, {
                secure: true,
                sameSite: 'strict',
                expires: 30  // 30 days
            });
        }
    }, [mapboxToken, isTokenLoading]);

    // Handle authentication status changes
    useEffect(() => {
        const updateAuthStatus = async () => {
            if (typeof window !== 'undefined' && (window as any).__FLURX__) {
                try {
                    const authParams: AuthParams = {
                        authenticated,
                        token: authenticated ? mapboxToken : null
                    };
                    await (window as any).__FLURX__.invoke("auth_status_changed", authParams);
                    console.log('Auth status updated:', authParams);
                } catch (error) {
                    console.error('Failed to update auth status:', error);
                }
            }
        };

        updateAuthStatus();
    }, [authenticated, mapboxToken]);

    // Button click handlers
    const handleNavigationClick = useCallback(async () => {
        if (typeof window !== 'undefined' && (window as any).__FLURX__) {
            try {
                await (window as any).__FLURX__.invoke("navigation_clicked");
                console.log('Navigation clicked');
            } catch (error) {
                console.error('Failed to invoke navigation_clicked:', error);
            }
        }
    }, []);

    const handleSearchClick = useCallback(async () => {
        if (typeof window !== 'undefined' && (window as any).__FLURX__) {
            try {
                await (window as any).__FLURX__.invoke("search_clicked");
                console.log('Search clicked');
            } catch (error) {
                console.error('Failed to invoke search_clicked:', error);
            }
        }
    }, []);

    const handleMapViewChange = useCallback(async (evt: any) => {
        const { longitude, latitude, zoom } = evt.viewState;
        setMapView({ longitude, latitude, zoom });

        if (typeof window !== 'undefined' && (window as any).__FLURX__) {
            try {
                const mapViewParams: MapViewParams = {
                    latitude,
                    longitude,
                    zoom
                };
                await (window as any).__FLURX__.invoke("map_view_changed", mapViewParams);
                console.log('Map view changed:', mapViewParams);
            } catch (error) {
                console.error('Failed to invoke map_view_changed:', error);
            }
        }
    }, []);


    return (
        /* Full-screen wrapper — fills the viewport and becomes the positioning context */
        <Box w="100vw" h="100vh" position="relative" overflow="hidden">
            {/* Button bar — absolutely positioned inside the wrapper */}
            <HStack position="absolute" top={4} right={4} zIndex={1}>
                <Button 
                    colorScheme="blue" 
                    size="sm" 
                    variant="solid"
                    onClick={handleNavigationClick}
                >
                    Navigation
                </Button>
                <Button 
                    colorScheme="teal" 
                    size="sm" 
                    variant="solid"
                    onClick={handleSearchClick}
                >
                    Search
                </Button>
                {!authenticated && (
                    <>
                        <Input
                            placeholder="Enter Mapbox token"
                            size="sm"
                            width="300px"
                            value={mapboxToken}
                            onChange={(e) => setMapboxToken(e.target.value)}
                        />
                        <Button
                            colorScheme="green"
                            size="sm"
                            onClick={() => setAuthenticated(true)}
                        >
                            Set Mapbox token
                        </Button>
                    </>
                )}
            </HStack>

            {/* Map itself */}
            {authenticated && (<Map
                mapboxAccessToken={mapboxToken}
                initialViewState={mapView}
                onMove={handleMapViewChange}
                mapStyle="mapbox://styles/mapbox/dark-v11"
                reuseMaps
                attributionControl={false}
                style={{width: '100%', height: '100%'}}  // let the wrapper dictate size
            />)}
        </Box>
    );
}

export default App;
