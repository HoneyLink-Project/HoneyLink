# HoneyLink Governance Charter

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> This charter defines the working group structure, communication channels, and review cadence required to execute the HoneyLink roadmap. Assignees are recorded as role-based identifiers to avoid referencing personal data while keeping accountability explicit.

---

## 1. Working Group Roster

| Working Group | Primary Scope | Chair (Role ID) | Core Members (Role IDs) | Communication Channel | Weekly Review Cadence | Key Deliverables |
|---------------|---------------|-----------------|-------------------------|-----------------------|-----------------------|------------------|
| Architecture WG | Systems architecture, module boundaries, dependency control | ENG-ARCH-01 (Principal Systems Architect) | ENG-ARCH-02 (Async Systems Designer), ENG-ARCH-03 (Observability Architect), ENG-ARCH-04 (Infrastructure Liaison) | Slack `#hl-arch-wg`, Email list `arch-wg@honeylink.local` | Tuesday 15:00-16:00 JST | Updated diagrams ([spec/architecture/overview.md](../architecture/overview.md)), dependency ledger, ADR drafts |
| Protocol WG | Session orchestration, transport abstraction, API contracts | ENG-PROTO-01 (Lead Protocol Strategist) | ENG-PROTO-02 (Session Analyst), ENG-PROTO-03 (QoS Scheduler Partner), ENG-PROTO-04 (API Steward) | Slack `#hl-proto-wg`, Confluence space `HoneyLink/Protocol` | Wednesday 10:00-11:00 JST | Control-plane API specs, state machines, interface baselines |
| UX WG | Experience layer specifications, accessibility, visual design | UX-LEAD-01 (Experience Director) | UX-LEAD-02 (Accessibility Specialist), UX-LEAD-03 (Motion Designer), UX-LEAD-04 (Localization Strategist) | Slack `#hl-ux-wg`, Figma team `HoneyLink UX` | Monday 14:00-15:00 JST | Wireframes, design tokens, UI review logs |
| Security WG | Crypto & trust anchors, identity, threat modeling | SEC-ARCH-01 (Security Principal) | SEC-ARCH-02 (Key Management SME), SEC-ARCH-03 (Auth Strategist), SEC-ARCH-04 (Vuln Response Lead) | Slack `#hl-sec-wg`, Secure channel (Matrix `@sec-wg:honeylink.local`) | Thursday 11:00-12:00 JST | Threat model updates, key lifecycle procedures, compliance mappings |
| Operations WG | Deployment, telemetry, SRE playbooks, runbooks | OPS-LEAD-01 (SRE Program Owner) | OPS-LEAD-02 (Telemetrist), OPS-LEAD-03 (CI/CD Owner), OPS-LEAD-04 (Incident Commander) | Slack `#hl-ops-wg`, PagerDuty schedule `HoneyLink-Primary` | Friday 10:00-11:30 JST | Runbook revisions, deployment recipes, DR drills |

> **Note:** Each chair is accountable for keeping the roster current in this document and pushing any personnel changes to the Decision Log within 72 hours.

---

## 2. Communication & Escalation Model

1. **WG Chairs Sync:** Every second Monday 17:00-17:30 JST (Slack huddle `#hl-wg-leads`). Agenda: cross-WG blockers, milestone readiness, and risk alignment.
2. **Roadmap Alignment Review:** First Thursday of each month, co-hosted by Architecture and Operations WGs. Output: alignment memo linked to [spec/roadmap.md](../roadmap.md).
3. **Escalation Path:**
   - Immediate blocker ➜ Notify relevant WG chair via Slack DM and tag `#hl-incident`.
   - If unresolved within 24 hours ➜ escalate to Governance Council (Arch + Sec + Ops chairs) using Matrix secure room `!govcouncil:honeylink.local`.
   - Document resolution in [spec/notes/decision-log.md](decision-log.md) with status `Proposed` or `Approved` as appropriate.

---

## 3. Review Cadence and Deliverable Mapping

| Cadence | Responsible WG(s) | Deliverables | Acceptance Criteria Reference |
|---------|-------------------|--------------|-------------------------------|
| Weekly (per WG) | Respective WG | Meeting minutes stored in `spec/notes/meeting-notes.md` (one entry per session) | Complete agenda coverage, action items with owners, risks logged |
| Bi-weekly Chairs Sync | All chairs | Cross-WG status digest (`/notes/chairs-digest-YYYYMMDD.md`) | Risks reconciled, handoffs clear, outstanding ADR updates identified |
| Monthly Roadmap Review | Architecture + Operations | Updated milestone burndown, risk register refresh | Consistency with [spec/architecture/dependencies.md](../architecture/dependencies.md) and [spec/testing/metrics.md](../testing/metrics.md) |
| Quarterly Audit Prep | Security + Operations | Compliance posture report, DR drill summary | Align with [spec/security/vulnerability.md](../security/vulnerability.md) and roadmap P3 criteria |

---

## 4. Decision Log Workflow

1. Draft decisions recorded by the originating WG using ADR template within 24 hours of the meeting.
2. Chairs ensure cross-review by impacted WGs before status changes to `Approved`.
3. Operations WG tracks implementation actions in the shared action register (AI IDs inside meeting notes).
4. Security WG validates that no C/C++ dependencies are introduced; Architecture WG verifies backward compatibility before closure.

> **Note:** Ensure every ADR explicitly documents how the choice preserves C/C++ free dependencies and how rollback is supported because these are non-negotiable program constraints.

---

## 5. Metrics & Accountability

- **Attendance Threshold:** Each WG must maintain ≥90% attendance for core members per month. Chairs log exceptions in meeting notes and schedule catch-up within 48 hours.
- **Review SLA:** Deliverables requiring multi-WG input (e.g., protocol-spec touches security) must be reviewed within 5 business days; otherwise escalate per Section 2.
- **KPI Linkage:** Tie each WG output to the SLI/SLO catalog in [spec/testing/metrics.md](../testing/metrics.md) to preserve traceability.

---

## 6. Maintenance

- Chairs rotate every two roadmap phases to prevent burnout; nominate successor 30 days prior to transition.
- Update this charter whenever roster or cadence changes; commit message prefix: `docs: governance charter update`.
- Archive superseded versions under `spec/notes/archive/governance-YYYYMMDD.md` to keep history immutable.
