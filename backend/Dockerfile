# Backend Dockerfile - Multi-stage build for TypeScript Express API
# Stage 1: Dependencies - Install everything for building
FROM node:20-alpine AS deps
WORKDIR /app

# Copy package files
COPY package.json package-lock.json* ./

# Install all dependencies (development + production)
RUN npm ci && npm cache clean --force

# Stage 2: Builder
FROM node:20-alpine AS builder
WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY package.json ./

# Copy source code
COPY . .

# Build TypeScript
RUN npm run build

# Stage 3: Runner (Production)
FROM node:20-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production

# Create non-root user for security
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 expressjs

# Copy everything from builder to ensure we have dependencies for pruning
COPY package.json ./
COPY --from=deps /app/node_modules ./node_modules

# Prune devDependencies for production
RUN npm prune --production

# Copy built application from builder
COPY --from=builder /app/dist ./dist

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
