# Module Specification: Crypto & Trust Anchor

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Implementation specification for Crypto & Trust Anchor module. Responsible for end-to-end encryption and key management.

**Traceability ID**: `MOD-004-CRYPTO`

---

## 1. Module Overview

- **Module Name:** Crypto & Trust Anchor
- **Responsible Team:** Security WG (ENG-SEC-01, ENG-SEC-02)
- **Overview:** Encryption, key exchange, and key hierarchy management using X25519/ChaCha20-Poly1305/HKDF-SHA512
- **Status:** Implementation in progress (P1 phase)
- **Repository Path:** `crates/crypto/`

### Value Proposition
- Preparation for Post-Quantum Cryptography (PQC) with quantum computer resistance (Kyber integration planned)
- No C/C++ dependencies (RustCrypto suite pure implementation)
- Root Key protection through HSM integration
- 90-day automatic key rotation

---

## 2. Responsibilities and Boundaries

### Primary Responsibilities
- **Key Exchange**: X25519 ECDH (Elliptic Curve Diffie-Hellman)
- **Encryption/Decryption**: ChaCha20-Poly1305 AEAD (Authenticated Encryption with Associated Data)
- **Key Derivation**: HKDF-SHA512 (HMAC-based Extract-and-Expand Key Derivation Function)
- **Key Hierarchy Management**: Root Key → Session Key → Stream Key
- **Digital Signature**: Ed25519 (profile signature, Audit Log signature)
- **Key Rotation**: 90-day cycle automatic update

### Non-Responsibilities
- **Key Distribution**: Delegated to Session Orchestrator
- **Access Control**: Delegated to Policy Engine
- **Audit Log Storage**: Delegated to Telemetry
- **HSM Operations**: Managed by Infrastructure Team

### Related Documents
- [spec/security/encryption.md](../security/encryption.md)
- [spec/security/key-management.md](../security/key-management.md)
- [spec/requirements.md](../requirements.md) - FR-02 (Authentication), NFR-02 (Encryption)

---

## 3. Interface

### 3.1 Input

| Name | Protocol/Format | Validation Rules | Source |
|------|----------------|------------------|--------|
| **KeyExchangeRequest** | gRPC (Protobuf) | public_key: 32 bytes | Session Orchestrator |
| **EncryptRequest** | Sync API (Rust) | plaintext.len() <= 1MB | Transport |
| **SignRequest** | Sync API (Rust) | data: Bytes | Policy Engine |

### 3.2 Output

| Name | Protocol/Format | SLA | Destination |
|------|----------------|-----|-------------|
| **SharedSecret** | Sync Response (32 bytes) | P95 < 10ms | Session Orchestrator |
| **EncryptedPayload** | CBOR (ChaCha20-Poly1305) | P95 < 20ms | Transport |
| **Signature** | Bytes (64) | P99 < 50ms | Policy Engine |
| **KeyRotation** | Event Bus (JSON) | Async | Telemetry |

**EncryptedPayload Schema**:
```json
{
  "ciphertext": "base64...",
  "nonce": "base64(12 bytes)",
  "tag": "base64(16 bytes)",
  "key_id": "key_stream_xyz",
  "algorithm": "ChaCha20-Poly1305"
}
```

Details: [spec/architecture/interfaces.md](../architecture/interfaces.md)

---

## 4. Data Model

### 4.1 Key Hierarchy

```
Root Key (HSM storage)
  ↓ HKDF-SHA512
Device Master Key (90-day rotation)
  ↓ HKDF-SHA512
Session Key (generated at session establishment)
  ↓ HKDF-SHA512
Stream Key (per stream ID, 24-hour rotation)
```

#### KeyMaterial
```yaml
KeyMaterial:
  key_id: String(64)  # Primary Key, prefix: key_
  key_type: Enum[Root, DeviceMaster, Session, Stream]
  key_bytes: Bytes(32)  # For ChaCha20-Poly1305
  parent_key_id: String(64) (nullable)
  created_at: Timestamp
  expires_at: Timestamp
  rotated: Boolean
  hsm_backed: Boolean  # Only true for Root Key
```

#### KeyRotationLog
```yaml
KeyRotationLog:
  rotation_id: UUIDv7
  old_key_id: String(64)
  new_key_id: String(64)
  rotation_reason: Enum[Scheduled, Compromised, Manual]
  rotated_at: Timestamp
  rotated_by: String(128)  # User ID or "system"
```

### 4.2 Persistence
- **Data Store**: HashiCorp Vault (Root/Device Master Keys), Redis (Session/Stream Keys - with TTL)
- **Retention Period**: Root (indefinite), Device Master (90 days), Session (24h), Stream (24h)
- **Encryption/Confidentiality**: Use Vault Transit Secret Engine

Details: [spec/security/key-management.md](../security/key-management.md)

---

## 5. Algorithm

### 5.1 X25519 Key Exchange

```
Alice (Device A):
  private_key_a ← random(32 bytes)
  public_key_a ← X25519_base_point × private_key_a

Bob (Device B):
  private_key_b ← random(32 bytes)
  public_key_b ← X25519_base_point × private_key_b

Alice → Bob: public_key_a
Bob → Alice: public_key_b

Alice: shared_secret ← X25519(private_key_a, public_key_b)
Bob:   shared_secret ← X25519(private_key_b, public_key_a)

assert(Alice.shared_secret == Bob.shared_secret)
```

### 5.2 HKDF-SHA512 Key Derivation

```
PRK ← HKDF-Extract(salt=random(32), IKM=shared_secret)
OKM ← HKDF-Expand(PRK, info="HoneyLink-SessionKey-v1", L=32)

Session Key ← OKM
```

### 5.3 ChaCha20-Poly1305 Encryption

```
nonce ← random(12 bytes)  # Must not reuse within one session
ciphertext, tag ← ChaCha20-Poly1305-Encrypt(key, nonce, plaintext, aad="session_id")
```

**AAD (Additional Authenticated Data)**: Including `session_id` prevents ciphertext reuse across different sessions

Reference: [spec/security/encryption.md](../security/encryption.md)

---

## 6. Dependencies

| Type | Component | Interface | SLA/Contract |
|------|-----------|-----------|--------------|
| **Upper** | Session Orchestrator | KeyExchangeRequest | P95 < 50ms |
| **Upper** | Transport | EncryptRequest | P95 < 30ms |
| **Upper** | Policy Engine | SignRequest | P99 < 50ms |
| **Peer** | HashiCorp Vault | HTTP REST API | P99 < 100ms |
| **Peer** | Redis | Redis Protocol | P99 < 10ms |

**Dependency Rules**: [spec/architecture/dependencies.md](../architecture/dependencies.md)

---

## 7. Performance & Scalability

### SLO/SLI

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Key Exchange Latency (P95) | < 10ms | KeyExchangeRequest → SharedSecret |
| Encryption Latency (P95) | < 20ms | EncryptRequest → EncryptedPayload (1KB) |
| Signature Generation Latency (P99) | < 50ms | SignRequest → Signature |
| Throughput | ≥ 10K ops/sec/instance | Number of encryption operations |

Details: [spec/performance/benchmark.md](../performance/benchmark.md)

---

## 8. Security & Privacy

### Threat Countermeasures (STRIDE)
| Threat | Countermeasure |
|--------|----------------|
| **Spoofing** | X25519 key exchange + mTLS |
| **Tampering** | ChaCha20-Poly1305 AEAD tag verification |
| **Information Disclosure** | End-to-end encryption |
| **Denial of Service** | Rate limiting (10K ops/sec/instance) |
| **Elevation of Privilege** | Root Key protection via HSM |

### PQC Support Roadmap
- **Phase 1 (2025 Q4)**: Kyber768 integration (Post-Quantum Key Encapsulation)
- **Phase 2 (2026 Q2)**: Hybrid mode (X25519 + Kyber768)
- **Phase 3 (2027)**: Consider X25519 deprecation

Details: [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 9. Observability

### Metrics

| Metric Name | Type | Labels |
|------------|------|--------|
| `crypto_operations_total` | Counter | operation_type, result |
| `crypto_operation_duration_seconds` | Histogram | operation_type |
| `crypto_key_rotations_total` | Counter | key_type, rotation_reason |
| `crypto_active_keys_count` | Gauge | key_type |

### Log Format
```json
{
  "timestamp": "2025-10-01T10:30:00Z",
  "level": "INFO",
  "event": "key.rotated",
  "key_type": "DeviceMaster",
  "old_key_id": "key_xyz",
  "new_key_id": "key_abc",
  "rotation_reason": "Scheduled",
  "trace_id": "..."
}
```

Reference: [spec/testing/metrics.md](../testing/metrics.md)

---

## 10. Key Rotation

### Automatic Rotation Schedule
| Key Type | Rotation Period | Trigger |
|----------|----------------|---------|
| Root Key | 365 days | Manual only (high risk) |
| Device Master Key | 90 days | Scheduled (Cron: 0 0 * * 0) |
| Session Key | On session termination | Event-driven |
| Stream Key | 24 hours | Scheduled |

### Rotation Procedure
```
1. Generate new key (HKDF-SHA512)
2. Save new key to Vault/Redis
3. Record KeyRotationLog
4. Mark old key as `rotated=true` (do not delete immediately)
5. Delete old key after Grace Period (1 hour)
6. Send event to Telemetry
```

Details: [spec/security/key-management.md](../security/key-management.md)

---

## 11. Requirements Traceability

### FR-02: Authentication
- **Related**: X25519 key exchange during device authentication
- **Implementation**: KeyExchangeRequest → SharedSecret

### NFR-02: Encryption
- **Related**: ChaCha20-Poly1305 encryption for all communications
- **Implementation**: EncryptRequest → EncryptedPayload

**Traceability ID Mapping**:
```
MOD-004-CRYPTO → FR-02 (authentication via key exchange)
MOD-004-CRYPTO → NFR-02 (end-to-end encryption)
```

---

## 12. Test Strategy

### Unit Tests
- X25519 key exchange (10 cases, including abnormal public key length)
- ChaCha20-Poly1305 encryption/decryption (20 cases, including nonce reuse detection)
- HKDF-SHA512 key derivation (15 cases)
- Ed25519 signature/verification (10 cases)
- Coverage target: 95%

### Integration Tests
- Vault integration (Root Key retrieval, Device Master Key storage)
- Redis integration (Session Key TTL verification)
- Key rotation E2E (decryption possible with old key, Grace Period verification)

### Security Tests
- Nonce reuse attack detection
- Tag tampering detection
- Known-plaintext attack resistance verification

Details: [spec/testing/unit-tests.md](../testing/unit-tests.md), [spec/security/vulnerability.md](../security/vulnerability.md)

---

## 13. Deployment & Operations

- **Deployment Method**: Blue/Green deployment
- **Infrastructure Requirements**: 1 vCPU, 512MB RAM/instance
- **Rollback Condition**: Encryption error rate > 0.1% (sustained for 1 minute)

Details: [spec/deployment/ci-cd.md](../deployment/ci-cd.md)

---

## 14. Risks & Technical Debt

| Risk | Mitigation |
|------|------------|
| Nonce Collision | Counter mode + session ID prefix |
| Quantum Computer Attack | Kyber768 integration (2025 Q4) |
| HSM Failure | Multi-region Vault cluster |

---

## 15. Acceptance Criteria (DoD)

- [x] X25519/ChaCha20-Poly1305/HKDF-SHA512 specification description complete
- [x] Key hierarchy diagram (Root → Stream) creation complete
- [x] Consistency with encryption.md verification complete
- [x] Linkage with FR-02/NFR-02 specified
- [x] Traceability ID (`MOD-004-CRYPTO`) assigned
- [x] C/C++ dependency exclusion confirmed (using RustCrypto suite)
- [x] 90-day automatic rotation specification complete

---

## 16. Change History

| Version | Date | Changes | Approver |
|---------|------|---------|----------|
| 1.0 | 2025-10-01 | Initial version | Security WG (ENG-SEC-01) |
