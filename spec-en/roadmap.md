# docs/roadmap.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> This roadmap focuses solely on specification development and standardization processes, excluding implementation descriptions and C/C++-dependent tasks. Technologies selected in each phase refer to [docs/architecture/tech-stack.md](./architecture/tech-stack.md) with alternative plans and exit strategies ensured.

## Table of Contents
- [Phase Overview](#phase-overview)
- [Timeline and Deliverables](#timeline-and-deliverables)
- [Release Policy](#release-policy)
- [Risks and Mitigation Strategies](#risks-and-mitigation-strategies)
- [Dependency Map](#dependency-map)
- [Measurement Metrics (SLI/SLO)](#measurement-metrics-slislo)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Phase Overview
| Phase | Purpose | Entry Criteria | Exit Criteria |
|-------|---------|----------------|---------------|
| P0 – Concept Finalization | Unify vision and use cases | Key personas approved, requirements extracted ([docs/requirements.md](./requirements.md)) | Agreement on elevator pitch and KPIs, risk hypothesis organized |
| P1 – Specification Elaboration | Protocol specification, UI, security details | P0 complete, WG composition complete | 30 documents developed and reviewed, traceability established |
| P2 – Prototype Design | Abstract design and test plan finalization | P1 complete, PoC resources secured | PoC design documents and test templates completed (no implementation descriptions) |
| P3 – Standardization Preparation | External evaluation and certification preparation | P2 complete, external partner agreement | Governance/CI-CD/rolling release policy approved |
| P4 – Continuous Improvement | KPI tracking and specification revision guide operations | P3 complete | KPI achievement status quarterly review, revision process operational |

## Timeline and Deliverables
```
Quarter | Key Deliverables
----------------------------------------------
Q1      | Requirements finalization, UX principles, security policy
Q2      | Complete protocol specification, dataflow diagrams
Q3      | Test design templates, benchmark plans
Q4      | Governance documents, external evaluation preparation package
```

Key deliverables are archived in a state that meets each document's DoD and history is maintained according to version management policy.

## Release Policy
- **Specification Release Cycle:** Semi-annual. Major for standard changes, Minor for explanatory additions, Patch for typographical corrections.
- **Approval Process:** Working group consensus → Security/UX review → Governance committee approval.
- **Compatibility Maintenance:** For breaking changes, follow versioning rules in [docs/architecture/interfaces.md](./architecture/interfaces.md) and simultaneously publish backward-compatible adapter specifications.
- **Exit Strategy:** If KPI targets are not met for 2 consecutive periods, consider alternative approaches (e.g., collaboration with other company protocols) based on [docs/architecture/tech-stack.md](./architecture/tech-stack.md#exit-strategy).

## Risks and Mitigation Strategies
| Risk | Impact | Indicators | Mitigation |
|------|--------|------------|------------|
| Handshake latency > KPI | High | Test plan exceeds target | Revise specifications to adjust retry limits, re-evaluate QoS algorithm |
| Discrepancy between standards and UX | Medium | Low usability test ratings | Joint review based on UX principles in [docs/ui/overview.md](./ui/overview.md) |
| Security certification failure | High | External audit findings | Priority response to countermeasure map in [docs/security/vulnerability.md](./security/vulnerability.md) |
| Delay in partner adoption | Medium | Contract conclusion delays | Pre-launch workshops, alternative schedule development |
| Difficulty measuring non-functional requirements | Medium | SLI inconsistencies | Advance development of measurement plan in [docs/testing/metrics.md](./testing/metrics.md) |

## Dependency Map
```
P0 -> P1 -> P2 -> P3 -> P4
 |      |      |      |
 |      |      |      +--> External certification body review (parallel)
 |      |      +--> Benchmark test plan (Testing WG)
 |      +--> Security specification (Security WG)
 +--> Requirements definition (Requirements WG)
```
- Detailed dependencies for each phase are synchronized with module dependency diagrams in [docs/architecture/dependencies.md](./architecture/dependencies.md).

## Measurement Metrics (SLI/SLO)
- **Schedule Compliance Rate:** Milestone achievement rate for each phase ≥95%.
- **Review Completion Rate:** Required stakeholder attendance rate ≥90%. In case of absence, supplementary review within 48 hours.
- **Risk Resolution Rate:** 100% agreement rate on mitigation measures for Medium or higher risks.
- **Document Quality:** 100% DoD satisfaction for each document (review checklist reminder).

## Acceptance Criteria (DoD)
- Entry/exit criteria are documented for all phases with links to related documents established.
- Risks and mitigation strategies are consistent with threat models such as STRIDE and performance plans.
- Timeline and deliverables match the publication order of 30 documents with correct cross-references.
- "No Implementation Code Output" and "No C/C++ Dependencies" policies are verified to be followed in each section through reviews.
- Measurement metrics are consistent with KPI/OKR and SLI/SLO, with measurement methods linked to [docs/testing/metrics.md](./testing/metrics.md).
