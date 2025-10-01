# Review Completion Checklist

**Badges:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> This checklist ensures that 90% review completion rate is consistently achieved across all HoneyLink working groups. It defines review items, approval criteria, and tracking mechanisms.

---

## 1. Objective and Key Results (OKR)

**Objective:** Maintain 90% review completion rate for all deliverables across all WGs.

**Key Results:**
- KR1: ≥90% of ADRs receive cross-review within 48 hours
- KR2: ≥90% of meeting action items are closed within sprint
- KR3: ≥90% of module specs pass quality gate before implementation
- KR4: ≥90% of code reviews are completed within 24 hours
- KR5: Zero critical deliverables miss review deadlines

---

## 2. Review Item Categories

### 2.1 Architecture Decision Records (ADRs)
**Target:** 100% cross-review within 48 hours  
**Tracking Method:** ADR status in `spec/notes/decision-log.md` + Slackbot reminder

**Review Checklist:**
- [ ] **Completeness:** All mandatory sections are filled (Background, Decision, Alternatives, Impact, Implementation Plan)
- [ ] **Clarity:** Technical rationale is clear and unambiguous
- [ ] **Dependency Analysis:** Architecture WG confirms no circular dependencies introduced
- [ ] **Security Review:** Security WG confirms C/C++ dependencies are excluded and crypto/auth implications are addressed
- [ ] **Operations Review:** Operations WG confirms deployment feasibility and SLO impact is acceptable
- [ ] **Backward Compatibility:** Breaking changes are documented with migration path or explicitly justified
- [ ] **Traceability:** Links to relevant specs (`spec/requirements.md`, `spec/architecture/*.md`) are valid
- [ ] **Actionability:** Implementation plan has clear completion criteria and owners

**Approval Criteria:**
- At least 2 WGs have provided written feedback (comment or approval)
- No unresolved critical objections from Security or Architecture WG
- WG chair signs off with `承認` status update

---

### 2.2 Module Specification Documents
**Target:** 100% review before implementation starts  
**Tracking Method:** GitHub PR review status + WG meeting action register

**Review Checklist:**
- [ ] **Traceability Matrix:** All FR/NFR IDs from `spec/requirements.md` are mapped
- [ ] **API Contract:** Input/output schemas are defined with examples and error cases
- [ ] **State Machine:** State transitions are documented with idempotency guarantees
- [ ] **Interface Consistency:** Conforms to `spec/architecture/interfaces.md` standards (SemVer, `deprecated_after`, etc.)
- [ ] **Dependency Declaration:** Upstream/downstream dependencies are explicit and match `spec/architecture/dependencies.md` layers
- [ ] **Observability Hooks:** Telemetry emission points are specified with OpenTelemetry format
- [ ] **C/C++ Exclusion:** No C/C++ dependencies are introduced; Rust/TypeScript alternatives are confirmed
- [ ] **Test Strategy:** Unit/integration test approach is outlined with coverage targets (≥80%)
- [ ] **DoD Alignment:** Meets template DoD from `spec/templates/module-template.md`

**Approval Criteria:**
- Protocol WG and Architecture WG have both approved
- Security WG has cleared security considerations
- At least 1 implementation engineer has confirmed feasibility

---

### 2.3 Meeting Action Items
**Target:** 90% closure within current sprint (2 weeks)  
**Tracking Method:** Action register in `spec/notes/meeting-notes.md` + weekly WG status reports

**Review Checklist:**
- [ ] **Clarity:** Action item has clear deliverable and acceptance criteria
- [ ] **Owner:** Single owner (role ID) is assigned
- [ ] **Due Date:** Realistic deadline is set (default: next WG meeting)
- [ ] **Dependencies:** Blockers are identified and tracked
- [ ] **Status Tracking:** Status is updated weekly (Not Started / In Progress / Blocked / Completed)

**Completion Criteria:**
- Deliverable is produced and linked in meeting notes
- Owner confirms completion in WG meeting
- Chair marks action as `Completed` with date

**Escalation for Overdue Items:**
- 3 days overdue: Chair sends reminder to owner
- 7 days overdue: Escalate to Chairs Sync
- 14 days overdue: Report to Governance Council

---

### 2.4 Code Reviews (Implementation Phase)
**Target:** 90% of PRs reviewed within 24 hours, merged within 48 hours  
**Tracking Method:** GitHub PR metrics dashboard + CI/CD pipeline gates

**Review Checklist:**
- [ ] **Build Pass:** CI pipeline (rustfmt, clippy, tests) is green
- [ ] **Test Coverage:** Added/modified code has ≥80% line coverage (measured by `cargo-llvm-cov`)
- [ ] **Code Quality:** No `clippy::pedantic` warnings without explicit `#[allow]` justification
- [ ] **Security Scan:** `cargo-audit` passes with no unresolved vulnerabilities
- [ ] **C/C++ Exclusion:** No C/C++ dependencies added (check `Cargo.lock` / `package-lock.json`)
- [ ] **API Stability:** Public API changes are backward-compatible or have ADR approval
- [ ] **Documentation:** Public functions have rustdoc/TSDoc comments with examples
- [ ] **Observability:** Critical paths emit structured logs/traces with `tracing` crate
- [ ] **Commit Hygiene:** Commits follow Conventional Commits format (`feat:`, `fix:`, etc.)
- [ ] **Change Size:** PR is ≤500 lines (excluding generated code); larger PRs have justification

**Approval Criteria:**
- At least 1 domain expert approval (same WG as the modified module)
- At least 1 general approval (different WG for cross-team perspective)
- All CI checks pass
- No unresolved "Request Changes" reviews

---

### 2.5 UI/UX Deliverables
**Target:** 100% review before Figma handoff to implementation  
**Tracking Method:** Figma file version history + UX WG approval log

**Review Checklist:**
- [ ] **Wireframe Alignment:** Screens match `spec/ui/wireframes.md` requirements
- [ ] **Design Token Usage:** Colors, typography, spacing adhere to `spec/ui/visual-design.md` tokens
- [ ] **Accessibility:** WCAG 2.2 AA compliance verified (contrast ratio ≥4.5:1, keyboard navigation, ARIA labels)
- [ ] **Responsive Design:** Layouts tested for mobile (<600px), tablet (600-1024px), desktop (>1024px)
- [ ] **Animation Specs:** Timing/easing documented per `spec/ui/animations.md`, `prefers-reduced-motion` alternative defined
- [ ] **i18n Readiness:** Text strings are externalized, 30% expansion rule applied to layouts, RTL considerations documented
- [ ] **Developer Handoff:** Assets exported (SVG/PNG), component props documented, interaction states defined

**Approval Criteria:**
- UX WG chair approves design
- Accessibility specialist signs off on WCAG compliance
- At least 1 frontend engineer confirms implementability

---

## 3. Tracking Mechanisms

### 3.1 Weekly WG Status Report Template
Each WG submits this report every Friday 17:00 JST to `#hl-wg-leads`:

```markdown
## [WG Name] Weekly Status - YYYY-MM-DD

### Review Completion Metrics
- ADRs reviewed this week: X / Y (Z%)
- Action items closed: X / Y (Z%)
- PRs reviewed within 24h: X / Y (Z%)
- Overdue items: X (list below)

### Overdue Review Items
| Item ID | Type | Owner | Days Overdue | Blocker |
|---------|------|-------|--------------|---------|
| ADR-XXX | ADR  | ENG-ARCH-02 | 5 | Awaiting Security WG feedback |

### Upcoming High-Priority Reviews
- [ ] ADR-YYY: New QoS algorithm (due Mon)
- [ ] Module spec: Telemetry Insights v2 (due Wed)

### Escalations
- None / [Describe any items requiring Chairs Sync attention]
```

### 3.2 Monthly Aggregate Dashboard
Operations WG maintains a Google Sheet / Grafana dashboard with:
- **Review Completion Rate:** Weekly trend graph per WG and aggregate
- **Bottleneck Analysis:** Which review type has lowest completion rate
- **Owner Performance:** Anonymous percentile distribution of review turnaround times
- **SLO Alignment:** Correlation between review delays and deployment SLO breaches

### 3.3 Automated Tracking Tools
**GitHub Actions Workflow:**
- Runs daily at 09:00 JST
- Parses `spec/notes/decision-log.md` for ADRs with status `提案中` older than 48h
- Posts summary to `#hl-wg-leads` Slack channel
- Tags relevant WG chairs

**Slackbot Commands:**
- `/honeylink-review-status [WG]` - Shows current review backlog for a WG
- `/honeylink-action-items [@user]` - Lists open action items for a user (role ID)
- `/honeylink-review-leaderboard` - Displays WGs by review completion rate (gamification)

---

## 4. Approval Criteria and Thresholds

### 4.1 Individual Item Approval
**Green (Ready to Proceed):**
- All checklist items are checked
- Required approvals are obtained (per review type above)
- No critical blockers

**Yellow (Needs Attention):**
- 1-2 checklist items missing, but non-critical
- Approval from 1 required reviewer pending
- Minor blockers identified with workaround

**Red (Blocked):**
- Critical checklist items missing (e.g., security review for crypto change)
- Multiple required approvals pending after 48h
- Major blocker with no workaround

### 4.2 Sprint/Weekly Thresholds
**Target:** ≥90% of items in Green state by sprint end

**Calculation:**
```
Review Completion Rate = (Green Items) / (Total Items) × 100%
```

**Alerts:**
- If rate drops below 85% mid-sprint: Chair sends reminder to all owners
- If rate drops below 80% at sprint end: Escalate to Governance Council
- If rate drops below 75% for 2 consecutive sprints: Initiate process improvement review (RCA)

---

## 5. Continuous Improvement

### 5.1 Retrospective Questions (Monthly)
- Which review types consistently miss 90% target? Why?
- Are review checklists too strict or too lenient?
- Do reviewers have enough time allocated (capacity planning)?
- Are automated reminders effective, or causing notification fatigue?

### 5.2 Adjustment Process
1. Operations WG collects feedback from all WGs
2. Proposes checklist refinements via ADR
3. Trial period (1 sprint) with revised checklist
4. Evaluate impact on completion rate and deliverable quality
5. Adopt or revert based on data

### 5.3 Success Criteria for This Checklist
- [ ] Review completion rate reaches ≥90% within 4 weeks of adoption
- [ ] Zero critical deliverables miss review deadlines for 8 consecutive weeks
- [ ] WGs report improved clarity on review expectations (qualitative feedback)
- [ ] Review bottleneck time reduces by ≥30% within 12 weeks

---

## 6. Enforcement and Accountability

### 6.1 WG Chair Responsibilities
- Monitor weekly status reports and dashboard metrics
- Send reminders for overdue reviews (automated + manual follow-up)
- Escalate persistent bottlenecks to Chairs Sync
- Report review completion rate in monthly roadmap review

### 6.2 Individual Reviewer Accountability
- Reviewers commit to 24h turnaround for assigned reviews
- If unavailable, reviewers must delegate to backup (defined in WG roster)
- Persistent delays (>3 missed deadlines per sprint) trigger 1-on-1 with chair to address capacity issues

### 6.3 Governance Council Oversight
- Quarterly audit of review completion trends
- Identify systemic issues (e.g., understaffed WG, unrealistic deadlines)
- Allocate resources (e.g., hire additional reviewers, reduce scope)

---

## 7. Integration with Other Processes

### 7.1 Links to Related Documents
- **Decision Log:** [spec/notes/decision-log.md](decision-log.md) - ADR review workflow
- **Governance Charter:** [spec/notes/governance.md](governance.md) - WG structure and escalation paths
- **Meeting Notes:** [spec/notes/meeting-notes.md](meeting-notes.md) - Action item tracking
- **Module Template:** [spec/templates/module-template.md](../templates/module-template.md) - Module spec DoD
- **Testing Metrics:** [spec/testing/metrics.md](../testing/metrics.md) - SLI/SLO alignment

### 7.2 Workflow Integration Points
- **Pre-Sprint Planning:** Review backlog is input to capacity planning
- **Daily Standups:** Reviewers report blocked reviews as impediments
- **Sprint Retrospectives:** Review completion rate is a standing agenda item
- **Monthly Roadmap Review:** Aggregate review metrics inform milestone risk assessment

---

## Appendix: Example Review Workflow

**Scenario:** New module spec for "Stream QoS Scheduler" is submitted

1. **Day 0 (Submission):**
   - Author creates PR with `spec/modules/qos-scheduler.md`
   - CI triggers Slackbot notification to `#hl-proto-wg` and `#hl-arch-wg`
   - Action item auto-created: "Review QoS Scheduler spec - due Day 2"

2. **Day 1 (Initial Review):**
   - Protocol WG reviewer checks API contract and state machine sections
   - Architecture WG reviewer verifies dependency alignment
   - Both leave inline comments in PR

3. **Day 2 (Cross-Review):**
   - Author addresses comments, pushes updates
   - Security WG reviewer confirms no crypto/auth implications
   - Operations WG reviewer confirms observability hooks are adequate

4. **Day 3 (Approval):**
   - All reviewers approve PR
   - WG chair merges PR and updates action item status to `Completed`
   - Review completion metric: 1/1 = 100% for this item

5. **Week End (Reporting):**
   - Protocol WG status report includes this item in "Specs reviewed this week: 1/1 (100%)"
