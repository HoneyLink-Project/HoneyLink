# docs/notes/decision-log.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Provides Architecture Decision Record (ADR) template for HoneyLink™. Manages version control of technical and operational important decisions, without including C/C++ dependencies or implementation code.

---

## Usage Guide
- Assign ID to decisions and specify component and impact scope.
- Use status: `Proposed / Approved / Rejected / Superseded`.
- Critical decisions coordinate with [docs/roadmap.md](../roadmap.md), [docs/security/vulnerability.md](../security/vulnerability.md), etc.
- List affected documents in `Related Docs`.

## ADR List
| ADR ID | Title | Date | Status | Primary Domain | Summary |
|--------|-------|------|--------|----------------|---------|
| ADR-001 | <!-- e.g., HoneyLink Protocol Cipher Suite Selection --> | <!-- YYYY-MM-DD --> | Proposed | Security | <!-- One-line summary --> |
| ADR-002 | | | | | |

## ADR Template
```markdown
## ADR-XXX: Title
- **Date:** YYYY-MM-DD
- **Status:** Proposed / Approved / Rejected / Superseded
- **Owner:** Name/Team
- **Domain:** Architecture / Security / Operations / Product
- **Related KPI/SLO:** <!-- docs/testing/metrics.md, etc. -->
- **Related Docs:** [docs/...](...)

### Background
- Issues or opportunities
- Current constraints

### Decision
- Adopted approach
- Rationale
- Justification for avoiding C/C++ dependencies

### Alternatives Comparison
| Alternative | Pros | Cons | Risks |
|-------------|------|------|-------|
| Option A | | | |
| Option B | | | |

### Impact
- Technical impact
- Organizational impact
- Cost/Schedule impact

### Implementation Plan
1. Step 1
2. Step 2
3. Completion criteria

### Follow-up
- Evaluation timing
- Metrics review (e.g., [docs/testing/metrics.md](../testing/metrics.md))
- Next actions
```

## Change History
| Date | Changes | Editor |
|------|---------|--------|
| YYYY-MM-DD | Initial version | Name |
| YYYY-MM-DD | ADR-XYZ added | Name |

---

## Operational Procedures

### 1. Standardized Record Formats

#### 1.1 Design Change Record
**Format:** ADR (Architecture Decision Record)  
**Required Fields:**
- ADR ID (sequential number)
- Title, Date, Status, Owner
- Background: Reason for change, current constraints
- Decision: Adopted approach, rationale for C/C++ dependency exclusion
- Alternatives: Compare at least 2 options in table format
- Impact: Technical/Organizational/Cost/Schedule impacts
- Implementation Plan: Including completion criteria
- Follow-up: Evaluation timing, metrics review links

#### 1.2 Risk Record
**Format:** Within ADR or independent risk entry  
**Required Fields:**
- Risk ID (e.g., `RISK-ADR-XXX-001`)
- Risk Category (Technical/Security/Schedule/Compliance)
- Probability (Low/Medium/High) × Impact (Low/Medium/High)
- Mitigation Plan: Specific actions, responsible party, deadline
- Monitoring Method: Observation metrics, thresholds, alert settings
- Links to related SLO/KPI ([spec/testing/metrics.md](../testing/metrics.md))

#### 1.3 RCA (Root Cause Analysis) Record
**Format:** Independent RCA section or append to related ADR  
**Required Fields:**
- Incident ID, Occurrence date/time, Detection method
- Timeline: Occurrence → Detection → Initial response → Root resolution
- Root Cause: Summary of 5 Whys or Fishbone diagram
- Corrective Actions: Short-term response (within 24h) + permanent fix (within 90 days)
- Preventive Measures: Process improvements, automation, monitoring enhancements
- Lessons Learned: Items for horizontal deployment to other teams/projects
- Verified: Date and metrics confirming no recurrence

### 2. 72-Hour Recording Enforcement

#### 2.1 Recording Completion Timeline
- **Within 24 hours of decision:** Create draft (Status: `Proposed`)
- **Within 48 hours of decision:** Request cross-review (notify affected WGs)
- **Within 72 hours of decision:** Complete final review, update status (`Approved` / `Rejected`)

#### 2.2 Automated Reminder System
**Implementation Methods:**
1. **Slackbot Integration:**
   - Post to WG channel on new ADR creation (e.g., `#hl-arch-wg`)
   - 24 hours later: DM owner if draft incomplete
   - 48 hours later: Notify related WG chairs if cross-review incomplete
   - 72 hours later: Escalate to Governance Council (`#hl-wg-leads`) if unapproved
2. **CI/CD Hook:**
   - Detect changes to `spec/notes/decision-log.md` with Git pre-commit hook
   - Parse ADR ID date field, check for 72-hour expiration
   - Display warning in CI if expired ADRs exist
3. **Weekly Summary:**
   - Auto-post incomplete ADR list to `#hl-wg-leads` every Friday 17:00 JST
   - List includes ADR ID, Owner, Elapsed days, Status

#### 2.3 Reminder Exclusion Conditions
- Awaiting legal review: Set status to `Legal Review In Progress`, pause deadline
- External dependency pending: Use `Blocked` status, specify blocker

### 3. Approval Flow and Escalation Path

#### 3.1 Standard Approval Flow
1. **Proposal Creation:**
   - Owner: Create ADR draft and post to own WG channel
   - Deadline: Within 24 hours of decision
2. **Cross-Review:**
   - Related WGs: Review from technical/security/operational perspectives
   - Required reviewers:
     - Architecture WG: Dependencies, backward compatibility
     - Security WG: C/C++ dependency exclusion, crypto/auth impacts
     - Operations WG: Deployability, SLO impacts
   - Deadline: 24 hours (within 48 hours of proposal posting)
3. **Approval Decision:**
   - WG Chair: Integrate review comments and make final judgment
   - Status update: `Approved` / `Rejected` / `Conditionally Approved` (specify improvements)
   - Deadline: 24 hours after cross-review (within 72 hours of proposal posting)
4. **Implementation Tracking:**
   - Operations WG: Add approved ADR implementation actions to action register
   - Progress check: Report at weekly WG meetings

#### 3.2 Escalation Path
**Level 1: WG Internal Resolution (within 24 hours)**
- Trigger: Draft creation delay, minor conflicts in review opinions
- Response: WG chair coordinates, provides guidance to owner

**Level 2: Cross-WG Coordination (within 48 hours)**
- Trigger: Conflicts between multiple WGs, difficult technical trade-off decisions
- Response: Discuss at Chairs Sync (bi-weekly Monday 17:00)
- Output: Add coordination memo to decision-log.md

**Level 3: Governance Council Decision (within 72 hours)**
- Trigger: No agreement at Chairs Sync, critical security/compliance risks
- Response: Arch + Sec + Ops chairs discuss in Matrix secure room (`!govcouncil:honeylink.local`)
- Final decision: Vote (majority decides), dissenting opinions recorded in minutes

**Level 4: Executive Escalation (beyond 72 hours)**
- Trigger: No agreement at Governance Council, budget/legal judgment needed
- Response: Escalate to CTO/CISO, request legal review
- Deadline: Additional 5 business days

#### 3.3 Fast-Track Approval (Emergency Process)
**Applicable Conditions:** Security incident response, critical production failures
**Procedure:**
1. Incident Commander posts emergency ADR to `#hl-incident`
2. Security WG / Operations WG chairs review immediately (within 2 hours)
3. Obtain verbal approval, supplement formal ADR afterward (within 24 hours)
4. Record in decision-log.md along with RCA (within 72 hours)

### 4. Quality Assurance and Auditing

#### 4.1 ADR Quality Checklist
- [ ] ADR ID is unique and sequential
- [ ] Title clearly expresses the change content
- [ ] Background section documents issues/constraints
- [ ] Alternatives comparison table includes at least 2 options
- [ ] Rationale for C/C++ dependency exclusion is stated
- [ ] Impact analysis (technical/organizational/cost) is complete
- [ ] Implementation plan defines completion criteria
- [ ] Links to related documents are valid
- [ ] Owner and date are accurate

#### 4.2 Periodic Audits
- **Monthly Audit:**
  - Operations WG confirms status of all ADRs
  - Track incomplete implementation plans
  - Investigate old `Proposed` status (over 30 days)
- **Quarterly Audit:**
  - Security WG reviews all ADRs from compliance perspective
  - Verify effectiveness of implemented ADRs (cross-check with SLO/KPI)
  - Confirm horizontal deployment status of lessons learned

---

### Operational Notes
- Critical security/compliance decisions require legal review.
- When superseding old ADRs, update old ADR status to `Superseded`.
- Link with meeting notes ([docs/notes/meeting-notes.md](meeting-notes.md)) to ensure traceability.
- Changes to this document itself are recorded as ADRs and version controlled.
