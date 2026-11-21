import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import * as Sentry from '@sentry/node';
import apiRoutes from './api/routes';
import redisClient from './config/redis';
import { errorHandler, rateLimit, configureRateLimiters } from './middleware/auth';
import { getTrustedProxyHops } from './lib/client-ip';
import { loadEnv } from './config/env';

const app = express();
const env = loadEnv();
// Honor X-Forwarded-For only from the configured number of trusted proxies
// (TRUST_PROXY_HOPS); 0 means req.ip is always the socket address.
app.set('trust proxy', getTrustedProxyHops());
const RESPONSE_TIMEOUT_MS = env.EXPRESS_RESPONSE_TIMEOUT_MS;

configureRateLimiters(redisClient);

app.use(helmet());
app.use(cors({
    origin: process.env.CORS_ORIGIN || 'http://localhost:3000',
    credentials: true,
}));

if (process.env.NODE_ENV !== 'test') {
    app.use(morgan('combined'));
}

// Rate limiter must come before express.json so rejected clients don't
// consume resources parsing bodies
app.use(rateLimit());

app.use((req, res, next) => {
    req.setTimeout(RESPONSE_TIMEOUT_MS);
    res.setTimeout(RESPONSE_TIMEOUT_MS, () => {
        if (!res.headersSent) {
            res.status(504).json({ error: 'Gateway timeout' });
        }
    });
    next();
});

app.use(express.json({ limit: '10mb' }));

// Safely serialize BigInt values as strings to avoid precision loss
app.set('json replacer', (_key: string, value: unknown) =>
    typeof value === 'bigint' ? value.toString() : value
);

app.use('/api', apiRoutes);

app.use((_req, res) => {
    res.status(404).json({ error: 'Route not found' });
});

Sentry.setupExpressErrorHandler(app);
app.use(errorHandler);

export default app;
