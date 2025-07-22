import 'mapbox-gl/dist/mapbox-gl.css';
import {Box, Button, HStack, Text} from '@chakra-ui/react';
import {useColorMode} from './components/ui/color-mode';
import {useCallback, useEffect, useState} from "react";
import MapNext from "@/MapNext.tsx";
import {getNeumorphicColors, getNeumorphicStyle} from './theme/neumorphic-theme';
import {layers, LayerSelector} from "@/LayerSelector.tsx";
import {useAISProvider, type VesselData} from './ais-provider';
import type {GpsPosition, VesselStatus} from './types';
import {GpsFeed} from "@/components/map/GpsFeedInfo.tsx";
import {AisFeed} from './components/map/AisFeedInfo';
import {Search} from "@/components/map/Search.tsx";
import {SearchResult} from "@/components/map/SearchResult.tsx";
import {NativeGeolocation} from "@/CustomGeolocate.ts";

// public key
const key =
    'cGsuZXlKMUlqb2laMlZ2Wm1aelpXVWlMQ0poSWpvaVkycDFOalo0YkdWNk1EUTRjRE41YjJnNFp6VjNNelp6YXlKOS56LUtzS1l0X3VGUGdCSDYwQUFBNFNn';

function App() {
    const {colorMode} = useColorMode();
    const [isSearchOpen, setIsSearchOpen] = useState(false);
    const [selectedLayer, setSelectedLayer] = useState(layers[0]);
    const [searchInput, setSearchInput] = useState('');
    const [searchResults, setSearchResults] = useState<any[]>([]);
    const [mapView, setMapView] = useState({
        longitude: -122.4,
        latitude: 37.8,
        zoom: 14
    });

    const custom_geolocation = new NativeGeolocation({
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

    // Vessel position state
    const [vesselPosition, setVesselPosition] = useState<VesselStatus | null>(null);

    // AIS state management
    const [aisEnabled, setAisEnabled] = useState(false);
    const [boundingBox, _setBoundingBox] = useState<{
        sw_lat: number;
        sw_lon: number;
        ne_lat: number;
        ne_lon: number;
    } | undefined>(undefined);
    const [vesselPopup, setVesselPopup] = useState<VesselData | null>(null);

    // Use the AIS provider when enabled
    const {
        vessels,
        isConnected: aisConnected,
        error: aisError,
        connectionStatus
    } = useAISProvider(aisEnabled ? boundingBox : undefined);


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
                const {lat, lon} = coordinates;
                console.log(`Got geocode coordinates: ${lat}, ${lon}`);
                setSearchResults([{lat, lon}]);
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
            <Box
                position="absolute"
                top={65}
                right={4}
                maxW="20%"
                zIndex={1}
                p={4}
            >
            {vesselPosition && (
                <GpsFeed vesselPosition={vesselPosition} colorMode={colorMode}/>
            )}

            {/* AIS Status Panel */}
            {aisEnabled && (
                <AisFeed
                    vesselPosition={vesselPosition}
                    colorMode={colorMode}
                    connectionStatus={connectionStatus}
                    vesselData={vessels}
                    aisError={aisError}
                    aisConnected={aisConnected}
                />
            )}
            </Box>
            {/* Button bar — absolutely positioned inside the wrapper */}
            <HStack position="absolute" top={4} right={4} zIndex={1}>
                <Search
                    onClick={handleSearchClick}
                    colorMode={colorMode}
                    searchOpen={isSearchOpen}
                    onKeyDown={(e) => {
                        console.log(e);
                        if (e.key === 'Escape') {
                            setIsSearchOpen(false)
                        }
                    }}
                    value={searchInput}
                    onChange={e => setSearchInput(e.target.value)}
                    onKeyPress={async (e) => {
                        console.log(e);
                        if (e.key === 'Enter' && searchResults.length === 0 && searchInput.length > 2) {
                            await handleSearchClick()
                        }
                    }}
                    searchResults={searchResults}
                    callbackfn={(result, index) => {
                        const colors = getNeumorphicColors(colorMode as 'light' | 'dark');
                        return (
                            <SearchResult key={index} onKeyPress={async (e) => {
                                if (e.key === 'Enter' && searchResults.length > 0) {
                                    console.log(`Selecting result ${result.lat}, ${result.lon}`);
                                    await selectSearchResult(result);
                                    setSearchResults([]);
                                    setIsSearchOpen(false);
                                }
                            }}
                              colors={colors}
                              onClick={async () => {
                                  console.log(`Selecting result ${result.lat}, ${result.lon}`);
                                  await selectSearchResult(result);
                                  setSearchResults([]);
                                  setIsSearchOpen(false);
                              }}
                              result={result}
                            />
                        );
                    }}/>
                <Button
                    size="sm"
                    variant="surface"
                    onClick={() => setAisEnabled(!aisEnabled)}
                    mr={2}
                    {...getNeumorphicStyle(colorMode as 'light' | 'dark')}
                    bg={aisEnabled ? 'green.500' : undefined}
                    _hover={{
                        bg: aisEnabled ? 'green.600' : undefined,
                    }}
                >
                    <Text>AIS {aisEnabled ? 'ON' : 'OFF'}</Text>
                </Button>
                <LayerSelector onClick={handleLayerChange}/>
            </HStack>
            <MapNext
                mapboxPublicKey={atob(key)}
                vesselPosition={vesselPosition}
                layer={selectedLayer}
                mapView={mapView}
                geolocation={custom_geolocation}
                aisVessels={aisEnabled ? vessels : []}
                onVesselClick={setVesselPopup}
                vesselPopup={vesselPopup}
                onVesselPopupClose={() => setVesselPopup(null)}
            />
        </Box>
    );
}

export default App;
