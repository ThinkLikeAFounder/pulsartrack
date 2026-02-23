import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
    test: {
        globals: true,
        environment: 'node',
        setupFiles: ['./src/test-setup.ts'],
        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'html'],
            exclude: ['node_modules/', 'dist/', 'src/**/*.test.ts', 'src/test-setup.ts'],
        },
        include: ['src/**/*.test.ts'],
        env: {
            NODE_ENV: 'test',
            DATABASE_URL: 'postgresql://postgres:password@localhost:5433/pulsartrack_test?schema=public',
            REDIS_URL: 'redis://localhost:6379',
            JWT_SECRET: 'test-secret-key-12345',
        },
    },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
});
