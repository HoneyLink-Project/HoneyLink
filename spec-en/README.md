# docs/README.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> This project strictly prohibits any implementation code or executable configuration descriptions and deals exclusively with pure specification definitions. C/C++ and libraries that depend on them are excluded from candidate selection, with alternative technologies clearly documented.

## Table of Contents
- [Elevator Pitch](#elevator-pitch)
- [Problem and Solution Approach](#problem-and-solution-approach)
- [Vision and Mission](#vision-and-mission)
- [Product Principles](#product-principles)
- [Key Scenarios](#key-scenarios)
- [System Overview Diagram](#system-overview-diagram)
- [Success Metrics (KPI/OKR)](#success-metrics-kpiokr)
- [Related Document Index](#related-document-index)
- [Contribution Guidelines](#contribution-guidelines)
- [Defined Terms and References](#defined-terms-and-references)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Elevator Pitch
HoneyLink™ is a next-generation universal wireless protocol that embodies "anyone, anywhere, instantly connected." While maximizing the use of existing wireless physical layers, it dynamically optimizes the latency, bandwidth, and reliability required by applications, making device collaboration experiences "as smooth as honey."

## Problem and Solution Approach
| Problem | HoneyLink™ Solution | Measurement Metrics |
|---------|---------------------|---------------------|
| UX fragmentation due to proliferation of multiple protocols | Profile integration and common handshake | 99.5% connection success rate for major use cases |
| Inconsistency in security strength | Industry-standard elliptic curve cryptography and zero-trust design | 100% MITM attack prevention rate (simulation tests) |
| Trade-off between low latency and high bandwidth | Multi-stream QoS and adaptive FEC control | P95 latency ≤ 8ms (LL Streams) |
| Difficulty in accommodating both IoT and rich media | Profile-based resource management | Battery life +30%, 4K streaming retention rate 98% |

For detailed functional requirements, please refer to [docs/requirements.md](./requirements.md).

## Vision and Mission
- **Vision:** Provide a "sweet and smooth" connection experience as the global standard for inter-device communication.
- **Mission:** Establish protocol specifications and surrounding ecosystems that are physical-layer independent, highly reliable, and UX-focused, minimizing the burden on both developers and users.
- **North Star Metric:** Session setup completion time under 5 seconds, annual churn rate under 3%.

## Product Principles
1. **Human-Centered:** Always minimize the number of connection operation steps and unify visual/tactile feedback.
2. **Secure by Default:** All channels require encryption and mutual authentication as mandatory.
3. **Adaptive Optimization:** Automatically detect per-stream latency and bandwidth requirements and tune according to network conditions.
4. **Built-in Observability:** Provide SLIs as standard stream metadata for operations and support.
5. **Future Compatibility:** Guarantee backward compatibility through version negotiation and implementation guides.

## Key Scenarios
- **Seamless Gaming Peripheral Connection:** Defined in [docs/requirements.md#use-cases](./requirements.md#use-cases).
- **Hi-Res Audio Streaming:** Bitrate and latency requirements specified in [docs/performance/scalability.md](./performance/scalability.md).
- **Multi-Site IoT Management:** Power-saving profiles and monitoring requirements described in [docs/architecture/dataflow.md](./architecture/dataflow.md).

## System Overview Diagram
```
        +-------------------------------+
        |       Application Layer        |
        |  (HoneyLink SDK / Portal)      |
        +---------------+---------------+
                        |
                        v
        +-------------------------------+
        |  Session Control & Policy Layer|
        |  - Handshake Management        |
        |  - Profile Adaptation          |
        +---------------+---------------+
                        |
                        v
        +-------------------------------+
        | Transport Abstraction & FEC   |
        |  - QoS Scheduler               |
        |  - FEC/Retransmission Control  |
        +---------------+---------------+
                        |
                        v
        +-------------------------------+
        | Physical Adapter (Wi-Fi/5G/THz)|
        +-------------------------------+
```

For detailed module configuration, refer to [docs/architecture/overview.md](./architecture/overview.md).

## Success Metrics (KPI/OKR)
- **KPI**
  - Connection Success Rate: ≥99.5% (monthly)
  - Initial Pairing Time: Average ≤4 seconds (P95 6 seconds)
  - Link Retention Rate: 99.9% in 24-hour continuous operation
  - Security Incident Count: 0 incidents
- **OKR Example (Quarterly)**
  - O: Expand support for new device categories (AR/VR)
    - KR1: Define and approve 3 standard profiles
    - KR2: Pass 5 UX field test cases
    - KR3: Improve latency KPI by 20%

## Related Document Index
| Domain | Path | Contents |
|--------|------|----------|
| Requirements | [docs/requirements.md](./requirements.md) | Personas, use cases, non-functional requirements |
| Architecture | [docs/architecture/overview.md](./architecture/overview.md) | Component responsibilities and boundaries |
| Dataflow | [docs/architecture/dataflow.md](./architecture/dataflow.md) | Synchronous/asynchronous processing, consistency |
| Security | [docs/security/auth.md](./security/auth.md) | Authentication and authorization model |
| Testing | [docs/testing/metrics.md](./testing/metrics.md) | SLI/SLO and quality gates |
| Deployment | [docs/deployment/ci-cd.md](./deployment/ci-cd.md) | Abstract pipeline |

## Contribution Guidelines
- **Working Groups:** Five councils: Architecture, Protocol, UX, Security, and Operations.
- **Communication:** Record weekly reviews using the [docs/notes/meeting-notes.md](./notes/meeting-notes.md) template.
- **Traceability:** Link Issue ⇄ Requirements ⇄ Design ⇄ Tests according to [docs/requirements.md#traceability-policy](./requirements.md#traceability-policy).
- **Change Management:** Conventional Commits (e.g., `feat: Add session key update algorithm specification`) are mandatory.
- **Review Criteria:** Each document must meet DoD and ensure alignment with related SLI/SLO.

## Defined Terms and References
- For formal term definitions, refer to [docs/requirements.md#glossary](./requirements.md#glossary).
- Architectural dependencies are documented in [docs/architecture/dependencies.md](./architecture/dependencies.md).

## Acceptance Criteria (DoD)
- Links to requirement documents are established for all critical scenarios.
- KPI/OKR are measurable and consistent with SLI/SLO in other documents.
- ASCII system diagram is confirmed to match the latest architecture version through reviews.
- "No Implementation Code Output" and "No C/C++ Dependencies" policies are explicitly stated and applied to all sections.
- Document index covers all 30 files (at least one link established).
