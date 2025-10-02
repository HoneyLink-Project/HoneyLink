-- Migration: sessions table and related structures
-- Purpose: Store active sessions with multi-stream configuration and TTL management
-- Created: 2025-10-02 for Task 3.3

-- Session status enum
CREATE TYPE session_status AS ENUM ('active', 'expired', 'terminated');

-- Sessions table
-- Stores session metadata, stream configuration, and key material
CREATE TABLE sessions (
    -- Primary key: UUIDv7 (time-ordered)
    session_id UUID PRIMARY KEY,

    -- Foreign key to devices table
    device_id VARCHAR(64) NOT NULL,

    -- Stream configuration (JSONB array)
    -- Format: [{"stream_id": "...", "name": "...", "mode": "reliable|unreliable", "qos": {...}, "fec": {...}}]
    streams JSONB NOT NULL,

    -- Session key material (encrypted at rest via DB encryption)
    -- Stores derived session key for key derivation hierarchy
    key_material BYTEA NOT NULL,

    -- Session expiration timestamp (TTL management)
    expires_at TIMESTAMPTZ NOT NULL,

    -- Session status
    status session_status NOT NULL DEFAULT 'active',

    -- Session endpoint URL (for client connection)
    endpoint VARCHAR(255) NOT NULL,

    -- Audit trail timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    terminated_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT fk_device FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE,
    CONSTRAINT expires_at_future CHECK (expires_at > created_at),
    CONSTRAINT terminated_at_after_created CHECK (terminated_at IS NULL OR terminated_at > created_at)
);

-- Indexes for query performance
CREATE INDEX idx_sessions_device_id ON sessions(device_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);

-- Composite index for active sessions by device
CREATE INDEX idx_sessions_device_active ON sessions(device_id, status) WHERE status = 'active';

-- GIN index for JSONB stream queries
CREATE INDEX idx_sessions_streams ON sessions USING GIN(streams);

-- Auto-update trigger for session status based on expiration
-- This function automatically marks sessions as expired when queried
CREATE OR REPLACE FUNCTION auto_expire_sessions()
RETURNS TRIGGER AS $$
BEGIN
    -- If session is active but expired, mark as expired
    IF NEW.status = 'active' AND NEW.expires_at < NOW() THEN
        NEW.status = 'expired';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_auto_expire_sessions
BEFORE UPDATE ON sessions
FOR EACH ROW
EXECUTE FUNCTION auto_expire_sessions();

-- Function to cleanup expired sessions (for periodic background job)
-- Returns count of deleted sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions(retention_hours INT DEFAULT 24)
RETURNS INT AS $$
DECLARE
    deleted_count INT;
BEGIN
    -- Delete expired sessions older than retention period
    DELETE FROM sessions
    WHERE status = 'expired'
      AND expires_at < NOW() - (retention_hours || ' hours')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE sessions IS 'Active and historical sessions with multi-stream configuration';
COMMENT ON COLUMN sessions.session_id IS 'UUIDv7 session identifier (time-ordered)';
COMMENT ON COLUMN sessions.device_id IS 'Device identifier (FK to devices table)';
COMMENT ON COLUMN sessions.streams IS 'JSONB array of stream configurations with FEC parameters';
COMMENT ON COLUMN sessions.key_material IS 'Encrypted session key material for HKDF derivation';
COMMENT ON COLUMN sessions.expires_at IS 'Session expiration timestamp (TTL)';
COMMENT ON COLUMN sessions.status IS 'Session lifecycle status (active, expired, terminated)';
COMMENT ON COLUMN sessions.endpoint IS 'Session endpoint URL for client connection (e.g., quic://...)';
COMMENT ON FUNCTION cleanup_expired_sessions IS 'Periodic cleanup function for expired sessions (default: 24h retention)';
