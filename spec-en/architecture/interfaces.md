# docs/architecture/interfaces.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines internal and external interfaces of HoneyLink™ in a language-independent manner. APIs described with abstract parameters and structures, excluding implementation code and C/C++-dependent specifications.

## Table of Contents
- [I/F Design Policy](#if-design-policy)
- [External APIs](#external-apis)
- [Internal Module I/F](#internal-module-if)
- [Data Model (Abstract Schema)](#data-model-abstract-schema)
- [Error Handling and Retry Strategy](#error-handling-and-retry-strategy)
- [Compatibility and Versioning](#compatibility-and-versioning)
- [Security and Authorization](#security-and-authorization)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## I/F Design Policy
1. **Language & Protocol Independent:** Present abstract models expressible in JSON/YAML/CBOR etc.
2. **Backward Compatibility Priority:** Mandate version field, phase out with deprecated flag.
3. **Secure by Design:** All external calls protected by mutual TLS or tokens.
4. **C/C++ Prohibition:** API design assuming native extensions not allowed. Resolve with L7 adapters if necessary.

## External APIs
| API Name | Path/Channel (Abstract) | Method | Authentication | Description | Response SLA |
|----------|------------------------|--------|----------------|-------------|--------------|
| Device Discovery Service | `/v1/discovery/devices` | QUERY | OAuth2 Client Credentials | Retrieve Beacon results, filtering | Within 200ms |
| Pairing Initiation | `/v1/pairing/sessions` | POST | OIDC Token + Proof of Possession | Create pairing request | Within 500ms |
| Profile Catalog | `/v1/profiles` | GET | OAuth2 | List available profiles | Within 150ms |
| Telemetry Ingest | `otlp://telemetry.honeylink/` | STREAM | mTLS | Send metrics/logs | 99.9% continuous operation |
| Policy Management | `/v1/policies/{id}` | PUT | OAuth2 + RBAC | Update policy definition | Within 400ms |
| Audit Export | `/v1/audit/streams` | STREAM | Signed Event Token | Subscribe to immutable logs | Within 24h delay |

## Internal Module I/F
| Producer | Consumer | Channel | Message Structure | Notes |
|----------|----------|---------|-------------------|-------|
| Session Orchestrator | Crypto & Trust | Async Request Queue | `HandshakeRequest` | Idempotent ID required |
| Policy Engine | QoS Scheduler | Event Bus | `QoSPolicyUpdate` | Versioned |
| Transport Abstraction | Telemetry & Insights | Metrics Stream | `StreamMetric` | 1-second granularity |
| Physical Adapter | Transport Abstraction | Callback API | `LinkStateChange` | Sandboxed communication |

## Data Model (Abstract Schema)
```
HandshakeRequest (object)
  - session_id: UUIDv7
  - client_public_key: Binary(32)
  - capabilities: List<CapabilityCode>
  - auth_context: map<string, string>

QoSPolicyUpdate (object)
  - policy_version: String
  - stream_id: UInt8
  - latency_budget_ms: UInt16
  - bandwidth_floor_mbps: Decimal
  - fec_mode: Enum { NONE, LIGHT, HEAVY }
  - expiration_ts: Timestamp

StreamMetric (object)
  - metric_version: String
  - stream_id: UInt8
  - latency_ms_p95: Float
  - jitter_ms_stddev: Float
  - loss_ratio: Float
  - battery_mw: Float
```

## Error Handling and Retry Strategy
- **Common Error Codes:** `INVALID_REQUEST`, `UNAUTHORIZED`, `CONFLICT`, `RETRYABLE`, `RATE_LIMITED`.
- **Retry:** Use exponential backoff (initial 200ms, max 2s) when RETRYABLE occurs, idempotency-key required.
- **Idempotency:** Each POST/PUT requires `Idempotency-Key` header. Use C-independent hash functions like FNV-1a for hash.
- **Audit:** Critical errors forwarded to incident flow defined in [docs/security/vulnerability.md](../security/vulnerability.md).

## Compatibility and Versioning
- API version in `v{major}` format. Branch path with `/v2/` etc. for Major changes.
- Field additions are backward compatible. Removals add `deprecated_after` metadata with 12-month grace period.
- Profile template compatibility matrix follows guidelines in [docs/templates/module-template.md](../templates/module-template.md).

## Security and Authorization
- Authentication method details in [docs/security/auth.md](../security/auth.md).
- Cryptographic parameters and key rotation in [docs/security/encryption.md](../security/encryption.md).
- All I/F calls evaluated from both RBAC/ABAC perspectives. Return `403` if scope/attributes insufficient.

## Acceptance Criteria (DoD)
- Parameters, errors, and SLA defined for both external and internal I/F.
- All models described language-independently without C/C++-specific concepts.
- Versioning and backward compatibility policy consistent with roadmap and testing policies.
- Security requirements linked to related documents.
- Idempotency and retry strategy consistent with [docs/architecture/dataflow.md](./dataflow.md).
