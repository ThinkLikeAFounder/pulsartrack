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
            DATABASE_URL: 'postgresql://pulsartrack:pulsartrack_dev_password@localhost:5432/pulsartrack_test',
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
