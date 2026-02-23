import { renderHook, act } from '@testing-library/react';
import { useWebSocket } from './useWebSocket';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { getPulsarWebSocket } from '@/lib/websocket';

// Mock websocket lib
vi.mock('@/lib/websocket', async (importOriginal) => {
    const actual = await importOriginal<typeof import('@/lib/websocket')>();

    const mockWs = {
        on: vi.fn(() => vi.fn()),
        connect: vi.fn(),
        disconnect: vi.fn(),
        isConnected: false,
        emit: vi.fn(),
    };

    return {
        ...actual,
        getPulsarWebSocket: vi.fn(() => mockWs),
    };
});

describe('useWebSocket', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should initialize with disconnected state', () => {
        const { result } = renderHook(() => useWebSocket());
        expect(result.current.isConnected).toBe(false);
        expect(result.current.lastEvent).toBeNull();
    });

    it('should subscribe to events on mount', () => {
        const mockWs = getPulsarWebSocket();
        renderHook(() => useWebSocket({ events: ['bid_placed'] }));

        expect(mockWs.on).toHaveBeenCalledWith('connected', expect.any(Function));
        expect(mockWs.on).toHaveBeenCalledWith('error', expect.any(Function));
        expect(mockWs.on).toHaveBeenCalledWith('bid_placed', expect.any(Function));
    });

    it('should update connection status when websocket connects', () => {
        const mockWs = getPulsarWebSocket() as any;
        let connectedHandler: Function = () => { };

        // Capture the connected handler
        vi.mocked(mockWs.on).mockImplementation((event: string, handler: Function) => {
            if (event === 'connected') connectedHandler = handler;
            return vi.fn();
        });

        const { result } = renderHook(() => useWebSocket());

        act(() => {
            connectedHandler();
        });

        expect(result.current.isConnected).toBe(true);
    });

    it('should update last event when a message is received', () => {
        const mockWs = getPulsarWebSocket() as any;
        let allHandler: Function = () => { };

        vi.mocked(mockWs.on).mockImplementation((event: string, handler: Function) => {
            if (event === 'all') allHandler = handler;
            return vi.fn();
        });

        const { result } = renderHook(() => useWebSocket());

        const mockEvent = {
            type: 'bid_placed',
            data: { amount: '100' },
            timestamp: Date.now(),
        };

        act(() => {
            allHandler(mockEvent);
        });

        expect(result.current.lastEvent).toEqual(mockEvent);
        expect(result.current.eventHistory).toContain(mockEvent);
    });
});
