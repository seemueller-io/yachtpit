// import Map from 'react-map-gl/mapbox';
// import {Source, Layer} from 'react-map-gl/maplibre';
import 'mapbox-gl/dist/mapbox-gl.css';
import {Box, Button, HStack, Input} from '@chakra-ui/react';
import {useCallback, useEffect, useState} from "react";
import MapNext from "@/MapNext.tsx";
// import type {FeatureCollection} from 'geojson';
// import type {CircleLayerSpecification} from "mapbox-gl";

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

    const [isSearchOpen, setIsSearchOpen] = useState(false);

    // Map state that can be updated from Rust
    // const [mapView, setMapView] = useState({
    //     longitude: -122.4,
    //     latitude: 37.8,
    //     zoom: 14
    // });

    // Vessel position state
    // const [vesselPosition, setVesselPosition] = useState<VesselStatus | null>(null);

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
        setIsSearchOpen(!isSearchOpen);
        if (typeof window !== 'undefined' && (window as any).__FLURX__) {
            try {
                await (window as any).__FLURX__.invoke("search_clicked");
                console.log('Search clicked');
            } catch (error) {
                console.error('Failed to invoke search_clicked:', error);
            }
        }
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
                    // setVesselPosition(vesselStatus);
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
            {/* Button bar — absolutely positioned inside the wrapper */}
            <HStack position="absolute" top={4} right={4} zIndex={1}>
                <Box
                    display="flex"
                    alignItems="center"
                >
                    <Button
                        colorScheme="teal"
                        size="sm"
                        variant="solid"
                        onClick={handleSearchClick}
                        mr={2}
                    >
                        Search
                    </Button>
                    {isSearchOpen && <Box
                        w="200px"
                        transition="all 0.3s"
                        transform={`translateX(${isSearchOpen ? "0" : "100%"})`}
                        opacity={isSearchOpen ? 1 : 0}
                        color="white"
                    >
                        <Input
                            placeholder="Search..."
                            size="sm"
                            _placeholder={{
                                color: "#d1cfcf"
                            }}
                        />
                    </Box>}
                </Box>
                <Button
                    colorScheme="blue"
                    size="sm"
                    variant="solid"
                    onClick={handleNavigationClick}
                >
                    Layer
                </Button>
            </HStack>
            <MapNext mapboxPublicKey={atob(key)}/>
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
