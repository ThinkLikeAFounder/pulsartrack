import '@testing-library/jest-dom';
import { vi, beforeAll, afterAll, afterEach } from 'vitest';

// Mock Freighter API
vi.mock('@stellar/freighter-api', () => ({
    requestAccess: vi.fn(),
    isAllowed: vi.fn(),
    getAddress: vi.fn(),
    signTransaction: vi.fn(),
    getNetworkDetails: vi.fn(),
    isConnected: vi.fn(() => Promise.resolve({ isConnected: true })),
}));

// Mock Stellar SDK
vi.mock('@stellar/stellar-sdk', async (importOriginal) => {
    const actual = await importOriginal<typeof import('@stellar/stellar-sdk')>();
    return {
        ...actual,
        rpc: {
            ...actual.rpc,
            Server: vi.fn().mockImplementation(() => ({
                getAccount: vi.fn(),
                simulateTransaction: vi.fn(),
                sendTransaction: vi.fn(),
                getTransaction: vi.fn(),
            })),
            Api: {
                isSimulationError: vi.fn(),
                isSimulationSuccess: vi.fn(),
                GetTransactionStatus: {
                    SUCCESS: 'SUCCESS',
                    FAILED: 'FAILED',
                },
            },
        },
    };
});

// Mock WebSocket
class MockWebSocket {
    onopen: () => void = () => { };
    onmessage: (event: { data: string }) => void = () => { };
    onerror: () => void = () => { };
    onclose: () => void = () => { };
    readyState: number = 0;

    constructor(public url: string) {
        setTimeout(() => {
            this.readyState = 1; // OPEN
            this.onopen();
        }, 0);
    }

    send(data: string) { }
    close() {
        this.readyState = 3; // CLOSED
        this.onclose();
    }
}

// @ts-ignore
global.WebSocket = MockWebSocket;

// Mock window.freighter
beforeAll(() => {
    Object.defineProperty(window, 'freighter', {
        value: {},
        writable: true,
    });
});

afterEach(() => {
    vi.clearAllMocks();
});
