#!/bin/bash
# TimescaleDB initialization script for HoneyLink development database
# This script runs automatically on first container startup via docker-entrypoint-initdb.d
#
# Executed as: postgres user
# Database: honeylink_dev (already created by POSTGRES_DB env var)

set -e

# Enable TimescaleDB extension
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Enable TimescaleDB extension
    CREATE EXTENSION IF NOT EXISTS timescaledb;

    -- Verify extension is enabled
    SELECT extname, extversion FROM pg_extension WHERE extname = 'timescaledb';

    -- Create UUID extension (used by HoneyLink schema)
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

    -- Enable PostgreSQL crypto functions (for future use)
    CREATE EXTENSION IF NOT EXISTS pgcrypto;

    -- Log successful initialization
    DO \$\$
    BEGIN
        RAISE NOTICE 'HoneyLink database initialized successfully';
        RAISE NOTICE 'TimescaleDB version: %', (SELECT extversion FROM pg_extension WHERE extname = 'timescaledb');
    END \$\$;
EOSQL

echo "TimescaleDB initialization completed for database: $POSTGRES_DB"
