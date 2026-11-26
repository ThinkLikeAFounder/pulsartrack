# Backend Dockerfile - Multi-stage build for TypeScript Express API
# Build from the repository root: docker build -f backend/Dockerfile .
# Stage 0: Build the shared @pulsartrack/sentry-scrubber package.
# backend/package.json and frontend/package.json reference it as
# "file:../packages/sentry-scrubber". The build context is the repository root
# (see docker-compose.yml), and WORKDIR /app makes "../packages" resolve to
# /packages, so the package is placed there in every stage that needs it.
FROM node:22-alpine AS scrubber
WORKDIR /packages/sentry-scrubber
COPY packages/sentry-scrubber/ ./
RUN npm install --no-audit --no-fund --no-package-lock \
    && npm run build \
    && npm prune --omit=dev

# Stage 1: Dependencies - Install everything for building
FROM node:22-alpine AS deps
WORKDIR /app

# Shared package that node_modules/@pulsartrack/sentry-scrubber links to
COPY --from=scrubber /packages/sentry-scrubber /packages/sentry-scrubber

# Copy package files
COPY backend/package.json backend/package-lock.json* ./

COPY backend/prisma ./prisma

# Install all dependencies (development + production)
RUN npm ci && npm cache clean --force

# Stage 2: Builder
FROM node:22-alpine AS builder
WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=scrubber /packages/sentry-scrubber /packages/sentry-scrubber
COPY backend/package.json ./

# Copy source code
COPY backend/ .

# Build TypeScript
RUN npm run build

# Stage 3: Runner (Production)
FROM node:22-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production

# Create non-root user for security
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 expressjs

# Copy everything from builder to ensure we have dependencies for pruning
COPY backend/package.json ./
COPY --from=deps /app/node_modules ./node_modules
COPY --from=scrubber /packages/sentry-scrubber /packages/sentry-scrubber

# Prune devDependencies for production
RUN npm prune --production

# Copy built application from builder
COPY --from=builder /app/dist ./dist

# The schema and migrations are plain SQL that tsc does not copy into dist
COPY --from=builder /app/src/db/schema.sql ./dist/db/schema.sql
COPY --from=builder /app/src/db/migrations ./dist/db/migrations

# Prisma is required by the generated client at runtime
COPY --from=builder /app/prisma ./prisma

# Switch to non-root user
USER expressjs

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3001/health', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Start the application
CMD ["node", "dist/index.js"]
