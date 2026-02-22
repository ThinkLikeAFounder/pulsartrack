# Docker Setup Guide

## Overview

This guide explains the Docker configuration for PulsarTrack, providing a reproducible local development environment that eliminates manual dependency installation.

## What's Included

The Docker setup includes:

- **Frontend**: Next.js application (port 3000)
- **Backend**: Express API with TypeScript (port 3001)
- **PostgreSQL**: Database server v16 (port 5432)
- **Redis**: Cache and rate limiting v7 (port 6379)

## Quick Start

### 1. Prerequisites

Only Docker is required:

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (includes Docker Compose)
- OR [Docker Engine](https://docs.docker.com/engine/install/) + [Docker Compose](https://docs.docker.com/compose/install/)

### 2. Start the Application

```bash
# Clone the repository
git clone https://github.com/yourusername/pulsartrack.git
cd pulsartrack

# Copy environment file (optional - has sensible defaults)
cp .env.example .env

# Start all services
docker-compose up
```

The first run will take a few minutes to build images. Subsequent starts are much faster.

### 3. Access Services

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:3001
- **Backend Health**: http://localhost:3001/health
- **PostgreSQL**: localhost:5432 (user: pulsartrack, password: pulsartrack_dev_password)
- **Redis**: localhost:6379

## Docker Architecture

### Multi-Stage Builds

Both frontend and backend use multi-stage Docker builds for optimization:

#### Frontend (Next.js)

```
Stage 1 (deps): Install production dependencies
Stage 2 (builder): Install all deps + build Next.js
Stage 3 (runner): Copy built app + production deps only
```

**Benefits:**

- Final image size: ~150MB (vs ~1GB without multi-stage)
- Only production dependencies included
- Faster deployment and startup

#### Backend (Express + TypeScript)

```
Stage 1 (deps): Install production dependencies
Stage 2 (builder): Install all deps + compile TypeScript
Stage 3 (runner): Copy compiled JS + production deps only
```

**Benefits:**

- Final image size: ~120MB
- No TypeScript compiler in production
- Faster cold starts

### Service Dependencies

```
frontend → backend → db
                  → redis
```

- Frontend waits for backend to be healthy
- Backend waits for db and redis to be healthy
- Health checks ensure services are ready before dependent services start

## Docker Compose Services

### Frontend Service

```yaml
Build: frontend/Dockerfile
Port: 3000
Environment: Production mode with testnet configuration
Depends on: backend
```

**Features:**

- Standalone Next.js build
- Non-root user (nextjs:1001)
- Health check via HTTP GET
- Hot reload disabled (use dev mode for development)

### Backend Service

```yaml
Build: backend/Dockerfile
Port: 3001
Environment: Development mode with full logging
Depends on: db, redis
```

**Features:**

- Compiled TypeScript
- Non-root user (expressjs:1001)
- Health check endpoint
- Automatic reconnection to db/redis

### Database Service

```yaml
Image: postgres:16-alpine
Port: 5432
Credentials: pulsartrack / pulsartrack_dev_password
```

**Features:**

- Persistent volume (postgres_data)
- Health check via pg_isready
- Auto-restart on failure

### Redis Service

```yaml
Image: redis:7-alpine
Port: 6379
Persistence: AOF (Append-Only File)
```

**Features:**

- Persistent volume (redis_data)
- Health check via PING
- Data survives container restarts

## Common Commands

### Start Services

```bash
# Start in foreground (see logs)
docker-compose up

# Start in background (detached)
docker-compose up -d

# Start specific service
docker-compose up frontend

# Rebuild and start
docker-compose up --build
```

### Stop Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (deletes database data)
docker-compose down -v

# Stop specific service
docker-compose stop backend
```

### View Logs

```bash
# All services
docker-compose logs

# Follow logs (live)
docker-compose logs -f

# Specific service
docker-compose logs backend

# Last 100 lines
docker-compose logs --tail=100
```

### Execute Commands in Containers

```bash
# Open shell in backend container
docker-compose exec backend sh

# Run npm command in frontend
docker-compose exec frontend npm run lint

# Access PostgreSQL
docker-compose exec db psql -U pulsartrack -d pulsartrack

# Access Redis CLI
docker-compose exec redis redis-cli
```

### Rebuild Services

```bash
# Rebuild all services
docker-compose build

# Rebuild specific service
docker-compose build backend

# Rebuild without cache
docker-compose build --no-cache
```

## Development Workflow

### Making Code Changes

#### Frontend Changes

```bash
# Option 1: Rebuild and restart
docker-compose up --build frontend

# Option 2: Use local dev mode (faster)
cd frontend
npm install
npm run dev
# Access at http://localhost:3000
```

#### Backend Changes

```bash
# Option 1: Rebuild and restart
docker-compose up --build backend

# Option 2: Use local dev mode with Docker db/redis
docker-compose up db redis
cd backend
npm install
npm run dev
# Access at http://localhost:3001
```

### Database Migrations

```bash
# Run Prisma migrations
docker-compose exec backend npx prisma migrate dev

# Generate Prisma client
docker-compose exec backend npx prisma generate

# Seed database
docker-compose exec backend npx prisma db seed
```

### Debugging

```bash
# Check service status
docker-compose ps

# Check service health
docker-compose exec backend wget -qO- http://localhost:3001/health

# View container resource usage
docker stats

# Inspect container
docker-compose exec backend env
```

## Environment Variables

### Default Configuration

The docker-compose.yml includes sensible defaults for local development:

- Database credentials
- Redis connection
- Stellar testnet URLs
- CORS settings

### Custom Configuration

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` to set your deployed contract IDs:

```env
CONTRACT_CAMPAIGN_ORCHESTRATOR=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
CONTRACT_CAMPAIGN_LIFECYCLE=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
# ... etc
```

These variables are automatically loaded by docker-compose.

### Override Environment Variables

```bash
# Set variable for single run
CONTRACT_CAMPAIGN_ORCHESTRATOR=CXXX docker-compose up

# Or export for session
export CONTRACT_CAMPAIGN_ORCHESTRATOR=CXXX
docker-compose up
```

## Volumes and Data Persistence

### Persistent Volumes

```yaml
postgres_data: Database files
redis_data: Redis AOF and RDB files
```

**Location:** Docker manages volume storage (usually `/var/lib/docker/volumes/`)

### Backup Data

```bash
# Backup PostgreSQL
docker-compose exec db pg_dump -U pulsartrack pulsartrack > backup.sql

# Restore PostgreSQL
docker-compose exec -T db psql -U pulsartrack pulsartrack < backup.sql

# Backup Redis
docker-compose exec redis redis-cli SAVE
docker cp pulsartrack-redis:/data/dump.rdb ./redis-backup.rdb
```

### Clear Data

```bash
# Remove all data (fresh start)
docker-compose down -v

# Remove specific volume
docker volume rm pulsartrack_postgres_data
```

## Troubleshooting

### Port Already in Use

```bash
# Check what's using the port
lsof -i :3000  # or :3001, :5432, :6379

# Kill the process
kill -9 <PID>

# Or change port in docker-compose.yml
ports:
  - "3002:3000"  # Map to different host port
```

### Container Won't Start

```bash
# Check logs
docker-compose logs <service-name>

# Check if previous container is still running
docker ps -a

# Remove old containers
docker-compose rm -f

# Rebuild from scratch
docker-compose down -v
docker-compose build --no-cache
docker-compose up
```

### Database Connection Issues

```bash
# Check if database is healthy
docker-compose ps

# Check database logs
docker-compose logs db

# Manually test connection
docker-compose exec backend sh
nc -zv db 5432
```

### Out of Disk Space

```bash
# Check Docker disk usage
docker system df

# Clean up unused images/containers
docker system prune

# Clean up everything (careful!)
docker system prune -a --volumes
```

### Build Failures

```bash
# Clear build cache
docker builder prune

# Rebuild without cache
docker-compose build --no-cache

# Check .dockerignore files
cat frontend/.dockerignore
cat backend/.dockerignore
```

## Production Deployment

### Building for Production

```bash
# Build production images
docker-compose build

# Tag images
docker tag pulsartrack-frontend:latest your-registry/pulsartrack-frontend:v1.0.0
docker tag pulsartrack-backend:latest your-registry/pulsartrack-backend:v1.0.0

# Push to registry
docker push your-registry/pulsartrack-frontend:v1.0.0
docker push your-registry/pulsartrack-backend:v1.0.0
```

### Production Considerations

1. **Environment Variables**: Use secrets management (not .env files)
2. **Database**: Use managed PostgreSQL (AWS RDS, Google Cloud SQL, etc.)
3. **Redis**: Use managed Redis (AWS ElastiCache, Redis Cloud, etc.)
4. **Reverse Proxy**: Add Nginx/Traefik for SSL and load balancing
5. **Monitoring**: Add health check endpoints and logging
6. **Scaling**: Use Kubernetes or Docker Swarm for orchestration

### Security Hardening

```dockerfile
# Use specific versions (not :latest)
FROM node:20.11.0-alpine

# Scan for vulnerabilities
docker scan pulsartrack-frontend

# Run as non-root user (already implemented)
USER nextjs

# Use read-only filesystem where possible
docker run --read-only pulsartrack-frontend
```

## Performance Optimization

### Image Size Optimization

Current sizes:

- Frontend: ~150MB
- Backend: ~120MB
- Total: ~270MB

Already optimized with:

- Multi-stage builds
- Alpine base images
- Production-only dependencies
- No dev tools in final image

### Build Speed Optimization

```bash
# Use BuildKit for faster builds
DOCKER_BUILDKIT=1 docker-compose build

# Cache dependencies
# (Already implemented via separate COPY of package.json)

# Parallel builds
docker-compose build --parallel
```

### Runtime Optimization

```yaml
# Limit resources
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: "0.5"
          memory: 512M
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Docker Build

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build images
        run: docker-compose build
      - name: Run tests
        run: docker-compose run backend npm test
```

### GitLab CI Example

```yaml
build:
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker-compose build
    - docker-compose run backend npm test
```

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Next.js Docker Documentation](https://nextjs.org/docs/deployment#docker-image)
- [Node.js Docker Best Practices](https://github.com/nodejs/docker-node/blob/main/docs/BestPractices.md)

## Support

For issues or questions:

1. Check the [Troubleshooting](#troubleshooting) section
2. Review Docker logs: `docker-compose logs`
3. Open an issue on GitHub
4. Join our Discord community
