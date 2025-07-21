import {useState, useMemo, useEffect, useCallback, useRef} from 'react';
import Map, {
    Marker,
    Popup,
    NavigationControl,
    FullscreenControl,
    ScaleControl,
    GeolocateControl,
    type MapRef
} from 'react-map-gl/mapbox';

import ControlPanel from './control-panel.tsx';
import Pin from './pin.tsx';
import VesselMarker from './vessel-marker.tsx';
import { type VesselData } from './real-ais-provider.tsx';
import { useRealAISProvider } from './real-ais-provider.tsx';

import PORTS from './test_data/nautical-base-data.json';
import {Box} from "@chakra-ui/react";


export interface Geolocation {
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Geolocation/clearWatch) */
    clearWatch(watchId: number): void;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Geolocation/getCurrentPosition) */
    getCurrentPosition(successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions): void;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Geolocation/watchPosition) */
    watchPosition(successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions): number;
}



export default function MapNext(props: any = {mapboxPublicKey: "", geolocation: Geolocation, vesselPosition: undefined, layer: undefined, mapView: undefined} as any) {
    const [popupInfo, setPopupInfo] = useState(null);
    const [vesselPopupInfo, setVesselPopupInfo] = useState<VesselData | null>(null);
    const [boundingBox, setBoundingBox] = useState<{sw_lat: number, sw_lon: number, ne_lat: number, ne_lon: number} | undefined>(undefined);
    const [userLocationLoaded, setUserLocationLoaded] = useState(false);
    const [mapFocused, setMapFocused] = useState(false);
    const mapRef = useRef<MapRef | null>(null);
    
    // Use the real AIS provider with bounding box, user location, and map focus status
    const { vessels } = useRealAISProvider(boundingBox, userLocationLoaded, mapFocused);


    useEffect(() => {
    console.log("vessles", vessels);
    }, [vessels]);

    // Function to update bounding box from map bounds
    const updateBoundingBox = useCallback(() => {
        if (mapRef.current) {
            const map = mapRef.current.getMap();
            const bounds = map.getBounds();
            if (bounds) {
                const sw = bounds.getSouthWest();
                const ne = bounds.getNorthEast();
                
                setBoundingBox({
                    sw_lat: sw.lat,
                    sw_lon: sw.lng,
                    ne_lat: ne.lat,
                    ne_lon: ne.lng
                });
            }
        }
    }, []);

    // Handle map move events
    const handleMapMove = useCallback(() => {
        updateBoundingBox();
    }, [updateBoundingBox]);

    // Initialize bounding box when map loads
    const handleMapLoad = useCallback(() => {
        updateBoundingBox();
    }, [updateBoundingBox]);

    // Handle user location events
    const handleGeolocate = useCallback((position: GeolocationPosition) => {
        console.log('User location loaded:', position);
        setUserLocationLoaded(true);
        setMapFocused(true); // When geolocate succeeds, the map focuses on user location
    }, []);

    const handleTrackUserLocationStart = useCallback(() => {
        console.log('Started tracking user location');
        // User location tracking started, but not necessarily loaded yet
    }, []);

    const handleTrackUserLocationEnd = useCallback(() => {
        console.log('Stopped tracking user location');
        setMapFocused(false);
    }, []);

    const pins = useMemo(
        () =>
            PORTS.map((city, index) => (
                <Marker
                    key={`marker-${index}`}
                    longitude={city.longitude}
                    latitude={city.latitude}
                    anchor="bottom"
                    onClick={e => {
                        // If we let the click event propagates to the map, it will immediately close the popup
                        // with `closeOnClick: true`
                        e.originalEvent.stopPropagation();
                        /*
                        src/MapNext.tsx:34:38 - error TS2345: Argument of type '{ city: string; population: string; image: string; state: string; latitude: number; longitude: number; }' is not assignable to parameter of type 'SetStateAction<null>'.
  Type '{ city: string; population: string; image: string; state: string; latitude: number; longitude: number; }' provides no match for the signature '(prevState: null): null'.
                         */
                        // @ts-ignore
                        setPopupInfo(city);
                    }}
                >
                    <Pin />
                </Marker>
            )),
        []
    );

    const vesselMarkers = useMemo(
        () =>
            vessels.map((vessel) => (
                <Marker
                    key={`vessel-${vessel.id}`}
                    longitude={vessel.longitude}
                    latitude={vessel.latitude}
                    anchor="center"
                    onClick={e => {
                        e.originalEvent.stopPropagation();
                        setVesselPopupInfo(vessel);
                    }}
                >
                    <VesselMarker 
                        heading={vessel.heading}
                        color={vessel.type === 'Yacht' ? '#00cc66' : vessel.type === 'Fishing Vessel' ? '#ff6600' : '#0066cc'}
                        size={14}
                    />
                </Marker>
            )),
        [vessels]
    );


    useEffect(() => {
        console.log("props.vesselPosition", props?.vesselPosition);
        // setLocationLock(props.vesselPosition)
    }, [props.vesselPosition]);

    return (
        <Box>
            <Map
                ref={mapRef}
                initialViewState={{
                    latitude: props.mapView?.latitude || 40,
                    longitude: props.mapView?.longitude || -100,
                    zoom: props.mapView?.zoom || 3.5,
                    bearing: 0,
                    pitch: 0
                }}
                key={`${props.mapView?.latitude}-${props.mapView?.longitude}-${props.mapView?.zoom}`}
                mapStyle={props.layer?.value || "mapbox://styles/mapbox/standard"}
                mapboxAccessToken={props.mapboxPublicKey}
                style={{position: "fixed", width: '100%', height: '100%', bottom: 0, top: 0, left: 0, right: 0}}
                onLoad={handleMapLoad}
                onMoveEnd={handleMapMove}
            >
                <GeolocateControl 
                    showUserHeading={true} 
                    showUserLocation={true} 
                    geolocation={props.geolocation} 
                    position="top-left"
                    onGeolocate={handleGeolocate}
                    onTrackUserLocationStart={handleTrackUserLocationStart}
                    onTrackUserLocationEnd={handleTrackUserLocationEnd}
                />
                <FullscreenControl position="top-left" />
                <NavigationControl position="top-left" />
                <ScaleControl />

                {pins}
                {vesselMarkers}

                {popupInfo && (
                    <Popup
                        anchor="top"
                        /*
                        src/MapNext.tsx:66:53 - error TS2339: Property 'longitude' does not exist on type 'never'.

66                         longitude={Number(popupInfo.longitude)}
                         */
                        // @ts-ignore
                        longitude={Number(popupInfo.longitude)}
                        /*
                        src/MapNext.tsx:67:52 - error TS2339: Property 'latitude' does not exist on type 'never'.

67                         latitude={Number(popupInfo.latitude)}
                                                      ~~~~~~~~
                         */
                        // @ts-ignore
                        latitude={Number(popupInfo.latitude)}
                        onClose={() => setPopupInfo(null)}
                    >
                        <div>
                            {/*src/MapNext.tsx:71:40 - error TS2339: Property 'city' does not exist on type 'never'.

71                             {popupInfo.city}, {popupInfo.state} |{' '}
                                          ~~~~*/}

                            {/*@ts-ignore*/}{/*@ts-ignore*/}
                            {popupInfo.city},{popupInfo.state}
                            {/*@ts-ignore*/}
                            <a
                                target="_new"

                                href={`http://en.wikipedia.org/w/index.php?title=Special:Search&search=${(popupInfo as any).city}, ${(popupInfo as any).state}`}
                            >
                                Wikipedia
                            </a>
                        </div>
                        {/*@ts-ignore*/}
                        <img width="100%" src={popupInfo.image} />
                    </Popup>
                )}

                {vesselPopupInfo && (
                    <Popup
                        anchor="top"
                        longitude={vesselPopupInfo.longitude}
                        latitude={vesselPopupInfo.latitude}
                        onClose={() => setVesselPopupInfo(null)}
                    >
                        <div style={{ minWidth: '200px' }}>
                            <h3 style={{ margin: '0 0 8px 0', fontSize: '16px', fontWeight: 'bold' }}>
                                {vesselPopupInfo.name}
                            </h3>
                            <div style={{ fontSize: '14px', lineHeight: '1.4' }}>
                                <div><strong>Type:</strong> {vesselPopupInfo.type}</div>
                                <div><strong>MMSI:</strong> {vesselPopupInfo.mmsi}</div>
                                <div><strong>Call Sign:</strong> {vesselPopupInfo.callSign}</div>
                                <div><strong>Speed:</strong> {vesselPopupInfo.speed.toFixed(1)} knots</div>
                                <div><strong>Heading:</strong> {vesselPopupInfo.heading.toFixed(0)}°</div>
                                <div><strong>Length:</strong> {vesselPopupInfo.length.toFixed(0)}m</div>
                                {vesselPopupInfo.destination && (
                                    <div><strong>Destination:</strong> {vesselPopupInfo.destination}</div>
                                )}
                                {vesselPopupInfo.eta && (
                                    <div><strong>ETA:</strong> {vesselPopupInfo.eta}</div>
                                )}
                                <div style={{ fontSize: '12px', color: '#666', marginTop: '8px' }}>
                                    Last Update: {vesselPopupInfo.lastUpdate.toLocaleTimeString()}
                                </div>
                            </div>
                        </div>
                    </Popup>
                )}



            </Map>

            <ControlPanel />
        </Box>
    );
}