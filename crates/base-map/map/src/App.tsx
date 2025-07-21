import 'mapbox-gl/dist/mapbox-gl.css';
import {Box, Button, HStack, Input, Text} from '@chakra-ui/react';
import { useColorMode } from './components/ui/color-mode';
import {useCallback, useEffect, useState} from "react";
import MapNext, {type Geolocation} from "@/MapNext.tsx";
import { getNeumorphicStyle, getNeumorphicColors } from './theme/neumorphic-theme';
import {layers, LayerSelector} from "@/LayerSelector.tsx";

// public key
const key =
    'cGsuZXlKMUlqb2laMlZ2Wm1aelpXVWlMQ0poSWpvaVkycDFOalo0YkdWNk1EUTRjRE41YjJnNFp6VjNNelp6YXlKOS56LUtzS1l0X3VGUGdCSDYwQUFBNFNn';


// const vesselLayerStyle: CircleLayerSpecification = {
//     id: 'vessel',
//     type: 'circle',
//     paint: {
//         'circle-radius': 8,
//         'circle-color': '#ff4444',
//         'circle-stroke-width': 2,
//         'circle-stroke-color': '#ffffff'
//     },
//     source: ''
// };

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


class MyGeolocation implements Geolocation {
    constructor({clearWatch, getCurrentPosition, watchPosition}: {
        clearWatch: (watchId: number) => void;
        getCurrentPosition: (successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions) => void;
        watchPosition: (successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions) => number;
    }) {
        this.clearWatch = clearWatch;
        this.watchPosition = watchPosition;
        this.getCurrentPosition = getCurrentPosition;
    }
    clearWatch(_watchId: number): void {
        throw new Error('Method not implemented.');
    }
    getCurrentPosition(_successCallback: PositionCallback, _errorCallback?: PositionErrorCallback | null, _options?: PositionOptions): void {
        throw new Error('Method not implemented.');
    }
    watchPosition(_successCallback: PositionCallback, _errorCallback?: PositionErrorCallback | null, _options?: PositionOptions): number {
        throw new Error('Method not implemented.');
    }

}


const custom_geolocation = new MyGeolocation({
    clearWatch: (watchId: number) => {
        if (typeof window !== 'undefined' && (window as any).geolocationWatches) {
            const interval = (window as any).geolocationWatches.get(watchId);
            if (interval) {
                clearInterval(interval);
                (window as any).geolocationWatches.delete(watchId);
            }
        }
    },
    watchPosition: (successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions) => {
        if (typeof window === 'undefined') return 0;

        // Initialize watches map if it doesn't exist
        if (!(window as any).geolocationWatches) {
            (window as any).geolocationWatches = new Map();
        }
        if (!(window as any).geolocationWatchId) {
            (window as any).geolocationWatchId = 0;
        }

        const watchId = ++(window as any).geolocationWatchId;

        const pollPosition = async () => {
            if ((window as any).__FLURX__) {
                try {
                    const vesselStatus: VesselStatus = await (window as any).__FLURX__.invoke("get_vessel_status");
                    const position: GeolocationPosition = {
                        coords: {
                            latitude: vesselStatus.latitude,
                            longitude: vesselStatus.longitude,
                            altitude: null,
                            accuracy: 10, // Assume 10m accuracy
                            altitudeAccuracy: null,
                            heading: vesselStatus.heading,
                            speed: vesselStatus.speed,
                            toJSON: () => ({
                                latitude: vesselStatus.latitude,
                                longitude: vesselStatus.longitude,
                                altitude: null,
                                accuracy: 10,
                                altitudeAccuracy: null,
                                heading: vesselStatus.heading,
                                speed: vesselStatus.speed
                            })
                        },
                        timestamp: Date.now(),
                        toJSON: () => ({
                            coords: {
                                latitude: vesselStatus.latitude,
                                longitude: vesselStatus.longitude,
                                altitude: null,
                                accuracy: 10,
                                altitudeAccuracy: null,
                                heading: vesselStatus.heading,
                                speed: vesselStatus.speed
                            },
                            timestamp: Date.now()
                        })
                    };
                    successCallback(position);
                } catch (error) {
                    if (errorCallback) {
                        const positionError: GeolocationPositionError = {
                            code: 2, // POSITION_UNAVAILABLE
                            message: 'Failed to get vessel status: ' + error,
                            PERMISSION_DENIED: 1,
                            POSITION_UNAVAILABLE: 2,
                            TIMEOUT: 3
                        };
                        errorCallback(positionError);
                    }
                }
            }
        };

        // Poll immediately and then at intervals
        pollPosition();
        const interval = setInterval(pollPosition, options?.timeout || 5000);
        (window as any).geolocationWatches.set(watchId, interval);

        return watchId;
    },
    getCurrentPosition: (successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, _options?: PositionOptions) => {
        if (typeof window !== 'undefined' && (window as any).__FLURX__) {
            (async () => {
                try {
                    const vesselStatus: VesselStatus = await (window as any).__FLURX__.invoke("get_vessel_status");
                    const position: GeolocationPosition = {
                        coords: {
                            latitude: vesselStatus.latitude,
                            longitude: vesselStatus.longitude,
                            altitude: null,
                            accuracy: 10, // Assume 10m accuracy
                            altitudeAccuracy: null,
                            heading: vesselStatus.heading,
                            speed: vesselStatus.speed,
                            toJSON: () => ({
                                latitude: vesselStatus.latitude,
                                longitude: vesselStatus.longitude,
                                altitude: null,
                                accuracy: 10,
                                altitudeAccuracy: null,
                                heading: vesselStatus.heading,
                                speed: vesselStatus.speed
                            })
                        },
                        timestamp: Date.now(),
                        toJSON: () => ({
                            coords: {
                                latitude: vesselStatus.latitude,
                                longitude: vesselStatus.longitude,
                                altitude: null,
                                accuracy: 10,
                                altitudeAccuracy: null,
                                heading: vesselStatus.heading,
                                speed: vesselStatus.speed
                            },
                            timestamp: Date.now()
                        })
                    };
                    successCallback(position);
                } catch (error) {
                    if (errorCallback) {
                        const positionError: GeolocationPositionError = {
                            code: 2, // POSITION_UNAVAILABLE
                            message: 'Failed to get vessel status: ' + error,
                            PERMISSION_DENIED: 1,
                            POSITION_UNAVAILABLE: 2,
                            TIMEOUT: 3
                        };
                        errorCallback(positionError);
                    }
                }
            })();
        } else if (errorCallback) {
            const positionError: GeolocationPositionError = {
                code: 2, // POSITION_UNAVAILABLE
                message: '__FLURX__ not available',
                PERMISSION_DENIED: 1,
                POSITION_UNAVAILABLE: 2,
                TIMEOUT: 3
            };
            errorCallback(positionError);
        }
    },
});

// interface MapViewParams {
//     latitude: number;
//     longitude: number;
//     zoom: number;
// }

// interface AuthParams {
//     authenticated: boolean;
//     token: string | null;
// }



function App() {
    const { colorMode } = useColorMode();
    const [isSearchOpen, setIsSearchOpen] = useState(false);
    const [selectedLayer, setSelectedLayer] = useState(layers[0]);
    const [searchInput, setSearchInput] = useState('');
    const [searchResults, setSearchResults] = useState<any[]>([]);
    const [mapView, setMapView] = useState({
        longitude: -122.4,
        latitude: 37.8,
        zoom: 14
    });

    // Map state that can be updated from Rust
    // const [mapView, setMapView] = useState({
    //     longitude: -122.4,
    //     latitude: 37.8,
    //     zoom: 14
    // });

    // Vessel position state
    const [vesselPosition, setVesselPosition] = useState<VesselStatus | null>(null);

    // Create vessel geojson data
    // const vesselGeojson: FeatureCollection = {
    //     type: 'FeatureCollection',
    //     features: vesselPosition ? [
    //         {
    //             type: 'Feature',
    //             geometry: {
    //                 type: 'Point',
    //                 coordinates: [vesselPosition.longitude, vesselPosition.latitude]
    //             },
    //             properties: {
    //                 title: 'Vessel Position',
    //                 heading: vesselPosition.heading,
    //                 speed: vesselPosition.speed
    //             }
    //         }
    //     ] : []
    // };
    

    // Button click handlers
    // const handleNavigationClick = useCallback(async () => {
    //     if (typeof window !== 'undefined' && (window as any).__FLURX__) {
    //         try {
    //             await (window as any).__FLURX__.invoke("navigation_clicked");
    //             console.log('Navigation clicked');
    //         } catch (error) {
    //             console.error('Failed to invoke navigation_clicked:', error);
    //         }
    //     }
    // }, []);


    const selectSearchResult = useCallback(async (searchResult: { lat: string, lon: string }) => {
        // Navigate to the selected location with zoom
        console.log(`Navigating to: ${searchResult.lat}, ${searchResult.lon}`);
        setMapView({
            longitude: parseFloat(searchResult.lon),
            latitude: parseFloat(searchResult.lat),
            zoom: 15
        });
    }, []);

    const handleSearchClick = useCallback(async () => {
        console.log("calling hsc")
        if (isSearchOpen && searchInput.length > 1) {
            try {
                console.log(`Trying to geocode: ${searchInput}`);
                const geocode = await fetch('https://geocode.geoffsee.com', {
                    method: 'POST',
                    mode: 'cors',
                    body: JSON.stringify({
                        location: searchInput,
                    }),
                });
                const coordinates = await geocode.json();
                const { lat, lon } = coordinates;
                console.log(`Got geocode coordinates: ${lat}, ${lon}`);
                setSearchResults([{ lat, lon }]);
            } catch (e) {
                console.error('Geocoding failed:', e);
                // Continue without results
            }
        } else {
            setIsSearchOpen(!isSearchOpen);
        }
        
        if (typeof window !== 'undefined' && (window as any).__FLURX__) {
            try {
                await (window as any).__FLURX__.invoke("search_clicked");
                console.log('Search clicked');
            } catch (error) {
                console.error('Failed to invoke search_clicked:', error);
            }
        }
    }, [isSearchOpen, searchInput]);

    const handleLayerChange = useCallback(async (layer: any) => {
        console.log('Layer change requested:', layer);
        setSelectedLayer(layer);
        console.log('Layer changed to:', layer.name);
    }, []);

    // const handleMapViewChange = useCallback(async (evt: any) => {
    //     const { longitude, latitude, zoom } = evt.viewState;
    //     setMapView({ longitude, latitude, zoom });
    //
    //     if (typeof window !== 'undefined' && (window as any).__FLURX__) {
    //         try {
    //             const mapViewParams: MapViewParams = {
    //                 latitude,
    //                 longitude,
    //                 zoom
    //             };
    //             await (window as any).__FLURX__.invoke("map_view_changed", mapViewParams);
    //             console.log('Map view changed:', mapViewParams);
    //         } catch (error) {
    //             console.error('Failed to invoke map_view_changed:', error);
    //         }
    //     }
    // }, []);

    // Poll for vessel status updates
    useEffect(() => {
        const pollVesselStatus = async () => {
            if (typeof window !== 'undefined' && (window as any).__FLURX__) {
                try {
                    const vesselStatus: VesselStatus = await (window as any).__FLURX__.invoke("get_vessel_status");
                    console.log('Vessel status:', vesselStatus);
                    setVesselPosition(vesselStatus);
                } catch (error) {
                    console.error('Failed to get vessel status:', error);
                }
            }
        };

        // Poll every 5 seconds
        const interval = setInterval(pollVesselStatus, 5000);
        // Also poll immediately
        pollVesselStatus();
        return () => clearInterval(interval);
    }, []);

    // Initialize map with data from Rust
    useEffect(() => {
        const initializeMap = async () => {
            if (typeof window !== 'undefined' && (window as any).__FLURX__) {
                try {
                    const mapInit: GpsPosition = await (window as any).__FLURX__.invoke("get_map_init");
                    console.log('Map initialization data:', mapInit);
                    // setMapView({
                    //     latitude: mapInit.latitude,
                    //     longitude: mapInit.longitude,
                    //     zoom: mapInit.zoom
                    // });
                } catch (error) {
                    console.error('Failed to get map initialization data:', error);
                }
            }
        };

        initializeMap();
    }, []);



    return (
        /* Full-screen wrapper — fills the viewport and becomes the positioning context */
        <Box w="100vw" h="100vh" position="relative" overflow="hidden">
            {/* GPS Feed Display — absolutely positioned at top-right */}
            {vesselPosition && (
                <Box
                    position="absolute"
                    top={65}
                    right={4}
                    zIndex={1}
                    p={4}
                    fontSize="sm"
                    fontFamily="monospace"
                    minW="220px"
                    backdropFilter="blur(10px)"
                    {...getNeumorphicStyle(colorMode as 'light' | 'dark')}
                >
                    <Box fontWeight="bold" mb={3} fontSize="md">GPS Feed</Box>
                    <Box mb={1}>Lat: {vesselPosition.latitude.toFixed(6)}°</Box>
                    <Box mb={1}>Lon: {vesselPosition.longitude.toFixed(6)}°</Box>
                    <Box mb={1}>Heading: {vesselPosition.heading.toFixed(1)}°</Box>
                    <Box>Speed: {vesselPosition.speed.toFixed(1)} kts</Box>
                </Box>
            )}
            {/* Button bar — absolutely positioned inside the wrapper */}
            <HStack position="absolute" top={4} right={4} zIndex={1}>
                <Box
                    display="flex"
                    alignItems="center"
                    position="relative"
                >
                    <Button
                        size="sm"
                        variant="surface"
                        onClick={handleSearchClick}
                        mr={2}
                        {...getNeumorphicStyle(colorMode as 'light' | 'dark')}
                    >
                        <Text>Search...</Text>
                    </Button>
                    {isSearchOpen && <Box
                        w="200px"
                        transform={`translateX(${isSearchOpen ? "0" : "100%"})`}
                        opacity={isSearchOpen ? 1 : 0}
                        onKeyDown={(e) => {
                            console.log(e);
                            if(e.key === 'Escape') {
                                setIsSearchOpen(false)
                            }
                        }}
                        backdropFilter="blur(10px)"
                        {...getNeumorphicStyle(colorMode as 'light' | 'dark', 'pressed')}
                    >
                        <Input
                            placeholder="Search..."
                            size="sm"
                            value={searchInput}
                            onChange={e => setSearchInput(e.target.value)}
                            onKeyPress={async (e) => {
                                console.log(e);
                                if (e.key === 'Enter' && searchResults.length === 0 && searchInput.length > 2) {
                                    await handleSearchClick()
                                }
                            }}
                            border="none"
                            {...getNeumorphicStyle(colorMode as 'light' | 'dark', 'pressed')}
                        />
                        {searchResults.length > 0 && (
                            <Box
                                position="absolute"
                                top="100%"
                                left={0}
                                w="200px"
                                zIndex={2}
                                mt={2}

                                backdropFilter="blur(10px)"
                                {...getNeumorphicStyle(colorMode as 'light' | 'dark')}
                            >
                                {searchResults.map((result, index) => {
                                    const colors = getNeumorphicColors(colorMode as 'light' | 'dark');
                                    return (
                                        <Box
                                            key={index}
                                            p={3}
                                            cursor="pointer"
                                            borderRadius="8px"
                                            transition="all 0.2s ease-in-out"
                                            onKeyPress={async (e) => {
                                                console.log(e.key)
                                                if (e.key === 'Enter' && searchResults.length > 0) {
                                                    console.log(`Selecting result ${result.lat}, ${result.lon}`);
                                                    await selectSearchResult(result);
                                                    setSearchResults([]);
                                                    setIsSearchOpen(false);
                                                }
                                            }}
                                            _hover={{ 
                                                bg: colors.accent + '20',
                                                transform: 'translateY(-1px)',
                                            }}
                                            onClick={async () => {
                                                console.log(`Selecting result ${result.lat}, ${result.lon}`);
                                                await selectSearchResult(result);
                                                setSearchResults([]);
                                                setIsSearchOpen(false);
                                            }}
                                        >
                                            {`${result.lat}, ${result.lon}`}
                                        </Box>
                                    );
                                })}
                            </Box>
                        )}
                    </Box>}
                </Box>
                <LayerSelector onClick={handleLayerChange} />
            </HStack>
            <MapNext mapboxPublicKey={atob(key)} vesselPosition={vesselPosition} layer={selectedLayer} mapView={mapView} geolocation={window.navigator.geolocation || custom_geolocation}/>
            {/*<Map*/}
            {/*    mapboxAccessToken={atob(key)}*/}
            {/*    initialViewState={mapView}*/}
            {/*    onMove={handleMapViewChange}*/}
            {/*    mapStyle="mapbox://styles/mapbox/dark-v11"*/}
            {/*    reuseMaps*/}
            {/*    attributionControl={false}*/}
            {/*    style={{width: '100%', height: '100%'}}  // let the wrapper dictate size*/}
            {/*>*/}
            {/*    /!*{vesselPosition && (*!/*/}
            {/*    /!*    <Source id="vessel-data" type="geojson" data={vesselGeojson}>*!/*/}
            {/*    /!*        <Layer {...vesselLayerStyle} />*!/*/}
            {/*    /!*    </Source>*!/*/}
            {/*    /!*)}*!/*/}
            {/*</Map>*/}
        </Box>
    );
}

export default App;
