# Frontend Dockerfile - Multi-stage build for Next.js
# Stage 1: Dependencies
FROM node:22-alpine AS deps
WORKDIR /app

# Install dependencies based on the preferred package manager
COPY package.json package-lock.json* ./
RUN npm ci

# Stage 2: Builder
FROM node:22-alpine AS builder
WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules

# Copy source code
COPY . .

# Set environment variables for build
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production

# NEXT_PUBLIC_* vars must be available at build time so Next.js inlines them
# into the client bundle.  docker-compose supplies these via build.args.
ARG NEXT_PUBLIC_NETWORK
ARG NEXT_PUBLIC_WS_URL
ARG NEXT_PUBLIC_CONTRACT_AD_REGISTRY
ARG NEXT_PUBLIC_CONTRACT_ANALYTICS_AGGREGATOR
ARG NEXT_PUBLIC_CONTRACT_AUCTION_ENGINE
ARG NEXT_PUBLIC_CONTRACT_CAMPAIGN_ORCHESTRATOR
ARG NEXT_PUBLIC_CONTRACT_DISPUTE_RESOLUTION
ARG NEXT_PUBLIC_CONTRACT_ESCROW_VAULT
ARG NEXT_PUBLIC_CONTRACT_FRAUD_PREVENTION
ARG NEXT_PUBLIC_CONTRACT_GOVERNANCE_DAO
ARG NEXT_PUBLIC_CONTRACT_GOVERNANCE_TOKEN
ARG NEXT_PUBLIC_CONTRACT_IDENTITY_REGISTRY
ARG NEXT_PUBLIC_CONTRACT_PAYMENT_PROCESSOR
ARG NEXT_PUBLIC_CONTRACT_PRIVACY_LAYER
ARG NEXT_PUBLIC_CONTRACT_PUBLISHER_REPUTATION
ARG NEXT_PUBLIC_CONTRACT_PUBLISHER_VERIFICATION
ARG NEXT_PUBLIC_CONTRACT_REVENUE_SETTLEMENT
ARG NEXT_PUBLIC_CONTRACT_REWARDS_DISTRIBUTOR
ARG NEXT_PUBLIC_CONTRACT_SUBSCRIPTION_MANAGER
ARG NEXT_PUBLIC_CONTRACT_TARGETING_ENGINE
ARG NEXT_PUBLIC_SIMULATION_ACCOUNT
ARG NEXT_PUBLIC_SENTRY_DSN
ARG SENTRY_DSN
ARG CSP_MODE

# Build Next.js application
RUN npm run build

# Stage 3: Runner (Production)
FROM node:22-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

# Create non-root user for security
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# Copy necessary files from builder
COPY --from=builder /app/public ./public
COPY --from=builder /app/package.json ./package.json

# Copy built application
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

# Switch to non-root user
USER nextjs

# Expose port
EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Start the application
CMD ["node", "server.js"]
