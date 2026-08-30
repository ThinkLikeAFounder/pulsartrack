import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import apiRoutes from './api/routes';
import redisClient from './config/redis';
import { errorHandler, rateLimit, configureRateLimiters } from './middleware/auth';

const app = express();
const RESPONSE_TIMEOUT_MS = Number.parseInt(
    process.env.EXPRESS_RESPONSE_TIMEOUT_MS || '30000',
    10,
);

configureRateLimiters(redisClient);

app.use(helmet());
app.use(cors({
    origin: process.env.CORS_ORIGIN || 'http://localhost:3000',
    credentials: true,
}));

if (process.env.NODE_ENV !== 'test') {
    app.use(morgan('combined'));
}

app.use(express.json({ limit: '10mb' }));
app.use((req, res, next) => {
    req.setTimeout(RESPONSE_TIMEOUT_MS);
    res.setTimeout(RESPONSE_TIMEOUT_MS, () => {
        if (!res.headersSent) {
            res.status(504).json({ error: 'Gateway timeout' });
        }
    });
    next();
});
app.use(rateLimit());

// Safely serialize BigInt values as strings to avoid precision loss
app.set('json replacer', (_key: string, value: unknown) =>
    typeof value === 'bigint' ? value.toString() : value
);

app.use('/api', apiRoutes);

app.use((_req, res) => {
    res.status(404).json({ error: 'Route not found' });
});

app.use(errorHandler);

export default app;
