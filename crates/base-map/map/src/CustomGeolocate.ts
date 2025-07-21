import type {Geolocation} from "@/MapNext.tsx";


export class NativeGeolocation implements Geolocation {
    constructor({clearWatch, getCurrentPosition, watchPosition}: {
        clearWatch: (watchId: number) => void;
        getCurrentPosition: (successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions) => void;
        watchPosition: (successCallback: PositionCallback, errorCallback?: PositionErrorCallback | null, options?: PositionOptions) => number;
    }) {
        this.clearWatch = clearWatch;
        this.watchPosition = watchPosition;
        this.getCurrentPosition = getCurrentPosition;
    }

    clearWatch(_watchId: number): void {
        throw new Error('Method not implemented.');
    }

    getCurrentPosition(_successCallback: PositionCallback, _errorCallback?: PositionErrorCallback | null, _options?: PositionOptions): void {
        throw new Error('Method not implemented.');
    }

    watchPosition(_successCallback: PositionCallback, _errorCallback?: PositionErrorCallback | null, _options?: PositionOptions): number {
        throw new Error('Method not implemented.');
    }

}
