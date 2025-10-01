# HoneyLink™ Control Plane API Specification

## Overview
The HoneyLink™ Control Plane API is an HTTPS-based REST/JSON API for integrated management of device registration, session control, and policy distribution. All communications require mTLS on HoneyLink Secure Transport (HLST) and are designed to apply the same specifications in any environment (on-premise, air-gapped, cloud).

- **Base URL:** `https://<control-plane-host>/api/v1`
- **Media Type:** `application/json; charset=utf-8`
- **Authorization:** mTLS + OAuth2-equivalent short-lived tokens (5 minutes)
- **Traceability:** OpenTelemetry Traceparent header required

## Security
- Client certificates allow `ed25519` or `secp384r1`. RSA is prohibited.
- Tokens are issued from OIDC-compatible IdP or, in offline environments, use hardware tokens to generate equivalent signed access tokens.
- All audit logs comply with audit requirements in `docs/security/key-management.md`.

## Error Model
| HTTP | Error Code | Description |
|------|------------|-------------|
| 400 | `ERR_VALIDATION` | Input validation error |
| 401 | `ERR_AUTH` | Authentication failure / Invalid certificate |
| 403 | `ERR_AUTHZ` | RBAC/ABAC policy violation |
| 404 | `ERR_NOT_FOUND` | Resource does not exist |
| 409 | `ERR_CONFLICT` | Version conflict, duplicate registration |
| 422 | `ERR_STATE` | State transition violation |
| 500 | `ERR_INTERNAL` | Unexpected error |
| 503 | `ERR_DEPENDENCY` | Lower service unavailable |

```json
{
  "error_code": "ERR_VALIDATION",
  "message": "pairing_code is invalid",
  "trace_id": "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
}
```

## Primary Resources

### 1. Device Registration
- **Endpoint:** `POST /devices`
- **Purpose:** Initial device registration on manufacturing line or in the field.
- **Request**
```json
{
  "device_id": "HL-EDGE-0001",
  "public_key": "base64(x25519 pub)",
  "firmware_version": "1.2.0",
  "capabilities": ["telemetry", "control"],
  "attestation": {
    "format": "remote-attestation-v1",
    "evidence": "base64(blob)",
    "nonce": "hex"
  },
  "metadata": {
    "location": "factory-A",
    "serial": "SN12345678"
  }
}
```
- **Validation**
  - `device_id`: `/^[A-Z0-9-]{4,64}$/`
  - `public_key`: X25519 32 bytes
  - `firmware_version`: SemVer compatible
- **Response (201)**
```json
{
  "device_token": "opaque",
  "pairing_code": "N3Q6-9P4L-7XZC",
  "registered_at": "2025-03-01T09:00:00Z",
  "expires_at": "2025-03-01T09:10:00Z"
}
```

### 2. Pairing Establishment
- **Endpoint:** `POST /devices/{device_id}/pair`
- **Purpose:** Device mutually authenticates with control plane.
- **Request**
```json
{
  "pairing_code": "N3Q6-9P4L-7XZC",
  "device_cert_csr": "base64(pem)",
  "telemetry_topics": ["default", "latency"],
  "policy_version": "2025.03.01"
}
```
- **Response (200)**
```json
{
  "device_certificate": "base64(pem)",
  "policy_bundle": {
    "version": "2025.03.01",
    "sha512": "hex",
    "signed_payload": "base64"
  },
  "session_endpoint": "quic://core.honeylink.local:7843"
}
```
- **State Transition:** `pending` → `paired`

### 3. Session Control
- **Endpoint:** `POST /sessions`
- **Purpose:** Multi-stream session establishment request.
- **Request**
```json
{
  "device_id": "HL-EDGE-0001",
  "requested_streams": [
    { "name": "telemetry", "mode": "unreliable", "qos": "burst" },
    { "name": "control", "mode": "reliable", "qos": "latency" }
  ],
  "traceparent": "00-..."
}
```
- **Response (201)**
```json
{
  "session_id": "sess_8f1c0d3a",
  "expires_at": "2025-03-01T09:30:00Z",
  "stream_allocations": [
    { "name": "telemetry", "cid": 4, "key_material": "base64" },
    { "name": "control", "cid": 1, "key_material": "base64" }
  ]
}
```
- **Note:** `key_material` is shared with device via HKDF and stored only in volatile memory.

### 4. Policy Distribution
- **Endpoint:** `PUT /devices/{device_id}/policy`
- **Purpose:** Update QoS, cipher suite, feature flags.
- **Request**
```json
{
  "policy_version": "2025.03.10",
  "qos": {
    "telemetry": { "priority": 3, "latency_budget_ms": 150 },
    "control": { "priority": 1, "latency_budget_ms": 30 }
  },
  "encryption": {
    "ciphers": ["chacha20-poly1305"],
    "fallback": null
  },
  "features": {
    "ota_update": false,
    "diagnostics": true
  }
}
```
- **Response (202)**
```json
{
  "policy_version": "2025.03.10",
  "applied": true,
  "applied_at": "2025-03-10T02:00:00Z"
}
```

### 5. Key Rotation
- **Endpoint:** `POST /devices/{device_id}/keys/rotate`
- **Purpose:** Request ad-hoc key update.
- **Request**
```json
{
  "reason": "suspected-compromise",
  "requested_by": "secops",
  "traceparent": "00-..."
}
```
- **Response (200)**
```json
{
  "rotation_id": "rot_45ab",
  "status": "initiated",
  "expected_completion": "2025-03-01T09:05:00Z"
}
```
- **Follow-up:** Device sends `POST /devices/{device_id}/keys/ack` after rotation completion.

### 6. Audit Event Retrieval
- **Endpoint:** `GET /audit/events`
- **Query:** `?device_id=HL-EDGE-0001&since=2025-03-01T00:00:00Z`
- **Response (200)**
```json
{
  "events": [
    {
      "id": "evt_c4a2",
      "timestamp": "2025-03-01T08:59:00Z",
      "category": "key-rotation",
      "actor": "secops",
      "outcome": "success",
      "details": {
        "rotation_id": "rot_45ab"
      }
    }
  ],
  "next": null
}
```

## Webhook (Optional)
- **Endpoint:** `POST https://<customer-endpoint>/honeylink/events`
- **Event Types:** `policy-applied`, `key-rotation`, `device-alert`
- **Signature:** HTTP header `X-HoneyLink-Signature: ed25519(base64)`
- **Retry:** Exponential backoff (maximum 24 hours).
- **On-premise Requirement:** Do not use Webhooks in offline environments, allow only polling via audit API.

## Versioning
- Breaking changes are separated by major version `/api/v{n}`.
- Schema differences are appended to `docs/templates/api-change-log.md`.

## SLA Metrics
| Metric | Target | Notes |
|--------|--------|-------|
| API Success Rate | 99.95% | Aggregation period: Monthly |
| Average Response Time (p95) | 180ms | Session control endpoint |
| Audit Delivery Latency | Within 60 seconds | Webhook or polling |

## Related Documents
- `docs/security/key-management.md`: Key rotation and issuance policy
- `docs/requirements.md`: FR-03, FR-05, NFR-02, NFR-07
- `docs/architecture/module-control-plane.md`: Control plane design
- `docs/testing/api.md`: API test cases
