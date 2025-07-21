import 'mapbox-gl/dist/mapbox-gl.css';
import React, { useState, useCallback, useRef, useMemo } from 'react';
import Map, { Marker, Popup, NavigationControl, ScaleControl, type MapRef } from 'react-map-gl/mapbox';
import { useAISProvider, type VesselData } from './ais-provider';
import VesselMarker from './vessel-marker';

// Mapbox token (base64 encoded)
const key = 'cGsuZXlKMUlqb2laMlZ2Wm1aelpXVWlMQ0poSWpvaVkycDFOalo0YkdWNk1EUTRjRE41YjJnNFp6VjNNelp6YXlKOS56LUtzS1l0X3VGUGdCSDYwQUFBNFNn';
const MAPBOX_TOKEN = atob(key);

const App: React.FC = () => {
    const [boundingBox, setBoundingBox] = useState<{
        sw_lat: number;
        sw_lon: number;
        ne_lat: number;
        ne_lon: number;
    } | undefined>(undefined);
    
    const [vesselPopup, setVesselPopup] = useState<VesselData | null>(null);
    const mapRef = useRef<MapRef | null>(null);
    
    // Use the AIS provider
    const { vessels, isConnected, error, connectionStatus, updateBoundingBox } = useAISProvider(boundingBox);

    // Update bounding box when map moves
    const handleMapMove = useCallback(() => {
        if (mapRef.current) {
            const map = mapRef.current.getMap();
            const bounds = map.getBounds();
            if (bounds) {
                const sw = bounds.getSouthWest();
                const ne = bounds.getNorthEast();
                
                const newBoundingBox = {
                    sw_lat: sw.lat,
                    sw_lon: sw.lng,
                    ne_lat: ne.lat,
                    ne_lon: ne.lng
                };
                
                setBoundingBox(newBoundingBox);
                updateBoundingBox(newBoundingBox);
            }
        }
    }, [updateBoundingBox]);

    // Initialize bounding box when map loads
    const handleMapLoad = useCallback(() => {
        handleMapMove();
    }, [handleMapMove]);

    // Create vessel markers
    const vesselMarkers = useMemo(() => 
        vessels.map((vessel) => (
            <Marker
                key={`vessel-${vessel.id}`}
                longitude={vessel.longitude}
                latitude={vessel.latitude}
                anchor="center"
                onClick={(e) => {
                    e.originalEvent.stopPropagation();
                    setVesselPopup(vessel);
                }}
            >
                <VesselMarker 
                    heading={vessel.heading}
                    color={getVesselColor(vessel.type)}
                    size={16}
                />
            </Marker>
        )),
        [vessels]
    );

    return (
        <div style={{ width: '100vw', height: '100vh', position: 'relative' }}>
            {/* Status Panel */}
            <div style={{
                position: 'absolute',
                top: 10,
                left: 10,
                zIndex: 1000,
                background: 'rgba(255, 255, 255, 0.9)',
                padding: '10px',
                borderRadius: '5px',
                boxShadow: '0 2px 10px rgba(0,0,0,0.1)',
                minWidth: '200px'
            }}>
                <h3 style={{ margin: '0 0 10px 0', fontSize: '16px' }}>AIS Test Map</h3>
                <div><strong>Status:</strong> {connectionStatus}</div>
                <div><strong>Vessels:</strong> {vessels.length}</div>
                {error && <div style={{ color: 'red' }}><strong>Error:</strong> {error}</div>}
                {isConnected && (
                    <div style={{ color: 'green', fontSize: '12px', marginTop: '5px' }}>
                        ✓ Connected to AIS server
                    </div>
                )}
            </div>

            {/* Map */}
            <Map
                ref={mapRef}
                initialViewState={{
                    latitude: 40.7128,
                    longitude: -74.0060,
                    zoom: 10
                }}
                style={{ width: '100%', height: '100%' }}
                mapStyle="mapbox://styles/mapbox/standard"
                mapboxAccessToken={MAPBOX_TOKEN}
                onLoad={handleMapLoad}
                onMoveEnd={handleMapMove}
            >
                <NavigationControl position="top-right" />
                <ScaleControl />
                
                {vesselMarkers}

                {/* Vessel Popup */}
                {vesselPopup && (
                    <Popup
                        longitude={vesselPopup.longitude}
                        latitude={vesselPopup.latitude}
                        anchor="bottom"
                        onClose={() => setVesselPopup(null)}
                        closeButton={true}
                        closeOnClick={false}
                    >
                        <div style={{ padding: '10px', minWidth: '200px' }}>
                            <h4 style={{ margin: '0 0 10px 0' }}>{vesselPopup.name}</h4>
                            <div><strong>MMSI:</strong> {vesselPopup.mmsi}</div>
                            <div><strong>Type:</strong> {vesselPopup.type}</div>
                            <div><strong>Speed:</strong> {vesselPopup.speed.toFixed(1)} knots</div>
                            <div><strong>Heading:</strong> {vesselPopup.heading}°</div>
                            <div><strong>Position:</strong> {vesselPopup.latitude.toFixed(4)}, {vesselPopup.longitude.toFixed(4)}</div>
                            <div style={{ fontSize: '12px', color: '#666', marginTop: '5px' }}>
                                Last update: {vesselPopup.lastUpdate.toLocaleTimeString()}
                            </div>
                        </div>
                    </Popup>
                )}
            </Map>
        </div>
    );
};

// Helper function to get vessel color based on type
const getVesselColor = (type: string): string => {
    switch (type.toLowerCase()) {
        case 'yacht':
        case 'pleasure craft':
            return '#00cc66';
        case 'fishing vessel':
        case 'fishing':
            return '#ff6600';
        case 'cargo':
        case 'container':
            return '#cc0066';
        case 'tanker':
            return '#ff0000';
        case 'passenger':
            return '#6600cc';
        default:
            return '#0066cc';
    }
};

export default App;