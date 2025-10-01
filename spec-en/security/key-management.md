# HoneyLink™ Key Management Specification

## Purpose
Clarify the overall key management in HoneyLink™ and ensure consistent security standards in all deployment formats including on-premise and air-gapped environments. This specification is implementation language-independent and assumes no use of C/C++-based libraries as cryptographic modules.

## Key Hierarchy
| Layer | Purpose | Generation Source | Storage Location |
|-------|---------|-------------------|------------------|
| Root Trust Anchor (RTA) | Sign entire trust chain | FIPS 140-3 compliant HSM or dedicated Secure Element | Offline control zone secure vault + HSM token |
| Intermediate CA / Service Keys | Control plane, inter-service authentication | Offline signing station (Rust + HSM workflow) | HSM / Vault cluster on isolated network |
| Edge / Device Keys | Device identification, session derivation | Device internal TRNG + X25519 | Device internal Secure Element |
| Session Keys | Stream encryption (ChaCha20-Poly1305) | X25519 → HKDF-SHA512 | Volatile memory only |

## Lifecycle
| Phase | RTA | Intermediate CA | Device Key | Session Key |
|-------|-----|-----------------|------------|-------------|
| Generation | Manual (2-person approval) | Automatic batch (mTLS) | Device initialization | Per handshake |
| Distribution | Physical media (offline) | On-premise Vault + mTLS | Pairing API (mTLS + registration code) | Within HLST channel |
| Rotation | 5 years / incident | 12 months / validation failure | 90 days / revocation event | Per connection + within 24 hours regeneration |
| Revocation | Secure Erase + audit log closure | CRL / OCSP | CRL + self-destruct | At handshake termination |
| Audit | Manual ledger + SBOM | GitOps trace | Device ledger | Session log (anonymized) |

## Generation Flow Details
1. **Root Key Generation**
   - Generate `ECDSA P-384` within offline HSM.
   - Preserve generation log as printout + digital (Write-Once Media).
   - Use only offline, re-isolate after short-term use.
2. **Intermediate CA Registration**
   - Convey signing request (CSR) to RTA via dedicated signing network.
   - Commit to GitOps repository after success, automatically attach audit trail.
3. **Device Key Issuance**
   - Device generates TRNG → X25519 pair on manufacturing line.
   - Convey registration package containing public key via QR/USB, register to control plane.
4. **Session Key Derivation**
   - Derive session key from X25519 → HKDF-SHA512 during handshake.
   - Place only in volatile memory, immediately zero on shutdown.

## Distribution/Registration Channels
- **Physical Offline Delivery:** Deliver root/intermediate materials via tamper-resistant media. Confirm with four-eyes principle upon receipt.
- **Pairing API:** Re-authenticate with registration code + mTLS setup. Details refer to `docs/api/control-plane.md`.
- **Directory Synchronization:** Sync between on-premise Vault clusters via Gossip + mTLS. Cloud environment also available but not mandatory.

## Rotation Policy
- **Schedule:**
   - RTA: Generate anew every 5 years. Perform cryptographic erasure (Secure Erase/NIST SP 800-88 compliant) on old key within HSM, confirm abolition in audit log.
   - Intermediate CA: Reissue every 12 months. Register old certificate to CRL and distribute revocation notice.
   - Device Key: 90 days + immediate rotation upon anomaly detection.
- **Event Trigger:**
   - When unauthorized access signs, audit log tampering, or configuration drift detected, initiate emergency rotation workflow within 30 minutes.
   - In air-gapped environments, perform signing operations with manual approval (one-time hardware token).

## Revocation & Destruction
- Use both CRL and OCSP, propagate revocation notice in real-time via pairing API.
- Upon device revocation, call secure element key erasure API, require manufacturing line re-verification for re-registration.
- Root key destruction performs Secure Erase procedure within HSM, preserve audit trail (log + approval signature).

## Protection Requirements
- Key material always stored in FIPS or ISO/IEC 19790 compliant modules.
- Device side uses only Rust/WASM implementation cryptographic layer. C/C++-based libraries prohibited.
- Execute zeroing routine before garbage collection in volatile memory, prohibit paging to prevent memory dumps.

## Audit and Observability
- Track all operations with OpenTelemetry Span, send cryptographic material masked to SIEM.
- Audit Events: Generation, signing, distribution, revocation, recovery attempt.
- Metrics: Success/failure rate, rotation time required, approval path, manual intervention count.

## Incident Response Matrix
| Scenario | Response Procedure | SLA |
|----------|-------------------|-----|
| Key Leakage Suspicion | Emergency rotation + complete audit log acquisition + impact range report | 30 min detection, within 4 hours containment |
| HSM Failure | Switch to failover HSM, verify integrity with SBOM | Within 15 min switching |
| CRL Distribution Failure | Re-distribute → fallback manual distribution → post-review | Within 60 min recovery |
| Air-gapped Site Rotation | Physical media exchange + signature approval double-check | Complete within 24 hours |

## Related Materials
- `docs/security/encryption.md`: Details of communication and at-rest encryption
- `docs/requirements.md`: Complies with FR-02, NFR-03, NFR-08
- `docs/testing/security.md`: Key management test cases
- `docs/deployment/runbook.md`: Rollback/rotation operations
