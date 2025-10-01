# docs/security/auth.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Specifies HoneyLink™'s authentication and authorization architecture. Abstract description independent of language/implementation, C/C++ libraries are out of scope.

## Table of Contents
- [Authentication Infrastructure Policy](#authentication-infrastructure-policy)
- [OIDC/OAuth2 Flow](#oidcoauth2-flow)
- [Session Management](#session-management)
- [Authorization Model (RBAC/ABAC)](#authorization-model-rbacabac)
- [Device Trust and Compliance](#device-trust-and-compliance)
- [Audit and Logging](#audit-and-logging)
- [Threats and Countermeasures](#threats-and-countermeasures)
- [Related Documents](#related-documents)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Authentication Infrastructure Policy
- **IdP:** External OIDC providers (e.g., Azure AD, Auth0). Do not use proprietary C/C++-based stacks.
- **Mandatory MFA:** Required for administrators and SRE. Requested risk-based for users.
- **Device Binding:** Use device certificates (X.509) or WebAuthn during pairing.
- **Zero Trust:** Evaluate user/device/context for all API calls.

## OIDC/OAuth2 Flow
| Flow | Target | Description |
|------|--------|-------------|
| Authorization Code with PKCE | End Users | Mobile/web portal operations. PKCE prevents code interception |
| Client Credentials | Microservices | Inter-service communication between session management and policy engine |
| Device Code | IoT | For devices without screens. QR/code input |

### Token Structure
- **ID Token:** Audience = HoneyLink portal. Contains claims: `sub`, `roles`, `device_id`, `mfa_level`.
- **Access Token:** Scope examples `stream.manage`, `policy.write`, `metrics.read`.
- **Refresh Token:** Rotating type. Provides revocation API.

## Session Management
- Session ID: UUIDv7. Server-side state stored in Session Orchestrator.
- Expiration: 12 hours. 30-minute sliding renewal with activity.
- Session Key: HKDF defined in [docs/security/encryption.md](./encryption.md).
- Reconnection: Device uses cached public key, user confirmation step is 2-factor.
- Logout: Immediately revoke all access tokens and session keys.

## Authorization Model (RBAC/ABAC)
| Role | Permissions | Attribute Examples |
|------|-------------|-------------------|
| `role:user` | Device connection, profile application | Company ID, region |
| `role:admin` | Policy creation/deletion, audit retrieval | Audit level, business hours |
| `role:sre` | Metrics viewing, alert resolution | Work shift, region |
| `role:auditor` | Read-only access | Contract ID |

- **ABAC:** Geography (region=EU/US), time (business_hours=true), device compliance (compliant=true).
- Policy Language: JSON-based DSL. Do not use C/C++ parsers, process with standard JSON evaluators.

## Device Trust and Compliance
- Device certificates issued by managed CA. Revocation information provided by CRL/OCSP-equivalent service.
- Scoring: Evaluated by OS version, cryptographic module, security patches.
- If compliance judgment is false, restrict access token scope.

## Audit and Logging
- Record all role/token operations in conjunction with key scopes in [docs/security/encryption.md](./encryption.md).
- Log Format: Immutable JSON Lines. Contains fields: `event_id`, `timestamp`, `actor`, `context`, `result`.
- Retention Period: 1 year, Write Once Read Many (WORM) storage.

## Threats and Countermeasures
| Threat | Example | Countermeasure |
|--------|---------|----------------|
| Phishing | Fake portal guidance | MFA requirement, FIDO2 support |
| Token Theft | Access token leakage | PoP token, short-lived Access Token, DPoP |
| Replay Attack | Session reuse | Nonce/replay prevention store |
| Privilege Escalation | RBAC configuration leak | Quarterly review, policy template signing |

## Related Documents
- [docs/security/encryption.md](./encryption.md): Key management and encryption details
- [docs/security/vulnerability.md](./vulnerability.md): STRIDE-based vulnerability countermeasures
- [docs/architecture/interfaces.md](../architecture/interfaces.md): Authentication header specifications
- [docs/testing/e2e-tests.md](../testing/e2e-tests.md): Authentication scenario tests

## Acceptance Criteria (DoD)
- OIDC/OAuth2 flows are defined for all roles.
- Token, session, and authorization specifications are quantitatively described.
- Assumptions without using C/C++ are clarified.
- Audit and threat countermeasures are consistent with [docs/security/vulnerability.md](./vulnerability.md).
- Related document links are valid and coordinated with roadmap/testing.
