# docs/architecture/dependencies.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Dependency management is a core element supporting HoneyLink™'s maintainability and security. This document defines abstract module dependencies and does not include implementation code or C/C++ integration.

## Table of Contents
- [Dependency Management Principles](#dependency-management-principles)
- [Module Dependency Graph](#module-dependency-graph)
- [Layer-Specific Rules](#layer-specific-rules)
- [Circular Dependency Avoidance](#circular-dependency-avoidance)
- [Versioning Policy](#versioning-policy)
- [Change Management Flow](#change-management-flow)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Dependency Management Principles
1. **Unidirectional:** Upper layer → Lower layer only. Reverse dependencies prohibited.
2. **Contract-Driven:** Define interface contracts in [docs/architecture/interfaces.md](./interfaces.md) and communicate via contracts without direct reference.
3. **C/C++ Exclusion:** Linking to native binaries prohibited. Use API-ized services if necessary.
4. **Plugin:** Profiles/policies etc. can be hot-swapped into plugin slots.

## Module Dependency Graph
```
      +-----------------------+
      | Experience Layer      |
      +----------+------------+
                 |
                 v
      +----------+------------+
      | Session Orchestrator  |
      +----+------------------+
           | \
           |  \
           v   v
  +--------+-----+     +-----------------------+
  | Policy Engine |--> | QoS Scheduler        |
  +--------+-----+     +-----------------------+
           |
           v
  +-------------------+
  | Crypto & Trust    |
  +--------+----------+
           |
           v
  +-------------------+
  | Transport Abstraction |
  +--------+----------+
           |
           v
  +-------------------+
  | Physical Adapter  |
  +-------------------+

Telemetry & Insights listens to events from Session Orchestrator, QoS Scheduler, and Transport Abstraction.
```

## Layer-Specific Rules
| Layer | Allowed Dependencies | Prohibited Dependencies |
|-------|---------------------|------------------------|
| Experience | Session Orchestrator (via API) | Direct dependency on Policy/QoS/Crypto |
| Session | Policy, Crypto, Telemetry | Direct reference to Physical Adapter |
| Policy | QoS, Telemetry | Experience, Physical |
| QoS | Transport, Telemetry | Experience |
| Transport | Physical Adapter, Telemetry | Experience, Policy |
| Telemetry | Subscribe to events from all (unidirectional) | Bidirectional dependencies |
| Physical Adapter | None (external world only) | Reverse reference to upper layers |

## Circular Dependency Avoidance
- Allow only notifications via event bus, synchronous calls are unidirectional.
- Use dependency checklist in [docs/templates/module-template.md](../templates/module-template.md) when adding new features.
- Update ADR in [docs/notes/decision-log.md](../notes/decision-log.md) when dependency violations detected.

## Versioning Policy
- **SemVer:** Assign `major.minor.patch` to module specifications. Major changes break compatibility.
- **Compatibility Guarantee Period:** Provide old interface in parallel for 12 months after Major release.
- **Dependency Declaration:** Each module declares version range of partner modules it supports (e.g., `PolicyEngine >=2.0 <3.0`).
- **C/C++ Prohibition Application:** Prohibit including native modules in dependency declarations. Alternatives provided via API services.

## Change Management Flow
1. Create change proposal (RFC) and specify affected modules and version range.
2. Cross-check dependency graph impact with [docs/architecture/overview.md](./overview.md).
3. Add test plan to [docs/testing/integration-tests.md](../testing/integration-tests.md).
4. Reflect in roadmap ([docs/roadmap.md](../roadmap.md)) and adjust phases.
5. Update ADR ([docs/notes/decision-log.md](../notes/decision-log.md)).

## Acceptance Criteria (DoD)
- Dependency graph encompasses all modules with no bidirectional dependencies.
- Layer-specific rules describe specific prohibitions (especially C/C++ related).
- Versioning policy consistent with interface documents and defines compatibility period.
- Change management flow linked to processes in other documents.
- Exceptions such as Telemetry special cases are documented.
