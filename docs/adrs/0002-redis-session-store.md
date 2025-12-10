# ADR-0002: Redis-Based Session Store for Kubernetes Horizontal Scaling

**Status:** Accepted

## Context

The VTT Maps application needs to support horizontal scaling in Kubernetes environments where multiple independent pod replicas must share session state. Currently, the system uses cookie-based session storage, which only works for single-instance deployments or sticky session load balancing.

As the application grows:

1. Kubernetes deployments require stateless application design
2. Pod replicas may be created/destroyed dynamically based on load
3. Load balancers cannot guarantee the same user reaches the same pod (nor should they)
4. Session data must be shared across all running instances
5. Sticky sessions defeat the purpose of horizontal scaling and prevent optimal load distribution

The current cookie-only approach would require:

- Sticky session load balancing (performance anti-pattern)
- Complex session replication between pods
- Session loss when pods are terminated
- Inability to leverage Kubernetes' auto-scaling capabilities

## Decision

> In the context of deploying a web application to Kubernetes, facing the need for stateless horizontal scaling where multiple pod replicas must share user sessions, we decided for **Redis-based distributed session storage** to achieve true horizontal scalability and pod independence, accepting the operational requirement to run and maintain a Redis instance (or cluster), because it enables seamless scaling, allows users to be routed to any pod replica without session loss, and provides industry-standard session sharing across stateless application instances.

## Considered Options

### Option 1: Sticky Session Load Balancing

Route all requests from a user to the same pod based on session affinity.

**Pros:**

- No changes to application code
- Session data stays local to pod
- Simple to understand

**Cons:**

- Defeats the purpose of horizontal scaling
- Reduces load distribution efficiency
- Pod termination causes session loss
- Prevents auto-scaling optimizations
- Not suitable for Kubernetes workloads

**Effort/Complexity:** Low (but architecturally poor)

---

### Option 2: Database-Backed Sessions

Store sessions in PostgreSQL alongside application data.

**Pros:**

- All instances share same data store
- Sessions persist across pod restarts
- Can be queried/managed via SQL
- Familiar relational approach

**Cons:**

- Database query required on every request (performance impact)
- Increases load on primary database
- Session operations contend with application data
- Slower than in-memory cache
- Not optimized for high-frequency access patterns

**Effort/Complexity:** Medium

---

### Option 3: Redis-Based Session Store (Chosen)

Use Redis as a dedicated, fast session storage layer via `actix-session` with `RedisSessionStore`.

**Pros:**

- Stateless application design (pods are truly interchangeable)
- Zero session loss during scaling events or pod restarts
- Sub-millisecond session lookups (in-memory cache)
- No database query overhead per request
- Auto-scaling works seamlessly (Kubernetes can spin up/down pods freely)
- Industry standard for distributed session management
- Lightweight, purpose-built for this use case
- Easy monitoring and debugging with Redis CLI
- Built-in expiration handling (`EXPIRE` key)
- Graceful fallback to cookie sessions if Redis unavailable

**Cons:**

- Additional infrastructure component to manage
- Requires Redis deployment/maintenance
- Session loss if Redis goes down (mitigated with Sentinel/Cluster)
- Another system to monitor and troubleshoot

**Effort/Complexity:** Medium-High

---

### Option 4: JWT Tokens (Session-less)

Use stateless JWT tokens stored in cookies, no server-side session storage.

**Pros:**

- Fully stateless (no session store needed)
- Simplest infrastructure requirement
- Works well with Kubernetes

**Cons:**

- Cannot revoke tokens before expiration
- Larger cookie size
- Token compromise affects all instances
- No ability to invalidate sessions on logout
- Misaligned with `actix-session` framework

**Effort/Complexity:** Low (but architectural mismatch)

## Consequences

### Positive

- ✅ **True Horizontal Scaling**: Pods are fully stateless and interchangeable; Kubernetes can scale up/down without session concerns
- ✅ **Zero Session Loss**: Users remain authenticated across pod terminations and new pod creation
- ✅ **Performance**: Sub-millisecond session access (in-memory cache) vs. database queries
- ✅ **Auto-Scaling Ready**: Kubernetes HPA (Horizontal Pod Autoscaler) can freely manage replicas
- ✅ **Load Balancing**: Users can be routed to any pod; load distribution is optimal
- ✅ **Graceful Degradation**: Application continues with cookie-only sessions if Redis is unavailable
- ✅ **Cloud Native**: Aligns with modern Kubernetes/containerized deployment patterns

### Negative

- ⚠️ **Operational Complexity**: Redis must be deployed and maintained (can be managed service: AWS ElastiCache, Google Cloud Memorystore, etc.)
- ⚠️ **Failure Mode**: Redis unavailability reverts to single-instance cookie sessions (acceptable fallback)
- ⚠️ **Monitoring Required**: Redis health, memory usage, and connection pool must be monitored
- ⚠️ **Data Persistence**: Need strategy for Redis backup/recovery in production

## Implementation Details

### Session Flow

```
User Login
  ↓
Instance 1 creates session in Redis → Returns session ID in cookie
  ↓
Load Balancer routes next request → Instance 2/3/N
  ↓
Any instance reads session from Redis using cookie → User authenticated
```

### Environment Configuration

```bash
# Production (Kubernetes)
REDIS_URL=redis://redis-service:6379

# Development (Docker Compose)
REDIS_URL=redis://redis:6379

# Fallback (Single instance)
# REDIS_URL unset → Uses cookie-based sessions
```

### File Changes

- **`src/hooks/redis_session.rs`**: Async middleware builder for Redis sessions with fallback
- **`Cargo.toml`**: Added `redis` crate with `actix-session` `redis-session` feature
- **`compose.yaml`**: Added Redis service with persistence
- **`docs/REDIS_MULTI_INSTANCE.md`**: Complete deployment and troubleshooting guide

## Kubernetes Deployment Example

```yaml
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vtt-maps-backend
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
  selector:
    matchLabels:
      app: vtt-maps-backend
  template:
    metadata:
      labels:
        app: vtt-maps-backend
    spec:
      containers:
        - name: backend
          image: vtt-maps:latest
          env:
            - name: REDIS_URL
              value: redis://redis-service:6379
          ports:
            - containerPort: 8080
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 10

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: vtt-maps-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: vtt-maps-backend
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

With this setup, Kubernetes can automatically scale from 3 to 10 replicas based on CPU usage, with users seamlessly routed between any pod instance and their sessions preserved via Redis.

## Monitoring & Operations

### Health Checks

```bash
# Verify Redis connectivity
redis-cli -u redis://redis-service:6379 ping

# Monitor session keys
redis-cli -u redis://redis-service:6379 KEYS "vtt-maps*"

# Check memory usage
redis-cli -u redis://redis-service:6379 INFO memory
```

### Production Deployment Strategies

1. **Managed Redis Service**: AWS ElastiCache, GCP Cloud Memorystore, Azure Cache for Redis
   - No operational burden
   - Automatic backups and failover
   - Built-in monitoring

2. **Redis Sentinel**: Self-hosted with automatic failover
   - Three Sentinel nodes + one Redis master
   - Automatic promotion on failover
   - Monitoring via standard tools

3. **Redis Cluster**: For very high throughput scenarios
   - Distributed session storage
   - No single point of failure
   - Additional operational complexity

## References

- 📖 Setup Guide: `docs/REDIS_MULTI_INSTANCE.md`
- 💻 Implementation: `src/hooks/redis_session.rs`
- 🐳 Docker: `compose.yaml`
- 🔑 Session Management: `src/hooks/identity.rs`
