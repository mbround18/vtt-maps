# Redis Multi-Instance Session Sharing

This guide explains how to deploy the VTT Maps application with Redis for session sharing across multiple HTTP server instances.

## Overview

By default, the HTTP server uses cookie-based session storage, which works perfectly for single-instance deployments. However, when running multiple server instances behind a load balancer, cookies created by one instance need to be readable by all others.

**Solution**: Use Redis as a centralized session store that all instances can access.

## Architecture

```mermaid
┌─────────────────────────────────────────────────────────────┐
│                     Load Balancer                           │
│              (reverse proxy, distributes traffic)           │
└─────────┬────────────────────────────────────┬──────────────┘
          │                                    │
    ┌─────▼─────┐  ┌──────────────┐  ┌─────────▼─────┐
    │ Instance 1 │  │   Instance 2 │  │  Instance 3   │
    │ (Port 8081)│  │  (Port 8082) │  │  (Port 8083)  │
    └─────┬─────┘  └──────┬───────┘  └─────────┬─────┘
          │               │                    │
          └───────────────┼────────────────────┘
                          │
                    ┌─────▼─────────────┐
                    │   Redis Server    │
                    │  (Session Store)  │
                    │  (Single Instance)│
                    └───────────────────┘
                          ▲
                          │
                    PostgreSQL (optional)
                    (for persistent data)
```

## Setup Instructions

### 1. Start Redis

#### Option A: Using Docker Compose

Edit your `compose.yaml` to include Redis:

```yaml
services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes

volumes:
  redis-data: {}
```

Then start it:

```bash
docker-compose up -d redis
```

#### Option B: Manual Installation

```bash
# macOS
brew install redis
redis-server

# Ubuntu/Debian
sudo apt-get install redis-server
redis-server

# Or Docker (single container)
docker run -d -p 6379:6379 redis:7-alpine
```

### 2. Verify Redis is Running

```bash
redis-cli ping
# Should return: PONG
```

### 3. Configure Your Environment

The HTTP server automatically detects Redis. Set the Redis connection URL:

```bash
# In your .env file or environment variables
export REDIS_URL=redis://localhost:6379

# OR specify host and port separately
export REDIS_HOST=localhost
export REDIS_PORT=6379
```

### 4. (Optional) Persist Session Key Across Restarts

For production, you may want to use the same session encryption key across server restarts:

```bash
# Generate a 64-byte base64-encoded key
openssl rand -base64 64 > session_key.txt

# Set as environment variable
export SESSION_KEY=$(cat session_key.txt)
```

### 5. Run Multiple Server Instances

Each instance can run on a different port:

```bash
# Terminal 1
export ADDRESS=0.0.0.0 PORT=8081 REDIS_URL=redis://localhost:6379
cargo run --release --bin actix-backend

# Terminal 2
export ADDRESS=0.0.0.0 PORT=8082 REDIS_URL=redis://localhost:6379
cargo run --release --bin actix-backend

# Terminal 3
export ADDRESS=0.0.0.0 PORT=8083 REDIS_URL=redis://localhost:6379
cargo run --release --bin actix-backend
```

Or using Docker:

```bash
docker-compose up --scale backend=3
```

### 6. Configure Load Balancer

Point your reverse proxy (nginx, Apache, HAProxy, etc.) at the three instances:

**Example nginx.conf:**

```nginx
upstream vtt_maps {
    server localhost:8081;
    server localhost:8082;
    server localhost:8083;
}

server {
    listen 80;
    server_name yourdomain.com;

    location / {
        proxy_pass http://vtt_maps;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cookie_path / "/";
    }
}
```

## How It Works

1. **User Logs In**: Browser sends credentials → any instance receives login request
2. **Session Created**: Instance creates session in Redis (not local memory)
3. **Session ID in Cookie**: Instance returns cookie with session ID to browser
4. **Load Balancer Distributes**: Next request might go to different instance
5. **Session Retrieved**: Any instance can read the session from Redis using the ID in the cookie
6. **Request Processed**: All instances have access to the same session data

## Fallback Behavior

If Redis is unavailable:

```
1. Server checks for REDIS_URL or REDIS_HOST environment variables
2. If Redis is available → Uses Redis session store (✅ recommended for multi-instance)
3. If Redis is unavailable → Falls back to cookie-based sessions (⚠️ only works with sticky sessions)
```

The fallback allows graceful degradation - your app won't crash if Redis goes down, but multi-instance deployments should use sticky sessions in that case.

## Monitoring Redis Sessions

```bash
# Connect to Redis
redis-cli

# List all session keys
keys vtt-maps.dnd-apps.dev*

# Check session data
get "vtt-maps.dnd-apps.dev:<session-id>"

# Monitor live commands
monitor

# Check memory usage
info memory

# Set password (production)
requirepass your_strong_password
```

## Session Expiration & TTL

- **Session TTL**: 30 minutes (configurable in `src/hooks/identity.rs`)
- **Redis auto-cleanup**: Sessions automatically expire from Redis after TTL
- **Cookie expiration**: Session cookie expires when browser closes or after TTL

## Production Checklist

- [ ] Redis running on a dedicated host (not same server as app)
- [ ] Redis configured with `requirepass` (strong password)
- [ ] Redis bound to private network (not exposed to internet)
- [ ] Redis persistence enabled (`appendonly yes`)
- [ ] Redis replication or cluster for HA
- [ ] All instances configured with same SESSION_KEY (if using persistence)
- [ ] Load balancer configured for proper cookie handling
- [ ] Monitoring and alerting on Redis health
- [ ] Regular Redis backups

## Troubleshooting

### "Redis initialization failed, falling back to cookie-based sessions"

**Cause**: Redis is not accessible at the configured URL

**Fix**:

```bash
# Test connection
redis-cli -u redis://your-redis-url ping

# Check environment variables
echo $REDIS_URL
echo $REDIS_HOST

# Verify Redis is running and accessible
redis-cli -h <host> -p 6379 ping
```

### Sessions not shared between instances

**Cause**: Likely using fallback cookie-based sessions due to Redis unavailable

**Fix**: See "Redis initialization failed" above

**Or**: Enable sticky sessions on load balancer (temporary workaround, not recommended)

### Redis memory growing unbounded

**Cause**: Sessions not expiring or memory policy not set

**Fix**:

```bash
# In Redis config or CLI
CONFIG SET maxmemory 256mb
CONFIG SET maxmemory-policy allkeys-lru
CONFIG REWRITE
```

### Performance issues with multiple instances

**Cause**: Network latency to Redis or insufficient Redis capacity

**Fix**:

- Use Redis connection pooling (built-in via `redis` crate)
- Monitor network latency: `redis-cli --latency`
- Consider Redis Cluster for HA and sharding
- Profile session read/write operations

## Further Reading

- [Actix-web Session Middleware](https://actix-rs.github.io/actix-web/actixweb/middleware/struct.SessionMiddleware.html)
- [Redis Persistence](https://redis.io/topics/persistence)
- [Redis Security](https://redis.io/topics/security)
- [Redis Cluster Tutorial](https://redis.io/topics/cluster-tutorial)

## Questions?

Refer to `src/hooks/redis_session.rs` for the implementation details.
