# docs/security/encryption.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines encryption and key management specifications in HoneyLink™. Limited to pure specification descriptions, excluding implementation code and C/C++-dependent modules.

## Table of Contents
- [docs/security/encryption.md](#docssecurityencryptionmd)
  - [Table of Contents](#table-of-contents)
  - [Cryptographic Policy Overview](#cryptographic-policy-overview)
  - [Encryption in Transit](#encryption-in-transit)
  - [Encryption at Rest](#encryption-at-rest)
  - [Key Management and Rotation](#key-management-and-rotation)
  - [Key Scope and Privilege Separation](#key-scope-and-privilege-separation)
  - [Secret Information Management](#secret-information-management)
  - [SBOM and Compliance](#sbom-and-compliance)
  - [Audit and Monitoring](#audit-and-monitoring)
  - [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Cryptographic Policy Overview
- **Standards:** Complies with NIST SP 800-56A, RFC 7748, RFC 8439.
- **Algorithms:** Key agreement X25519, key derivation HKDF-SHA512, symmetric encryption ChaCha20-Poly1305.
- **Hash:** SHA-512. Cryptographic libraries implemented in C/C++ are prohibited.

## Encryption in Transit
| Channel | Protocol | Specification |
|---------|----------|---------------|
| Session Control | HoneyLink Secure Transport (HLST) | mTLS + ChaCha20-Poly1305 |
| Telemetry | OTLP/gRPC over TLS 1.3 | AKE: TLS 1.3 (PSK prohibited) |
| Inter-Device | DTLS 1.3 Equivalent | WireGuard-compatible configuration as fallback |

- Ensure Perfect Forward Secrecy for all channels.
- Certificates issued by on-premise HSM or trusted managed PKI. Immediately revoke on failure.

## Encryption at Rest
| Data Type | Storage | Encryption Method |
|-----------|---------|-------------------|
| Session Secrets | HSM / Secure Element | AES-256-GCM (within hardware) |
| Logs/Audit | Immutable Media (WORM/Offline Media) | Server-side encryption + dedicated customer key |
| Profile Settings | Distributed SQL or Edge Key-Value Store | Column-level encryption + re-encryption at app layer |

- Backups stored encrypted in geographically separated vaults (offline possible). Decryption key access follows four-eyes principle.

## Key Management and Rotation
- **Key Management Layer:** On-premise HSM, Secure Element, or cloud-independent managed KMS. C/C++-based cryptographic modules are prohibited.
- **Rotation:** Automatic every 90 days. Session keys generated per connection.
- **Key Hierarchy:** Root (HSM) → Service Master → Session/Data Key.
- **Emergency Rotation:** Reissue keys within 30 minutes of incident occurrence. Add manual approval flow in air-gapped environments.

## Key Scope and Privilege Separation
| Key Type | Purpose | Access Entity |
|----------|---------|---------------|
| `k_root` | KMS internal, meta operations only | Security personnel (HSM) |
| `k_service` | Session management service | Session Orchestrator |
| `k_profile` | Profile data encryption | Policy Engine |
| `k_telemetry` | Observation data | Telemetry & Insights |

- Each key has least privilege access. Use ABAC conditions (region, time, role) in conjunction.

## Secret Information Management
- Secrets stored in on-premise Vault clusters or cloud-independent managed stores.
- Retrieval via short-lived tokens (5 minutes) + mTLS. Use hardware tokens in network-isolated environments.
- Add "encrypted" flag in YAML/JSON and prohibit unencrypted.

## SBOM and Compliance
- Manage cryptographic module origin in SBOM. Maintain license and audit history.
- Verify FIPS 140-3 compliance for each release and use FIPS-compliant mode if necessary.

## Audit and Monitoring
- Coordinate KMS API calls with audit in [docs/security/vulnerability.md](./vulnerability.md).
- Metrics: Key rotation success rate 100%, failure notification SLA 5 minutes.
- Alert thresholds: Rotation delay > 1 hour, access violation detection immediate notification.

## Acceptance Criteria (DoD)
- Encryption methods for transit/storage clearly documented, PFS and rotation policies quantified.
- Key scope and privilege separation table exists and connected to RBAC/ABAC policies.
- Secret management and audit strategies consistent with other documents (vulnerability, deployment).
- Assumption without C/C++ dependencies explicitly documented.
- SBOM and compliance requirements described in a form that can be reflected in roadmap.
