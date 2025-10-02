-- Initial database schema for HoneyLink Control Plane
-- This migration creates tables for device management, pairing, and audit logs

-- devices table: Stores registered devices and their metadata
CREATE TABLE IF NOT EXISTS devices (
    device_id TEXT PRIMARY KEY,
    public_key BYTEA NOT NULL,
    firmware_version TEXT NOT NULL,
    capabilities TEXT[] NOT NULL DEFAULT '{}',
    attestation_format TEXT,
    attestation_evidence BYTEA,
    attestation_nonce TEXT,
    metadata JSONB,
    device_token TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'paired', 'revoked')) DEFAULT 'pending',
    certificate_serial TEXT,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paired_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_devices_status ON devices(status);
CREATE INDEX idx_devices_registered_at ON devices(registered_at);

-- pairing_codes table: Stores temporary pairing codes with TTL
CREATE TABLE IF NOT EXISTS pairing_codes (
    pairing_code TEXT PRIMARY KEY,
    device_id TEXT NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pairing_codes_device_id ON pairing_codes(device_id);
CREATE INDEX idx_pairing_codes_expires_at ON pairing_codes(expires_at);

-- audit_events table: Append-only audit log (WORM compliance)
CREATE TABLE IF NOT EXISTS audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    category TEXT NOT NULL,
    actor TEXT NOT NULL,
    device_id TEXT,
    outcome TEXT NOT NULL CHECK (outcome IN ('success', 'failure')),
    details JSONB,
    trace_id TEXT
);

CREATE INDEX idx_audit_events_timestamp ON audit_events(timestamp DESC);
CREATE INDEX idx_audit_events_device_id ON audit_events(device_id);
CREATE INDEX idx_audit_events_category ON audit_events(category);
CREATE INDEX idx_audit_events_trace_id ON audit_events(trace_id);

-- Function to prevent audit log modifications (WORM compliance)
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit_events table is append-only (WORM compliance)';
END;
$$ LANGUAGE plpgsql;

-- Trigger to prevent updates on audit_events
CREATE TRIGGER prevent_audit_update
    BEFORE UPDATE ON audit_events
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

-- Trigger to prevent deletes on audit_events
CREATE TRIGGER prevent_audit_delete
    BEFORE DELETE ON audit_events
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at on devices table
CREATE TRIGGER update_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
