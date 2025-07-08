import Map from 'react-map-gl/mapbox'; // ↔ v5+ uses this import path
import 'mapbox-gl/dist/mapbox-gl.css';
import {Box, Button, HStack} from '@chakra-ui/react';

// public key
const key =
    'cGsuZXlKMUlqb2laMlZ2Wm1aelpXVWlMQ0poSWpvaVkycDFOalo0YkdWNk1EUTRjRE41YjJnNFp6VjNNelp6YXlKOS56LUtzS1l0X3VGUGdCSDYwQUFBNFNn';


function App() {
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
            </HStack>
            <Map
                mapboxAccessToken={atob(key)}
                initialViewState={{longitude: -122.4, latitude: 37.8, zoom: 14}}
                mapStyle="mapbox://styles/mapbox/dark-v11"
                reuseMaps
                attributionControl={false}
                style={{width: '100%', height: '100%'}}  // let the wrapper dictate size
            />
        </Box>
    );
}

export default App;