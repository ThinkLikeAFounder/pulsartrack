import { beforeAll, afterAll, vi } from 'vitest';

// Mock Redis (ioredis)
vi.mock('ioredis', () => {
    return {
        default: class MockRedis {
            on = vi.fn();
            quit = vi.fn().mockResolvedValue('OK');
            defineCommand = vi.fn();
            get = vi.fn().mockResolvedValue(null);
            set = vi.fn().mockResolvedValue('OK');
            constructor() { }
        },
    };
});

// Mock Rate Limiter
vi.mock('rate-limiter-flexible', () => {
    return {
        RateLimiterRedis: class MockRateLimiter {
            consume = vi.fn().mockResolvedValue({});
            constructor() { }
        },
    };
});

// Mock Prisma
vi.mock('./db/prisma', () => ({
    default: {
        $connect: vi.fn().mockResolvedValue(undefined),
        $disconnect: vi.fn().mockResolvedValue(undefined),
        campaign: {
            findMany: vi.fn(),
            findUnique: vi.fn(),
            create: vi.fn(),
            update: vi.fn(),
            delete: vi.fn(),
            count: vi.fn(),
        },
        publisher: {
            findMany: vi.fn(),
            findUnique: vi.fn(),
            create: vi.fn(),
            update: vi.fn(),
            delete: vi.fn(),
            count: vi.fn(),
        },
        auction: {
            findMany: vi.fn(),
            findUnique: vi.fn(),
            create: vi.fn(),
            update: vi.fn(),
            delete: vi.fn(),
            count: vi.fn(),
        },
        bid: {
            create: vi.fn(),
        },
    },
}));

// Mock PG Pool
vi.mock('./config/database', () => ({
    default: {
        query: vi.fn(),
        connect: vi.fn().mockResolvedValue({
            query: vi.fn().mockResolvedValue({ rows: [], rowCount: 0 }),
            release: vi.fn(),
        }),
    },
    checkDbConnection: vi.fn().mockResolvedValue(true),
}));

// Mock Soroban Client
vi.mock('./services/soroban-client', () => ({
    callReadOnly: vi.fn(),
    toAddressScVal: vi.fn(),
}));

beforeAll(async () => {
    // Silent console in tests unless needed
    vi.spyOn(console, 'log').mockImplementation(() => { });
    vi.spyOn(console, 'error').mockImplementation(() => { });
    vi.spyOn(console, 'warn').mockImplementation(() => { });
});

afterAll(async () => {
});
