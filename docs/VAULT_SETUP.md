# HashiCorp Vault Setup for HoneyLink™

**Status:** Development Guide  
**Last Updated:** 2025-10-01  
**Target Audience:** Backend Engineers, Security Engineers, DevOps

---

## Table of Contents

- [Overview](#overview)
- [Key Hierarchy](#key-hierarchy)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Quick Start (Development Mode)](#quick-start-development-mode)
  - [Manual Installation](#manual-installation)
- [Production Setup](#production-setup)
  - [Architecture](#architecture)
  - [High Availability Configuration](#high-availability-configuration)
  - [HSM Integration](#hsm-integration)
- [Rust Integration](#rust-integration)
  - [Client Configuration](#client-configuration)
  - [Key Operations](#key-operations)
  - [Error Handling](#error-handling)
- [Migration Path](#migration-path)
- [Troubleshooting](#troubleshooting)
- [Security Best Practices](#security-best-practices)
- [Related Documentation](#related-documentation)

---

## Overview

HoneyLink™ uses HashiCorp Vault for secure key management, implementing a hierarchical key structure as defined in `spec/security/encryption.md`. This document provides setup instructions for both development and production environments.

**Why Vault?**
- ✅ **Pure Rust client** (`vaultrs` crate) - no C/C++ dependencies
- ✅ **Hardware Security Module (HSM) support** for production
- ✅ **Automatic key rotation** with grace periods
- ✅ **Audit logging** for compliance
- ✅ **High availability** with clustering
- ✅ **Multi-cloud and on-premises** deployment options

---

## Key Hierarchy

HoneyLink's key hierarchy maps to Vault paths as follows:

| Scope | Vault Path | Purpose | Rotation Interval |
|-------|------------|---------|-------------------|
| `k_root` | `honeylink/k_root/*` | Root trust anchor (HSM-backed) | 5 years |
| `k_service` | `honeylink/k_service/*` | Service-level master keys | 1 year |
| `k_profile` | `honeylink/k_profile/*` | Profile data encryption keys | 90 days |
| `k_telemetry` | `honeylink/k_telemetry/*` | Telemetry data encryption keys | 90 days |

**Access Control:**
- `k_root`: Security team only (offline HSM access)
- `k_service`: Session Orchestrator, Policy Engine
- `k_profile`: Policy Engine
- `k_telemetry`: Telemetry & Insights module

---

## Development Setup

### Prerequisites

**Required:**
- HashiCorp Vault CLI (v1.15.0 or later)
- PowerShell 5.1+ (Windows) or bash (WSL/Linux)
- Network access to `127.0.0.1:8200` (default dev port)

**Optional:**
- Docker (for containerized Vault)
- jq (for JSON processing)

### Quick Start (Development Mode)

#### Windows (PowerShell)

```powershell
# 1. Run the setup script
.\scripts\setup-vault-dev.ps1

# 2. Set environment variables
$env:VAULT_ADDR = "http://127.0.0.1:8200"
$env:VAULT_TOKEN = "root"

# 3. Verify connection
vault status

# 4. View key hierarchy
vault kv list honeylink
```

#### WSL/Linux (bash)

```bash
# 1. Make script executable
chmod +x scripts/setup-vault-dev.sh

# 2. Run setup
./scripts/setup-vault-dev.sh

# 3. Source environment
source .vault-dev.env

# 4. Verify connection
vault status

# 5. View key hierarchy
vault kv list honeylink
```

**Script Features:**
- ✅ Automatic Vault binary download (if not installed)
- ✅ Dev server with root token `root`
- ✅ KV v2 secrets engine at `honeylink/` mount
- ✅ Pre-configured key hierarchy paths
- ✅ Development policy (`honeylink-dev`)
- ✅ Environment variable file (`.vault-dev.env`)

### Manual Installation

If you prefer to install Vault manually:

**Windows (Chocolatey):**
```powershell
choco install vault
```

**macOS (Homebrew):**
```bash
brew install vault
```

**Linux (binary):**
```bash
wget https://releases.hashicorp.com/vault/1.15.0/vault_1.15.0_linux_amd64.zip
unzip vault_1.15.0_linux_amd64.zip
sudo mv vault /usr/local/bin/
```

**Manual Dev Server Start:**
```bash
vault server -dev -dev-root-token-id=root -dev-listen-address=127.0.0.1:8200
```

---

## Production Setup

> ⚠️ **WARNING:** Never use `-dev` mode in production! Dev mode stores data in memory and uses insecure defaults.

### Architecture

**Recommended Production Setup:**
- **3-5 Vault nodes** in HA cluster (Raft consensus)
- **HSM integration** for root key protection (FIPS 140-3 Level 3)
- **TLS 1.3 mutual authentication** for all connections
- **Separate namespaces** per environment (dev/staging/prod)
- **Geographic distribution** for disaster recovery

**Infrastructure Options:**
1. **On-Premises:** Bare metal with HSM appliances
2. **Cloud-Agnostic:** Kubernetes with Vault Operator
3. **Hybrid:** On-prem HSM + cloud Vault cluster

### High Availability Configuration

**Example `config.hcl` (3-node Raft cluster):**

```hcl
# Common configuration for all nodes
storage "raft" {
  path    = "/vault/data"
  node_id = "vault-node-1"  # Change per node
  
  retry_join {
    leader_api_addr = "https://vault-node-1.internal:8200"
  }
  retry_join {
    leader_api_addr = "https://vault-node-2.internal:8200"
  }
  retry_join {
    leader_api_addr = "https://vault-node-3.internal:8200"
  }
}

listener "tcp" {
  address       = "0.0.0.0:8200"
  tls_cert_file = "/vault/tls/vault.crt"
  tls_key_file  = "/vault/tls/vault.key"
  tls_min_version = "tls13"
  tls_require_and_verify_client_cert = true
  tls_client_ca_file = "/vault/tls/ca.crt"
}

seal "pkcs11" {
  lib            = "/usr/lib/libCryptoki2_64.so"
  slot           = "0"
  pin            = "env://HSM_PIN"
  key_label      = "vault-hsm-key"
  hmac_key_label = "vault-hmac-key"
}

api_addr = "https://vault-node-1.internal:8200"  # Change per node
cluster_addr = "https://vault-node-1.internal:8201"  # Change per node

ui = true

telemetry {
  prometheus_retention_time = "30s"
  disable_hostname = true
}

log_level = "info"
```

**Initialization:**

```bash
# Initialize (run once on any node)
vault operator init -key-shares=5 -key-threshold=3

# Unseal (run on all nodes with 3 different keys)
vault operator unseal <key-1>
vault operator unseal <key-2>
vault operator unseal <key-3>

# Enable audit logging
vault audit enable file file_path=/vault/logs/audit.log

# Mount KV v2 for HoneyLink
vault secrets enable -path=honeylink kv-v2
```

### HSM Integration

**Supported HSMs:**
- Thales Luna Network HSM
- Entrust nShield
- AWS CloudHSM
- YubiHSM 2 (development/testing)

**PKCS#11 Configuration:**

```hcl
seal "pkcs11" {
  lib            = "/path/to/pkcs11.so"
  slot           = "0"
  pin            = "env://HSM_PIN"      # Read from secure env var
  key_label      = "honeylink-root"
  hmac_key_label = "honeylink-hmac"
  generate_key   = true                 # Auto-generate on first boot
}
```

**Best Practices:**
- ✅ Use separate HSM slots per environment
- ✅ Implement 4-eyes principle for HSM PIN access
- ✅ Rotate HSM keys according to `spec/security/key-management.md`
- ✅ Maintain offline HSM backups in geographically separate locations

---

## Rust Integration

### Client Configuration

**Cargo.toml:**

```toml
[dependencies]
honeylink-crypto = { path = "../crates/crypto", features = ["vault"] }
tokio = { version = "1.0", features = ["full"] }
```

**Environment Variables:**

```bash
export VAULT_ADDR="https://vault.honeylink.internal:8200"
export VAULT_TOKEN="<service-token>"  # Short-lived, 5 min TTL
export VAULT_NAMESPACE="honeylink"    # Enterprise feature
export HONEYLINK_ENV="production"
```

### Key Operations

**Initialize Client:**

```rust
use honeylink_crypto::vault::{VaultClient, KeyScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // From environment variables
    let client = VaultClient::from_env()?;
    
    // Or explicit configuration
    let client = VaultClient::new(
        "https://vault.example.com:8200",
        "s.abc123...",
        Some("honeylink"),  // namespace
        "honeylink",        // mount point
        "production",       // environment
    )?;
    
    // Health check
    client.health_check().await?;
    
    Ok(())
}
```

**Store a Key:**

```rust
use std::time::Duration;

// Generate key (example: X25519)
let key_data = vec![0u8; 32];  // Replace with actual key gen

// Store with 90-day TTL
client.store_key(
    KeyScope::Service,
    "session-orchestrator-v1",
    key_data,
    Duration::from_secs(90 * 24 * 3600),
).await?;
```

**Retrieve a Key:**

```rust
let key_material = client.retrieve_key(
    KeyScope::Service,
    "session-orchestrator-v1",
).await?;

println!("Key version: {}", key_material.metadata.version);
println!("Created: {}", key_material.metadata.created_at);
println!("Expires: {}", key_material.metadata.expires_at);

// Key data is automatically zeroized on drop
let sensitive_data = &key_material.data;
```

**Rotate a Key:**

```rust
let new_key_data = vec![0u8; 32];  // New key material

let new_version = client.rotate_key(
    KeyScope::Service,
    "session-orchestrator-v1",
    new_key_data,
).await?;

println!("Rotated to version: {}", new_version);
```

**List Keys in Scope:**

```rust
let keys = client.list_keys(KeyScope::Profile).await?;
for key in keys {
    println!("Profile key: {}", key);
}
```

### Error Handling

```rust
use honeylink_crypto::vault::VaultError;

match client.retrieve_key(KeyScope::Service, "missing-key").await {
    Ok(material) => {
        // Use key
    }
    Err(VaultError::KeyNotFound { scope, name }) => {
        eprintln!("Key not found: {:?}/{}", scope, name);
    }
    Err(VaultError::KeyExpired { scope, name }) => {
        eprintln!("Key expired: {:?}/{}", scope, name);
        // Trigger rotation
    }
    Err(VaultError::Connection(msg)) => {
        eprintln!("Vault connection failed: {}", msg);
        // Fall back to local encrypted storage
    }
    Err(e) => {
        eprintln!("Vault error: {}", e);
    }
}
```

---

## Migration Path

### From Development to Production

**Phase 1: Staging Environment (Week 1-2)**

1. Deploy 3-node Vault cluster in staging
2. Enable TLS with internal CA
3. Configure OIDC authentication (no root tokens)
4. Migrate dev keys to staging (re-encrypt)
5. Run integration tests with production config

**Phase 2: Production Deployment (Week 3-4)**

1. Provision HSM and initialize Vault seal
2. Generate root keys in offline ceremony (4-eyes)
3. Configure mTLS for service-to-service auth
4. Deploy monitoring (Prometheus + Grafana)
5. Enable audit logging to WORM storage
6. Gradual traffic migration (canary deployment)

**Phase 3: Operational Hardening (Week 5+)**

1. Implement automated key rotation
2. Set up disaster recovery (cross-region backup)
3. Conduct security audit and penetration testing
4. Document runbooks for incident response

### Data Migration Script

```bash
#!/bin/bash
# migrate-keys.sh - Migrate from dev to production Vault

DEV_ADDR="http://127.0.0.1:8200"
DEV_TOKEN="root"

PROD_ADDR="https://vault.honeylink.internal:8200"
PROD_TOKEN="<service-token>"

# Export from dev
export VAULT_ADDR=$DEV_ADDR
export VAULT_TOKEN=$DEV_TOKEN

vault kv get -format=json honeylink/k_service/session-orchestrator > /tmp/key.json

# Import to production
export VAULT_ADDR=$PROD_ADDR
export VAULT_TOKEN=$PROD_TOKEN

vault kv put honeylink/k_service/session-orchestrator @/tmp/key.json

# Secure delete temporary file
shred -vfz -n 10 /tmp/key.json
```

---

## Troubleshooting

### Common Issues

#### 1. Connection Refused

**Symptom:**
```
Error: connection error: Connection refused (os error 111)
```

**Solutions:**
- ✅ Check Vault is running: `ps aux | grep vault`
- ✅ Verify `VAULT_ADDR` is set correctly
- ✅ Check firewall rules: `sudo ufw status`
- ✅ Review Vault logs: `tail -f data/vault-dev/vault.log`

#### 2. Permission Denied

**Symptom:**
```
Error: permission denied: insufficient capabilities
```

**Solutions:**
- ✅ Verify token has correct policy attached
- ✅ Check policy allows operation: `vault policy read honeylink-dev`
- ✅ Ensure namespace is correct (Enterprise)
- ✅ Renew token: `vault token renew`

#### 3. Key Expired

**Symptom:**
```
Error: key expired: scope=Service, name=session-orchestrator
```

**Solutions:**
- ✅ Trigger key rotation immediately
- ✅ Check rotation policy: `spec/security/key-management.md`
- ✅ Review grace period configuration
- ✅ Update monitoring alerts for approaching expiry

#### 4. Seal Status Sealed

**Symptom:**
```
Error: Vault is sealed
```

**Solutions:**
```bash
# Check status
vault status

# Unseal (requires 3 keys)
vault operator unseal <key-1>
vault operator unseal <key-2>
vault operator unseal <key-3>

# Auto-unseal with HSM (production)
# Configure seal "pkcs11" in config.hcl
```

### Debug Mode

**Enable verbose logging:**

```bash
export VAULT_LOG_LEVEL=debug
export RUST_LOG=honeylink_crypto::vault=trace

# Run application
cargo run --features vault
```

**Vault server debug logs:**

```bash
vault server -dev -log-level=debug
```

---

## Security Best Practices

### Development

- ✅ Use separate Vault instance for dev (never share with prod)
- ✅ Rotate dev root token weekly
- ✅ Never commit `.vault-dev.env` to version control
- ✅ Use short-lived tokens (5 min TTL)
- ✅ Enable audit logging even in dev

### Production

- ✅ **Seal with HSM** (never auto-unseal with cloud KMS in sensitive environments)
- ✅ **mTLS everywhere** (client certificates for all services)
- ✅ **No root tokens** (use OIDC/LDAP for human access)
- ✅ **Principle of least privilege** (narrow policies per service)
- ✅ **Audit all access** (SIEM integration for anomaly detection)
- ✅ **Geographic redundancy** (multi-region with async replication)
- ✅ **Regular key rotation** (automated with grace periods)
- ✅ **Offline root key backups** (4-eyes principle, air-gapped storage)

### Compliance

- ✅ **FIPS 140-3 Level 3** HSM for root keys
- ✅ **SOC 2 Type II** audit trails
- ✅ **GDPR compliance** (data residency, right to erasure)
- ✅ **NIST SP 800-56A** key agreement
- ✅ **NIST SP 800-88** secure key deletion

---

## Related Documentation

- [spec/security/encryption.md](../spec/security/encryption.md) - Cryptographic standards
- [spec/security/key-management.md](../spec/security/key-management.md) - Key lifecycle
- [spec/security/auth.md](../spec/security/auth.md) - Authentication flows
- [spec/deployment/infrastructure.md](../spec/deployment/infrastructure.md) - Production deployment
- [docs/RUST_SETUP.md](./RUST_SETUP.md) - Rust development environment

---

**Feedback & Questions:**  
For issues or improvements to this guide, please open an issue or submit a PR.

**Security Concerns:**  
Report security vulnerabilities privately via [security@honeylink.example.com](mailto:security@honeylink.example.com).
