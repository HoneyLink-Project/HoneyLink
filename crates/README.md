# HoneyLink P2P Crates

**Architecture:** Pure P2P (Peer-to-Peer), no central server, local-first

This directory contains the core Rust crates for HoneyLink, a "Bluetooth superset" P2P communication protocol.

## ⚠️ Important: P2P Design

**All crates are designed for local-first, peer-to-peer operation:**
- ❌ **No central server** (no Control Plane, no backend API)
- ❌ **No cloud databases** (no PostgreSQL, no Redis, no TimescaleDB)
- ❌ **No external IdP** (no OAuth2, no JWT, no Auth0)
- ✅ **Local storage only** (`~/.honeylink/` SQLite, JSON files)
- ✅ **P2P protocols** (mDNS, BLE, QUIC, WebRTC)
- ✅ **Pure Rust** (no C/C++ dependencies)

## Crates Overview

### Core Protocol
- **`core/`** - Shared types and traits
- **`session-orchestrator/`** - P2P session management (TOFU trust model)
- **`transport/`** - QUIC/WebRTC abstraction with FEC (Reed-Solomon)
- **`physical-adapter/`** - mDNS/BLE discovery, NAT traversal (STUN/TURN)

### Security
- **`crypto/`** - X25519 ECDH, ChaCha20-Poly1305, HKDF-SHA512, Ed25519
  - `vault` feature: **Optional** HashiCorp Vault integration (disabled by default)
  - Default: Local file storage with OS Keychain protection

### QoS & Policy
- **`qos-scheduler/`** - Weighted Fair Queuing (WFQ) with 3-tier bandwidth allocation
- **`policy-engine/`** - Local QoS policy management (SQLite)
- **`experience/`** - SDK API and UI shell

### Observability
- **`telemetry/`** - OpenTelemetry integration
  - **Default:** Local SQLite (`~/.honeylink/metrics/metrics.db`)
  - **Optional:** OTLP export (user opt-in, disabled by default)
  - **Removed:** PagerDuty/Slack alerting (replaced with OS notifications)

## Optional Server Features

Some crates have **optional features** for server-side deployments (e.g., enterprise managed infrastructure). These are **disabled by default** and NOT required for P2P operation:

```toml
[features]
default = ["p2p"]  # Pure P2P mode (no servers)

# Optional server features (not recommended for typical use)
vault = ["dep:vaultrs"]  # HashiCorp Vault integration
otlp-export = ["dep:opentelemetry-otlp"]  # OTLP Collector export
```

**For typical P2P use cases, do NOT enable these features.**

## Architecture Consistency

All crates follow these P2P design principles:

1. **Local-first:** Data stored in `~/.honeylink/` (SQLite, JSON, PEM files)
2. **No network dependencies:** Works offline after initial pairing
3. **TOFU trust:** Trust established on first pairing (like Bluetooth)
4. **No authentication servers:** Public key exchange via QR/PIN/NFC
5. **Optional telemetry:** Local metrics only, OTLP export requires user consent

## Testing

All crates have unit tests that verify P2P operation without servers:

```bash
# Run all tests (no server dependencies)
cargo test --all-features

# Run P2P-only tests (default)
cargo test
```

## Related Documentation

- [spec/architecture/overview.md](../spec/architecture/overview.md) - P2P architecture
- [spec/modules/](../spec/modules/) - Module specifications (all P2P)
- [spec/security/auth.md](../spec/security/auth.md) - TOFU trust model
- [spec/deployment/infrastructure.md](../spec/deployment/infrastructure.md) - Client distribution

## Migration from Server-Centric Design

**Historical Note:** Earlier versions of HoneyLink had server-centric code (Control Plane API, CockroachDB, Vault, TimescaleDB). This was removed in October 2025 to focus on pure P2P "Bluetooth superset" design.

**If you encounter server references in code:**
- Check if feature-gated (`#[cfg(feature = "vault")]`) - these are optional
- Report as bug if hard-coded server dependencies exist in default features
- See backups: `spec/*-old-server.md` for historical context

## License

MIT License - See [LICENSE](../LICENSE)
