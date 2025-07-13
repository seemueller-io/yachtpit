import {useState, useMemo} from 'react';
import Map, {
    Marker,
    Popup,
    NavigationControl,
    FullscreenControl,
    ScaleControl,
    GeolocateControl
} from 'react-map-gl/mapbox';

import ControlPanel from './control-panel';
import Pin from './pin';

import PORTS from './test_data/nautical-base-data.json';
import {Box} from "@chakra-ui/react";


export default function MapNext(props: any = {mapboxPublicKey: ""} as any) {
    const [popupInfo, setPopupInfo] = useState(null);

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

    return (
        <Box>
            <Map
                initialViewState={{
                    latitude: 40,
                    longitude: -100,
                    zoom: 3.5,
                    bearing: 0,
                    pitch: 0
                }}

                mapStyle="mapbox://styles/geoffsee/cmd1qz39x01ga01qv5acea02y"
                mapboxAccessToken={props.mapboxPublicKey}
                style={{position: "fixed", width: '100%', height: '100%', bottom: 0, top: 0, left: 0, right: 0}}
            >
                <GeolocateControl position="top-left" />
                <FullscreenControl position="top-left" />
                <NavigationControl position="top-left" />
                <ScaleControl />

                {pins}

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