import Map from 'react-map-gl/mapbox';          // ↔ v5+ uses this import path
import 'mapbox-gl/dist/mapbox-gl.css';
import {Box, HStack, Button, Input} from '@chakra-ui/react';
import {useState, useEffect} from 'react';
import Cookies from 'js-cookie';

function App() {
    const [mapboxToken, setMapboxToken] = useState(() => Cookies.get('mapbox_token') || '');

    useEffect(() => {
        if (mapboxToken) {
            Cookies.set('mapbox_token', mapboxToken, {
                secure: true,
                sameSite: 'strict',
                expires: 30  // 30 days
            });
        } else {
            Cookies.remove('mapbox_token');
        }
    }, [mapboxToken]);


    return (
        /* Full-screen wrapper — fills the viewport and becomes the positioning context */
        <Box w="100vw" h="100vh" position="relative" overflow="hidden">
            {/* Button bar — absolutely positioned inside the wrapper */}
            <HStack position="absolute" top={4} right={4} zIndex={1}>
                <Button colorScheme="blue" size="sm" variant="solid">
                    Navigation
                </Button>
                <Button colorScheme="teal" size="sm" variant="solid">
                    Search
                </Button>
                {!mapboxToken && (
                    <Input
                        placeholder="Enter Mapbox token"
                        size="sm"
                        width="300px"
                        value={mapboxToken}
                        onChange={(e) => setMapboxToken(e.target.value)}
                    />
                )}
            </HStack>

            {/* Map itself */}
            <Map
                mapboxAccessToken={mapboxToken}
                initialViewState={{ longitude: -122.4, latitude: 37.8, zoom: 14 }}
                mapStyle="mapbox://styles/mapbox/dark-v11"
                reuseMaps
                attributionControl={false}
                style={{ width: '100%', height: '100%' }}  // let the wrapper dictate size
            />
        </Box>
    );
}

export default App;