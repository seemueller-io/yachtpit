import {useMemo, useCallback, useRef} from 'react';
import Map, {
    Source, Layer,
    NavigationControl, FullscreenControl, ScaleControl, GeolocateControl,
    type MapRef
} from 'react-map-gl/mapbox';
import {Box} from '@chakra-ui/react';
import type {Feature, FeatureCollection, Point} from 'geojson';
import PORTS from './test_data/nautical-base-data.json';
import type {VesselData} from './ais-provider';

export interface Geolocation {
    clearWatch(watchId: number): void;
    getCurrentPosition(
        successCallback: PositionCallback,
        errorCallback?: PositionErrorCallback | null,
        options?: PositionOptions
    ): void;
    watchPosition(
        successCallback: PositionCallback,
        errorCallback?: PositionErrorCallback | null,
        options?: PositionOptions
    ): number;
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

export default function GeoMap(props: MapNextProps) {
    const mapRef = useRef<MapRef | null>(null);

    const portsGeoJSON = useMemo<FeatureCollection<Point>>(() => ({
        type: 'FeatureCollection',
        features: PORTS.map(port => ({
            type: 'Feature',
            geometry: {type: 'Point', coordinates: [port.longitude, port.latitude]},
            properties: {city: port.city, state: port.state}
        } as Feature<Point>))
    }), []);

    const handleGeolocate = useCallback((pos: GeolocationPosition) => {
        console.log('User location loaded:', pos);
    }, []);

    return (
        <Box>
            <Map
                ref={mapRef}
                initialViewState={{
                    latitude: props.mapView?.latitude ?? 40,
                    longitude: props.mapView?.longitude ?? -100,
                    zoom: props.mapView?.zoom ?? 3.5,
                    bearing: 0,
                    pitch: 0
                }}
                key={`${props.mapView?.latitude}-${props.mapView?.longitude}-${props.mapView?.zoom}`}
                mapStyle={props.layer?.value ?? 'mapbox://styles/mapbox/standard'}
                mapboxAccessToken={props.mapboxPublicKey}
                style={{ position: 'fixed', inset: 0 }}
            >
                <Source id="ports" type="geojson" data={portsGeoJSON}>
                    <Layer
                        id="ports-layer"
                        type="circle"
                        paint={{
                            'circle-radius': 6,
                            'circle-color': '#007bff',
                            'circle-stroke-width': 1,
                            'circle-stroke-color': '#ffffff'
                        }}
                    />
                </Source>

                <GeolocateControl
                    showUserHeading={true}
                    showUserLocation={true}
                    geolocation={props.geolocation}
                    position="top-left"
                    onGeolocate={handleGeolocate}
                />
                <FullscreenControl position="top-left" />
                <NavigationControl position="top-left" />
                <ScaleControl />
            </Map>
        </Box>
    );
}
