import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useRealAISProvider } from '../src/real-ais-provider.tsx';

// Mock WebSocket
class MockWebSocket {
    static instances: MockWebSocket[] = [];
    static CONNECTING = 0;
    static OPEN = 1;
    static CLOSING = 2;
    static CLOSED = 3;

    readyState: number = MockWebSocket.CONNECTING;
    onopen: ((event: Event) => void) | null = null;
    onmessage: ((event: MessageEvent) => void) | null = null;
    onerror: ((event: Event) => void) | null = null;
    onclose: ((event: CloseEvent) => void) | null = null;

    constructor(public url: string) {
        MockWebSocket.instances.push(this);
        
        // Simulate connection opening after a short delay
        setTimeout(() => {
            this.readyState = MockWebSocket.OPEN;
            if (this.onopen) {
                this.onopen(new Event('open'));
            }
        }, 10);
    }

    send(data: string) {
        console.log('MockWebSocket send:', data);
    }

    close() {
        this.readyState = MockWebSocket.CLOSED;
        if (this.onclose) {
            this.onclose(new CloseEvent('close', { wasClean: true }));
        }
    }

    static reset() {
        MockWebSocket.instances = [];
    }

    static getConnectionCount() {
        return MockWebSocket.instances.filter(ws => 
            ws.readyState === MockWebSocket.OPEN || 
            ws.readyState === MockWebSocket.CONNECTING
        ).length;
    }
}

// Replace global WebSocket with mock
(global as any).WebSocket = MockWebSocket;

describe('useRealAISProvider WebSocket Connection Management', () => {
    beforeEach(() => {
        MockWebSocket.reset();
        vi.clearAllMocks();
    });

    afterEach(() => {
        MockWebSocket.reset();
    });

    test('should create only one WebSocket connection on initial render', async () => {
        const boundingBox = {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0
        };

        const { result } = renderHook(() => useRealAISProvider(boundingBox));

        // Wait for connection to be established
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 50));
        });

        expect(MockWebSocket.instances).toHaveLength(1);
        expect(MockWebSocket.getConnectionCount()).toBe(1);
        expect(result.current.isConnected).toBe(true);
    });

    test('should not create multiple connections when bounding box changes', async () => {
        const initialBoundingBox = {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0
        };

        const { result, rerender } = renderHook(
            ({ boundingBox }) => useRealAISProvider(boundingBox),
            { initialProps: { boundingBox: initialBoundingBox } }
        );

        // Wait for initial connection
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 50));
        });

        expect(MockWebSocket.instances).toHaveLength(1);
        expect(MockWebSocket.getConnectionCount()).toBe(1);

        // Change bounding box multiple times
        const newBoundingBox1 = {
            sw_lat: 34.0,
            sw_lon: -120.0,
            ne_lat: 35.0,
            ne_lon: -119.0
        };

        const newBoundingBox2 = {
            sw_lat: 35.0,
            sw_lon: -121.0,
            ne_lat: 36.0,
            ne_lon: -120.0
        };

        await act(async () => {
            rerender({ boundingBox: newBoundingBox1 });
            await new Promise(resolve => setTimeout(resolve, 20));
        });

        await act(async () => {
            rerender({ boundingBox: newBoundingBox2 });
            await new Promise(resolve => setTimeout(resolve, 20));
        });

        // Should still have only one connection
        expect(MockWebSocket.getConnectionCount()).toBe(1);
        expect(result.current.isConnected).toBe(true);
    });

    test('should properly cleanup connection when component unmounts', async () => {
        const boundingBox = {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0
        };

        const { result, unmount } = renderHook(() => useRealAISProvider(boundingBox));

        // Wait for connection
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 50));
        });

        expect(MockWebSocket.getConnectionCount()).toBe(1);
        expect(result.current.isConnected).toBe(true);

        // Unmount component
        unmount();

        // Connection should be closed
        expect(MockWebSocket.instances[0].readyState).toBe(MockWebSocket.CLOSED);
    });

    test('should not create connection when isActive is false', async () => {
        const boundingBox = {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0
        };

        // Create a custom hook that starts with isActive = false
        const { result } = renderHook(() => {
            const provider = useRealAISProvider(boundingBox);
            // Set inactive immediately on first render
            if (provider.isActive) {
                provider.setIsActive(false);
            }
            return provider;
        });

        // Wait a bit to ensure no connection is created
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 100));
        });

        expect(MockWebSocket.instances).toHaveLength(0);
        expect(result.current.isConnected).toBe(false);
        expect(result.current.isActive).toBe(false);
    });

    test('should handle reconnection properly without creating multiple connections', async () => {
        const boundingBox = {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0
        };

        const { result } = renderHook(() => useRealAISProvider(boundingBox));

        // Wait for initial connection
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 50));
        });

        expect(MockWebSocket.getConnectionCount()).toBe(1);

        // Simulate connection loss
        await act(async () => {
            const ws = MockWebSocket.instances[0];
            ws.readyState = MockWebSocket.CLOSED;
            if (ws.onclose) {
                ws.onclose(new CloseEvent('close', { wasClean: false }));
            }
            await new Promise(resolve => setTimeout(resolve, 3100)); // Wait for reconnection timeout
        });

        // Should have attempted reconnection but still only one active connection
        expect(MockWebSocket.getConnectionCount()).toBe(1);
    });

    test('should send bounding box configuration on connection', async () => {
        const boundingBox = {
            sw_lat: 33.0,
            sw_lon: -119.0,
            ne_lat: 34.0,
            ne_lon: -118.0
        };

        const sendSpy = vi.spyOn(MockWebSocket.prototype, 'send');

        const { result } = renderHook(() => useRealAISProvider(boundingBox));

        // Wait for connection and bounding box message
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 50));
        });

        expect(sendSpy).toHaveBeenCalledWith(
            JSON.stringify({
                type: 'set_bounding_box',
                bounding_box: boundingBox
            })
        );
    });
});