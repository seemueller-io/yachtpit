import {useState, useMemo, useCallback, useRef} from 'react';
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
import VesselMarker from './vessel-marker';
import type { VesselData } from './ais-provider';

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



interface MapNextProps {
    mapboxPublicKey: string;
    geolocation: Geolocation;
    vesselPosition?: any;
    layer?: any;
    mapView?: any;
    aisVessels?: VesselData[];
    onVesselClick?: (vessel: VesselData) => void;
    vesselPopup?: VesselData | null;
    onVesselPopupClose?: () => void;
}

export default function MapNext(props: MapNextProps) {
    const [popupInfo, setPopupInfo] = useState(null);
    const mapRef = useRef<MapRef | null>(null);

    // Handle user location events
    const handleGeolocate = useCallback((position: GeolocationPosition) => {
        console.log('User location loaded:', position);
    }, []);

    const handleTrackUserLocationStart = useCallback(() => {
        console.log('Started tracking user location');
    }, []);

    const handleTrackUserLocationEnd = useCallback(() => {
        console.log('Stopped tracking user location');
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

    // Create vessel markers
    const vesselMarkers = useMemo(() => 
        (props.aisVessels || []).map((vessel) => (
            <Marker
                key={`vessel-${vessel.id}`}
                longitude={vessel.longitude}
                latitude={vessel.latitude}
                anchor="center"
                onClick={(e) => {
                    e.originalEvent.stopPropagation();
                    if (props.onVesselClick) {
                        props.onVesselClick(vessel);
                    }
                }}
            >
                <VesselMarker 
                    heading={vessel.heading}
                    color={getVesselColor(vessel.type)}
                    size={16}
                />
            </Marker>
        )),
        [props.aisVessels, props.onVesselClick]
    );

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

                {/* Vessel Popup */}
                {props.vesselPopup && (
                    <Popup
                        longitude={props.vesselPopup.longitude}
                        latitude={props.vesselPopup.latitude}
                        anchor="bottom"
                        onClose={() => props.onVesselPopupClose && props.onVesselPopupClose()}
                        closeButton={true}
                        closeOnClick={false}
                    >
                        <div style={{ padding: '10px', minWidth: '200px' }}>
                            <h4 style={{ margin: '0 0 10px 0' }}>{props.vesselPopup.name}</h4>
                            <div><strong>MMSI:</strong> {props.vesselPopup.mmsi}</div>
                            <div><strong>Type:</strong> {props.vesselPopup.type}</div>
                            <div><strong>Speed:</strong> {props.vesselPopup.speed.toFixed(1)} knots</div>
                            <div><strong>Heading:</strong> {props.vesselPopup.heading}°</div>
                            <div><strong>Position:</strong> {props.vesselPopup.latitude.toFixed(4)}, {props.vesselPopup.longitude.toFixed(4)}</div>
                            <div style={{ fontSize: '12px', color: '#666', marginTop: '5px' }}>
                                Last update: {props.vesselPopup.lastUpdate.toLocaleTimeString()}
                            </div>
                        </div>
                    </Popup>
                )}

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



            </Map>

            <ControlPanel />
        </Box>
    );
}