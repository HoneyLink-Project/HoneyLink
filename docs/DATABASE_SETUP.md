# HoneyLink Database Setup Guide

Complete guide for setting up TimescaleDB development database and running migrations.

## Prerequisites

- **Docker & Docker Compose** installed
- **Rust toolchain** 1.89.0+ with `cargo` and `sqlx-cli`
- **WSL** (Windows users) or native Linux/macOS environment

## Quick Start

### 1. Install sqlx-cli

```bash
# Install sqlx-cli with PostgreSQL support
cargo install sqlx-cli --no-default-features --features postgres
```

### 2. Start TimescaleDB

```bash
# Start TimescaleDB container
cd infrastructure
docker-compose -f docker-compose.db.yml up -d

# Wait for database to be ready (health check)
docker-compose -f docker-compose.db.yml ps
# Expected: honeylink-timescaledb (healthy)

# Verify TimescaleDB extension is loaded
docker exec honeylink-timescaledb psql -U honeylink -d honeylink_dev -c "SELECT extname, extversion FROM pg_extension WHERE extname = 'timescaledb';"
```

### 3. Set Environment Variable

**Linux/macOS/WSL:**
```bash
export DATABASE_URL=postgres://honeylink:dev_password@localhost:5432/honeylink_dev
```

**Windows PowerShell:**
```powershell
$env:DATABASE_URL = "postgres://honeylink:dev_password@localhost:5432/honeylink_dev"
```

**Or use .env file (recommended):**
```bash
# Copy .env.example to .env (already done if you have .env)
cp .env.example .env

# Verify DATABASE_URL is set
cat .env | grep DATABASE_URL
```

### 4. Run Migrations

```bash
# Navigate to backend directory
cd backend

# Run pending migrations
sqlx migrate run

# Expected output:
# Applied 20250101000001/migrate initial schema (XXX.XXXs)
# Applied 20250102000001/migrate sessions schema (XXX.XXXs)
```

### 5. Generate sqlx Query Cache

```bash
# Still in backend/ directory
# This generates .sqlx/ directory with offline query metadata
cargo sqlx prepare

# Expected output:
# query data written to `.sqlx` in the current directory
# please check this into version control
```

### 6. Verify Backend Build

**WSL (recommended):**
```bash
cd /mnt/c/Users/Aqua/Programming/HoneyLink
source ~/.cargo/env
RUSTUP_TOOLCHAIN=1.89.0 cargo build -p honeylink-control-plane --target x86_64-unknown-linux-gnu
```

**Windows PowerShell (if WSL unavailable):**
```powershell
cd C:\Users\Aqua\Programming\HoneyLink
cargo build -p honeylink-control-plane
```

Expected: **0 errors** (warnings OK)

## Database Schema Overview

### Tables Created by Migrations

**20250101000001_initial_schema.sql:**
- `devices` - Device registration and attestation data
- `pairing_codes` - Temporary pairing codes for device provisioning
- `audit_events` - Audit log for all system events

**20250102000001_sessions_schema.sql:**
- `sessions` - Active session state and metadata

### TimescaleDB Hypertables

Sessions table is converted to a TimescaleDB hypertable for time-series optimization:
```sql
SELECT create_hypertable('sessions', 'created_at', if_not_exists => TRUE);
```

This enables:
- Efficient time-based queries
- Automatic data retention policies (future)
- Compression for historical data (future)

## Maintenance

### Reset Database (Clean Slate)

```bash
# Stop and remove containers + volumes
docker-compose -f infrastructure/docker-compose.db.yml down -v

# Start fresh
docker-compose -f infrastructure/docker-compose.db.yml up -d

# Re-run migrations
cd backend
sqlx migrate run
cargo sqlx prepare
```

### View Applied Migrations

```bash
# Check migration status
sqlx migrate info

# Or query _sqlx_migrations table directly
docker exec honeylink-timescaledb psql -U honeylink -d honeylink_dev -c "SELECT * FROM _sqlx_migrations ORDER BY installed_on;"
```

### Inspect Database

```bash
# Connect to PostgreSQL CLI
docker exec -it honeylink-timescaledb psql -U honeylink -d honeylink_dev

# List tables
\dt

# Describe table schema
\d devices
\d sessions

# Query data
SELECT * FROM devices LIMIT 10;

# Exit
\q
```

### Backup & Restore

**Backup:**
```bash
docker exec honeylink-timescaledb pg_dump -U honeylink honeylink_dev > backup.sql
```

**Restore:**
```bash
cat backup.sql | docker exec -i honeylink-timescaledb psql -U honeylink -d honeylink_dev
```

## Troubleshooting

### Error: "role 'honeylink' does not exist"

**Cause:** Database container not fully initialized.

**Solution:**
```bash
docker-compose -f infrastructure/docker-compose.db.yml logs timescaledb
# Wait for "database system is ready to accept connections"

docker-compose -f infrastructure/docker-compose.db.yml restart timescaledb
```

### Error: "sqlx::query_as! failed: no cached query data"

**Cause:** `cargo sqlx prepare` not run or .sqlx/ directory missing.

**Solution:**
```bash
cd backend
cargo sqlx prepare --check  # Verify cache is up-to-date
cargo sqlx prepare          # Regenerate cache if needed
```

### Error: "Connection refused (port 5432)"

**Cause:** TimescaleDB container not running or port conflict.

**Solution:**
```bash
# Check container status
docker ps | grep honeylink-timescaledb

# Check port 5432 availability
netstat -an | grep 5432  # Linux/macOS
Get-NetTCPConnection -LocalPort 5432  # Windows PowerShell

# If port conflict, edit docker-compose.db.yml:
# ports: "5433:5432"  # Map to different host port
# Then update DATABASE_URL: localhost:5433
```

### Error: "migration checksum mismatch"

**Cause:** Migration files modified after being applied.

**Solution:**
```bash
# Reset database and re-run migrations
docker-compose -f infrastructure/docker-compose.db.yml down -v
docker-compose -f infrastructure/docker-compose.db.yml up -d
cd backend
sqlx migrate run
```

## Production Deployment

### Security Checklist

- [ ] **Change Default Credentials:** Never use `dev_password` in production
- [ ] **Enable SSL/TLS:** Use `sslmode=require` in DATABASE_URL
- [ ] **Rotate Credentials:** Use Vault dynamic secrets or AWS Secrets Manager
- [ ] **Firewall:** Restrict port 5432 to application servers only (no public access)
- [ ] **Audit Logging:** Enable PostgreSQL audit logging for compliance
- [ ] **Backups:** Automated daily backups with retention policy

### Managed TimescaleDB Options

- **Timescale Cloud:** Fully managed TimescaleDB (recommended)
- **AWS RDS:** PostgreSQL with TimescaleDB extension
- **Google Cloud SQL:** PostgreSQL with TimescaleDB support
- **Azure Database for PostgreSQL:** Flexible Server with TimescaleDB

### Migration Best Practices

1. **Test Migrations:** Run on staging environment first
2. **Backup Before Migrate:** Always take backup before running migrations
3. **Rollback Plan:** Create down migrations for reversibility
4. **Zero Downtime:** Use online schema changes (e.g., pg_repack) for large tables
5. **Monitor Performance:** Check query execution plans after schema changes

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Database Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: timescale/timescaledb:2.14.2-pg16
        env:
          POSTGRES_USER: honeylink
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: honeylink_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - name: Install sqlx-cli
        run: cargo install sqlx-cli --no-default-features --features postgres
      - name: Run migrations
        env:
          DATABASE_URL: postgres://honeylink:test_password@localhost:5432/honeylink_test
        run: |
          cd backend
          sqlx migrate run
      - name: Build backend
        env:
          DATABASE_URL: postgres://honeylink:test_password@localhost:5432/honeylink_test
        run: cargo build -p honeylink-control-plane
```

## References

- [TimescaleDB Documentation](https://docs.timescale.com/)
- [sqlx Documentation](https://github.com/launchbadge/sqlx)
- [PostgreSQL Official Docs](https://www.postgresql.org/docs/)
- [Docker Compose Reference](https://docs.docker.com/compose/)
