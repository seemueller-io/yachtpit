// Types for bevy_flurx_ipc communication
export interface GpsPosition {
    latitude: number;
    longitude: number;
    zoom: number;
}

export interface VesselStatus {
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