# docs/architecture/overview.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> This document covers only abstract architecture, excluding implementation languages, code snippets, and C/C++-dependent elements. Refer to related documents for detailed dataflow and interface specifications.

## Table of Contents
- [Architecture Principles](#architecture-principles)
- [Component Diagram](#component-diagram)
- [Component Responsibilities](#component-responsibilities)
- [Architecture Patterns](#architecture-patterns)
- [Boundaries and Dependency Management](#boundaries-and-dependency-management)
- [Observability and Metrics](#observability-and-metrics)
- [Changeability Assessment](#changeability-assessment)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Architecture Principles
1. **Layered + Hexagonal Hybrid:** Separate core domain from boundary adapters to minimize impact of external system changes.
2. **Event-Driven Complement:** QoS and state changes are communicated via events, enabling asynchronous collaboration of streams.
3. **Zero Trust:** All components mutually authenticate and apply least privilege.
4. **Built-in Observability:** Treat SLIs as first-class objects and synchronize with [docs/testing/metrics.md](../testing/metrics.md).
5. **C/C++ Independence:** Provide cryptography and FEC through pure language implementations or cloud services, designing abstract adapters.

## Component Diagram
```
+-------------------------------------------------------------+
|                      HoneyLink Core                          |
|                                                             |
|  +----------------------+   +-----------------------------+ |
|  |  Session Orchestrator|<->|  Policy & Profile Engine    | |
|  +----------+-----------+   +--------------+--------------+ |
|             ^                              ^                |
|             |                              |                |
|  +----------+-----------+      +-----------+------------+   |
|  | Transport Abstraction |<----| Telemetry & Insights   |   |
|  +----------+-----------+      +-----------+------------+   |
|             |                              ^                |
|             v                              |                |
|  +----------+-----------+      +-----------+------------+   |
|  | Crypto & Trust Anchor |      | Stream QoS Scheduler  |   |
|  +-----------------------+      +-----------------------+   |
+-------------------------------------------------------------+
            |                          |
            v                          v
+-----------+-----------+     +--------+---------+
| Physical Adapter Layer |     | Experience Layer |
| (Wi-Fi/5G/THz bridges) |     | (SDK, UI Shell)  |
+------------------------+     +------------------+
```

For detailed data paths, refer to [docs/architecture/dataflow.md](./dataflow.md).

## Component Responsibilities
| Component | Primary Responsibilities | Input/Output | Reference |
|-----------|--------------------------|--------------|-----------|
| Session Orchestrator | Handshake, session state management, version negotiation | Connection requests, policy application instructions | [docs/security/auth.md](../security/auth.md) |
| Policy & Profile Engine | Use-case-specific policy decision, profile management | QoS requirements, environment metadata | [docs/performance/scalability.md](../performance/scalability.md) |
| Transport Abstraction | Bridge between logical streams and physical layer, FEC adapter | Packets, status metrics | [docs/architecture/interfaces.md](./interfaces.md) |
| Crypto & Trust Anchor | Key management, certificate coordination, secret management | Handshake materials, key store | [docs/security/encryption.md](../security/encryption.md) |
| Stream QoS Scheduler | Priority control, bandwidth allocation, backpressure | Stream requests, network status | [docs/performance/benchmark.md](../performance/benchmark.md) |
| Telemetry & Insights | SLI collection, event analysis, alert coordination | Observation data, reports | [docs/testing/metrics.md](../testing/metrics.md) |
| Physical Adapter Layer | Physical layer dependency absorption, driver abstraction | Radio settings, hardware events | Pure abstract API due to C/C++ prohibition |
| Experience Layer | SDK API and UI Shell specs, device guidance | UX patterns, user input | [docs/ui/overview.md](../ui/overview.md) |

## Architecture Patterns
- **Session Control:** Maintain minimal core as microkernel, extend profiles/policies as plugins.
- **Event Bridge:** Major components communicate via event bus (e.g., in-memory queue / message abstraction).
- **CQRS-like Separation:** Separate configuration changes (Command) from state queries (Query) to enhance observability.
- **Strategy Pattern:** Swap QoS and encryption mode strategies per profile.

## Boundaries and Dependency Management
- Inter-layer dependencies are upper→lower only. Reverse dependencies are prohibited.
- External integration (e.g., ID providers) described in port/adapter pattern.
- For detailed dependency rules, see [docs/architecture/dependencies.md](./dependencies.md).
- Physical layer adapters commonly use C/C++ implementations, but this specification designs language-independent APIs such as gRPC and REST for pure implementations or external service alternatives.

## Observability and Metrics
- Each component outputs SLIs to align with [docs/testing/metrics.md](../testing/metrics.md).
- Telemetry & Insights layer aggregates metrics and triggers procedures in [docs/deployment/rollback.md](../deployment/rollback.md) when SLO is violated.

## Changeability Assessment
| Domain | Change Example | Impact Scope | Mitigation |
|--------|----------------|--------------|------------|
| Profile Addition | New profile template | Policy Engine, QoS Scheduler | Guide with [docs/templates/module-template.md](../templates/module-template.md) |
| Encryption Method Update | New key exchange | Centered on Crypto & Trust Anchor | Other layers only update key ID through abstraction |
| Physical Layer Addition | New adapter | Physical Adapter, Transport Abstraction | Guarantee API compatibility in [docs/architecture/interfaces.md](./interfaces.md) |

## Acceptance Criteria (DoD)
- All components clearly state responsibilities, input/output, and references with links to other documents.
- Diagrams and tables reflect C/C++ independence approach.
- Architecture principles are consistent with requirements, performance, and security documents.
- Component diagram matches latest dependency rules with review records in [docs/notes/decision-log.md](../notes/decision-log.md).
- Observation metrics have bidirectional references with [docs/testing/metrics.md](../testing/metrics.md).
