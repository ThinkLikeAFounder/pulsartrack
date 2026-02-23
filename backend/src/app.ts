import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import Redis from 'ioredis';
import apiRoutes from './api/routes';
import { errorHandler, rateLimit, configureRateLimiters } from './middleware/auth';

const app = express();

// Redis connection for rate limiting
const redisUrl = process.env.REDIS_URL || 'redis://localhost:6379';
export const redisClient = new Redis(redisUrl, {
    enableOfflineQueue: false,
    maxRetriesPerRequest: 1,
});

if (process.env.NODE_ENV !== 'test') {
    redisClient.on('connect', () => console.log('[Redis] Connected'));
    redisClient.on('error', (err) => console.error('[Redis] Error:', err.message));
}

// Initialize Redis-backed rate limiters
configureRateLimiters(redisClient);

// Middleware
app.use(helmet());
app.use(cors({
    origin: process.env.CORS_ORIGIN || 'http://localhost:3000',
    credentials: true,
}));

if (process.env.NODE_ENV !== 'test') {
    app.use(morgan('combined'));
}

app.use(express.json({ limit: '10mb' }));
app.use(rateLimit());

// API routes
app.use('/api', apiRoutes);

// 404 handler
app.use((_req, res) => {
    res.status(404).json({ error: 'Route not found' });
});

// Error handler
app.use(errorHandler);

export default app;
