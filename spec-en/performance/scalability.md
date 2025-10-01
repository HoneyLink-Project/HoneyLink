# docs/performance/scalability.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines scalability strategy for HoneyLink™. Describes horizontal/vertical scaling, load balancing, auto-scaling, and performance testing without implementation code or C/C++ dependencies.

## Table of Contents
- [Scalability Goals](#scalability-goals)
- [Horizontal Scaling](#horizontal-scaling)
- [Vertical Scaling](#vertical-scaling)
- [Load Balancing Strategy](#load-balancing-strategy)
- [Auto-Scaling Policies](#auto-scaling-policies)
- [Database Scaling](#database-scaling)
- [Performance Testing for Scalability](#performance-testing-for-scalability)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Scalability Goals
- Support 10k → 100k → 1M devices without architectural redesign.
- Maintain SLA (P99 latency < 120ms) under 10x load increase.
- Auto-scale within 2 minutes on demand spike.
- Cost-efficient scaling: Linear cost growth relative to load.

## Horizontal Scaling
- **Stateless Services:** All control-plane and data-plane services stateless. Session state externalized to Redis/database.
- **Kubernetes HPA (Horizontal Pod Autoscaler):**
  - Scale pods based on CPU (70%) or custom metrics (RPS, queue depth).
  - Min replicas: 3 (for HA), Max replicas: 100 (configurable per service).
- **Shard-Nothing Architecture:** Each pod handles full request set. No data partitioning required at application layer.
- **Benefits:** Rapid scale-out, no single point of failure, simplifies deployment.

## Vertical Scaling
- **Use Case:** Database, high-memory analytics services.
- **Database:** Upgrade RDS instance class (e.g., db.r5.2xlarge → db.r5.4xlarge).
- **Kubernetes:** Adjust pod resource requests/limits. Requires pod restart.
- **Limitations:** Upper bound on instance size. Vertical scaling alone insufficient for 10x growth.

## Load Balancing Strategy
- **Application Load Balancer (ALB):**
  - TLS termination at ALB. mTLS between ALB and backend pods.
  - Health checks: HTTP `/health` endpoint. Unhealthy pods removed from pool.
- **Service Mesh (Istio/Linkerd):**
  - Advanced traffic management: Circuit breaking, retry budgets, rate limiting.
  - Weighted routing for canary releases.
- **Global Load Balancing:**
  - CloudFront + Route 53 for geo-distributed traffic.
  - Latency-based routing: Direct users to nearest region.

## Auto-Scaling Policies
| Service | Metric | Scale-Out Threshold | Scale-In Threshold | Cooldown |
|---------|--------|---------------------|-------------------|----------|
| Control-Plane API | CPU | 70% | 30% | 5 min |
| Data-Plane Broker | Custom (Queue Depth) | >1000 msgs | <100 msgs | 10 min |
| Telemetry Collector | RPS | >5000 req/s | <1000 req/s | 5 min |
| Database Read Replicas | Connections | >80% max | <20% max | 15 min |

- **Predictive Scaling:** Leverage historical patterns (e.g., scale up before known peak hours).
- **Spot Instances:** Use for non-critical batch processing (cost savings up to 90%).

## Database Scaling
### Read Replicas
- Create 2-5 read replicas for read-heavy workloads (telemetry queries, dashboards).
- Route read queries to replicas. Write queries to primary.
- Replication lag monitored. Alert if lag >10s.

### Connection Pooling
- Use PgBouncer for connection pooling. Reduce connection overhead.
- Pool size tuned based on max_connections and expected concurrency.

### Query Optimization
- Identify slow queries via `pg_stat_statements`.
- Add indexes on frequently queried columns.
- Partition large tables (e.g., telemetry data by time range).

### Sharding (Future)
- If single database insufficient, implement horizontal sharding.
- Shard key: Device ID or Tenant ID.
- Coordinated via middleware layer (e.g., Vitess for PostgreSQL).

## Performance Testing for Scalability
- **Load Ramp Test:** Gradually increase load from 1x → 5x → 10x over 2 hours. Measure latency degradation.
- **Spike Test:** Sudden traffic spike (10x for 5 min). Verify auto-scaling responds within 2 min.
- **Soak Test:** Sustained load (5x baseline) for 24 hours. Detect memory leaks, connection exhaustion.
- **Stress Test:** Push system beyond max capacity. Identify breaking point and graceful degradation behavior.

- Test procedures detailed in [docs/performance/benchmark.md](benchmark.md).

## Acceptance Criteria (DoD)
- Scalability goals and strategies (horizontal, vertical) documented.
- Load balancing and auto-scaling policies defined.
- Database scaling approaches (read replicas, pooling, sharding) specified.
- Performance testing scenarios for scalability described.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
