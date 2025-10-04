# Document Quality Definition of Done (DoD) Review Template

**Badges:** `🚫 実装コード非出力` `🚫 C/C++依存禁止`

> This template defines the quality criteria for achieving 100% DoD satisfaction across all HoneyLink documentation deliverables. It provides a comprehensive checklist and scoring system to objectively verify document completeness, accuracy, and adherence to project standards.

---

## 1. Purpose and Scope

**Purpose:** Ensure every specification, design document, and operational runbook meets the project's Definition of Done before being marked as complete.

**Scope:**
- Architecture specifications (`spec/architecture/*.md`)
- Module specifications (`spec/modules/*.md`)
- API documentation (`spec/api/*.md`)
- Security specifications (`spec/security/*.md`)
- Testing strategies (`spec/testing/*.md`)
- Deployment procedures (`spec/deployment/*.md`)
- UI/UX specifications (`spec/ui/*.md`)
- Meeting notes and governance documents (`spec/notes/*.md`)

**Out of Scope:** Implementation code, configuration files, CI/CD scripts (covered by separate code review process).

---

## 2. DoD Quality Dimensions

### 2.1 Completeness (40 points)
**Definition:** All mandatory sections are present and filled with substantive content (not placeholders or "TBD").

### 2.2 Accuracy (20 points)
**Definition:** Technical details are correct, links are valid, and references to external specs are up-to-date.

### 2.3 Clarity (15 points)
**Definition:** Language is precise, unambiguous, and accessible to the target audience (engineers, security reviewers, operations).

### 2.4 Consistency (10 points)
**Definition:** Document adheres to template structure, naming conventions, and cross-references align with other specs.

### 2.5 Actionability (10 points)
**Definition:** Acceptance criteria, implementation steps, and test strategies are concrete and testable.

### 2.6 Non-Negotiables (5 points)
**Definition:** Critical project constraints are met (C/C++ dependency exclusion, traceability to requirements, security review sign-off).

**Total Score:** 100 points  
**Passing Threshold:** ≥90 points (100% DoD satisfaction)

---

## 3. Review Checklist by Document Type

### 3.1 Architecture Specifications
**Applicable to:** `spec/architecture/overview.md`, `interfaces.md`, `dependencies.md`, `dataflow.md`, `tech-stack.md`

#### Completeness Checklist (40 points)
- [ ] (10 pts) **System Context Diagram:** High-level architecture with external actors, module boundaries, and data flows is present
- [ ] (10 pts) **Component Descriptions:** Each major component has a dedicated section with responsibilities, interfaces, and dependencies
- [ ] (10 pts) **Technology Choices:** Technology stack is documented with rationale for each choice (e.g., Rust for performance, no C/C++ for security)
- [ ] (10 pts) **Cross-References:** Links to related specs (`requirements.md`, `security/*.md`, `testing/*.md`) are present and valid

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **Dependency Graph:** Directed dependency graph is acyclic and matches `dependencies.md` layer rules (orchestration → services → platform → adapters)
- [ ] (5 pts) **Interface Contracts:** API schemas in `interfaces.md` are syntactically valid (e.g., JSON Schema, Rust type definitions)
- [ ] (5 pts) **SemVer Compliance:** Versioning strategy adheres to Semantic Versioning 2.0.0
- [ ] (5 pts) **Link Validation:** All hyperlinks (`[text](path)`) resolve to existing files or external URLs (verified by CI)

#### Clarity Checklist (15 points)
- [ ] (5 pts) **Audience-Appropriate:** Technical level matches target audience (e.g., no Rust implementation details in high-level overview)
- [ ] (5 pts) **Terminology Consistency:** Key terms (e.g., "session", "profile", "stream") are used consistently and defined in a glossary or first use
- [ ] (5 pts) **Diagram Legibility:** Diagrams have clear labels, legends, and are referenced in the text with explanations

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Template Adherence:** Follows the structure from `spec/templates/module-template.md` or architecture-specific template
- [ ] (5 pts) **Naming Conventions:** Component names match those in other documents (e.g., "Session Orchestrator" not "SessionOrch")

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Implementation Guidance:** Provides enough detail for engineers to start implementation without additional design sessions
- [ ] (5 pts) **Testability:** Describes how architecture decisions will be validated (e.g., integration tests for dependency injection)

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Exclusion:** Explicitly states no C/C++ dependencies are allowed and provides alternatives (Rust, pure TypeScript)
- [ ] (2 pts) **Security Review:** Has been reviewed by Security WG (sign-off comment in PR or ADR reference)
- [ ] (1 pt) **Version Control:** Document has a "Last Updated" date or Git commit reference

**Architecture Spec Scoring Example:**
- Completeness: 40/40 (all sections filled)
- Accuracy: 18/20 (1 broken link, -2 pts)
- Clarity: 15/15
- Consistency: 10/10
- Actionability: 8/10 (implementation steps too high-level, -2 pts)
- Non-Negotiables: 5/5
- **Total: 96/100** → **PASS (100% DoD)**

---

### 3.2 Module Specifications
**Applicable to:** `spec/modules/*.md` (e.g., `session-orchestrator.md`, `policy-profile-engine.md`)

#### Completeness Checklist (40 points)
- [ ] (5 pts) **Module Overview:** Name, team, purpose, and status (proposed/in-progress/production) are defined
- [ ] (5 pts) **Responsibilities:** Primary responsibilities and non-responsibilities are explicitly listed
- [ ] (5 pts) **Interface Definitions:** Input/output schemas with validation rules, protocols (gRPC/REST/async channels), and SLAs
- [ ] (5 pts) **Data Model:** Entity schemas, persistence strategy, retention policy, encryption requirements
- [ ] (5 pts) **Dependency Map:** Upstream and downstream dependencies with interface contracts and SLAs
- [ ] (5 pts) **Performance Requirements:** SLOs for latency, throughput, availability (e.g., P99 < 12ms, 99.9% uptime)
- [ ] (5 pts) **Security Considerations:** Authentication/authorization, threat model mitigations, data classification (PII/confidential/public)
- [ ] (5 pts) **Observability Hooks:** Metrics, logs, traces with formats and sampling strategies (OpenTelemetry)

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **Traceability Matrix:** All FR/NFR IDs from `spec/requirements.md` are mapped and validated
- [ ] (5 pts) **Interface Consistency:** Schemas match `spec/architecture/interfaces.md` standards (SemVer, `deprecated_after`, etc.)
- [ ] (5 pts) **Dependency Correctness:** Dependencies respect layer boundaries from `spec/architecture/dependencies.md` (no circular deps)
- [ ] (5 pts) **Performance Alignment:** SLOs are consistent with `spec/performance/benchmark.md` targets

#### Clarity Checklist (15 points)
- [ ] (5 pts) **State Machine Clarity:** State transitions are documented with ASCII diagrams or Mermaid, including error states
- [ ] (5 pts) **Error Handling:** Error cases are enumerated with HTTP status codes / error enums and recovery strategies
- [ ] (5 pts) **Example Scenarios:** At least 1 happy-path and 1 error-path example with sample data

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Template Adherence:** Follows `spec/templates/module-template.md` structure (12 sections)
- [ ] (5 pts) **Cross-Module Alignment:** Terminology and concepts align with related modules (e.g., "session" definition is consistent)

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Test Strategy:** Unit/integration/E2E test approach with coverage targets (≥80% line coverage)
- [ ] (5 pts) **Deployment Steps:** References `spec/deployment/ci-cd.md` and includes rollback procedure

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Exclusion:** No C/C++ dependencies; Rust/TypeScript alternatives are specified
- [ ] (2 pts) **Security Sign-Off:** Security WG has approved threat model and crypto choices
- [ ] (1 pt) **DoD Self-Assessment:** "受け入れ基準 (DoD)" section at the end confirms all criteria are met

---

### 3.3 API Documentation
**Applicable to:** ~~`spec/api/control-plane.md`~~ (Deleted - server-centric design), `spec/modules/*.md` (P2P modules)

#### Completeness Checklist (40 points)
- [ ] (8 pts) **Endpoint Inventory:** All endpoints listed with HTTP method, path, summary
- [ ] (8 pts) **Request/Response Schemas:** Full schemas with types, constraints, examples (JSON/Protobuf)
- [ ] (8 pts) **Error Codes:** Comprehensive error catalog (e.g., `ERR_VALIDATION`, `ERR_AUTH`) with HTTP status mapping
- [ ] (8 pts) **Authentication:** Auth mechanisms (mTLS, OAuth2/OIDC, API keys) and token lifecycle (5 min TTL)
- [ ] (8 pts) **Rate Limiting:** Rate limits, quotas, and backoff strategies

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **OpenAPI Compliance:** If OpenAPI spec exists, it matches narrative documentation
- [ ] (5 pts) **Example Validity:** cURL examples execute successfully against a test environment
- [ ] (5 pts) **SLA Correctness:** Response time SLOs match `spec/performance/benchmark.md`
- [ ] (5 pts) **Security Validation:** Security WG confirms authentication/authorization design is sound

#### Clarity Checklist (15 points)
- [ ] (5 pts) **API Design Principles:** RESTful conventions, idempotency, versioning strategy are explained
- [ ] (5 pts) **Change Management:** How breaking changes are handled (deprecation timeline, SemVer bumps)
- [ ] (5 pts) **Developer Experience:** Getting started guide, sandbox environment, troubleshooting tips

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Naming Conventions:** Consistent use of snake_case/camelCase, pluralization rules
- [ ] (5 pts) **Error Format:** All errors follow the same JSON structure (`{error_code, message, trace_id}`)

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Integration Examples:** Sample code in ≥2 languages (e.g., Rust, TypeScript) for key workflows
- [ ] (5 pts) **Testing Guide:** How to write integration tests against the API (mock server, test fixtures)

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Client Exclusion:** Official client libraries do not use C/C++ bindings
- [ ] (2 pts) **Audit Trail:** API references audit logging requirements from `spec/notes/decision-log.md`
- [ ] (1 pt) **Version Control:** API versioning strategy (e.g., `/v1/`, `/v2/`) is documented

---

### 3.4 Security Specifications
**Applicable to:** `spec/security/auth.md`, `encryption.md`, `key-management.md`, `vulnerability.md`

#### Completeness Checklist (40 points)
- [ ] (10 pts) **Threat Model:** STRIDE or equivalent analysis with identified threats and mitigations
- [ ] (10 pts) **Cryptographic Choices:** Algorithms (X25519, ChaCha20-Poly1305, HKDF-SHA512), key sizes, and justifications
- [ ] (10 pts) **Key Lifecycle:** Generation, rotation (90-day auto, 30-min emergency), storage (Vault/KMS), destruction procedures
- [ ] (10 pts) **Compliance Mapping:** GDPR, SOC2, ISO 27001 requirements and how they're met

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **Crypto Library Validation:** Specified libraries (e.g., RustCrypto) are audited and maintained
- [ ] (5 pts) **Attack Surface Analysis:** Potential vulnerabilities (e.g., timing attacks, side-channels) are addressed
- [ ] (5 pts) **Incident Response:** References `spec/deployment/rollback.md` for breach procedures
- [ ] (5 pts) **Third-Party Audit:** External security audit findings are incorporated (or planned)

#### Clarity Checklist (15 points)
- [ ] (5 pts) **Layered Explanation:** Content is accessible to both security experts and general engineers
- [ ] (5 pts) **Threat Scenarios:** Real-world attack examples and how the design defends against them
- [ ] (5 pts) **Key Hierarchy Diagram:** Visual representation of k_root → k_service → k_session → k_stream

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Cross-Spec Alignment:** Auth/encryption/key-management specs don't contradict each other
- [ ] (5 pts) **Terminology Precision:** Terms like "session key" vs "stream key" are used consistently

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Operational Runbooks:** Step-by-step guides for key rotation, breach response, access revocation
- [ ] (5 pts) **Secure Coding Guidelines:** References for developers (e.g., "Always use `zeroize` for keys in memory")

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Exclusion:** No C/C++ crypto libraries (e.g., no OpenSSL FFI); pure Rust alternatives
- [ ] (2 pts) **Security WG Sign-Off:** Principal Security Architect has approved the document
- [ ] (1 pt) **Vulnerability Disclosure:** Policy for reporting and patching vulnerabilities is defined

---

### 3.5 Testing Specifications
**Applicable to:** `spec/testing/unit-tests.md`, `integration-tests.md`, `e2e-tests.md`, `metrics.md`

#### Completeness Checklist (40 points)
- [ ] (10 pts) **Test Strategy:** Unit/integration/E2E scope, tooling (cargo test, Playwright), and coverage targets (≥80%)
- [ ] (10 pts) **Test Pyramid:** Distribution of tests across layers (70% unit, 20% integration, 10% E2E)
- [ ] (10 pts) **SLI/SLO Definitions:** Service-level indicators (latency, error rate) and objectives (P99 < 12ms, 99.9% uptime)
- [ ] (10 pts) **Metrics Catalog:** All collected metrics with units, aggregation methods, and alert thresholds

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **SLO Realism:** Targets are achievable based on `spec/performance/benchmark.md` results
- [ ] (5 pts) **Test Tooling Verification:** Listed tools (e.g., `cargo-llvm-cov`, `k6`) are installed and integrated in CI
- [ ] (5 pts) **Alerting Validation:** Alert rules (Yellow/Orange/Red) match `spec/testing/metrics.md` thresholds
- [ ] (5 pts) **Regression Detection:** Process for detecting performance regressions between releases

#### Clarity Checklist (15 points)
- [ ] (5 pts) **Test Case Examples:** At least 1 example per test type (unit, integration, E2E) with code snippets
- [ ] (5 pts) **Failure Triage Guide:** How to interpret test failures and escalate (owner, Slack channel)
- [ ] (5 pts) **Metrics Dashboard:** Link to live dashboard (Grafana) with screenshot/description

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Template Adherence:** Follows `spec/templates/test-template.md` structure
- [ ] (5 pts) **Cross-Module Alignment:** SLOs are consistent across all modules (e.g., Session Orchestrator and QoS Scheduler both target 99.9% uptime)

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Test Automation:** All tests run in CI pipeline; manual test steps are minimized
- [ ] (5 pts) **Performance Benchmarking:** Repeatable benchmark suite with historical trend tracking

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Tool Exclusion:** Test tools do not depend on C/C++ binaries (e.g., use pure Rust/TypeScript test runners)
- [ ] (2 pts) **Coverage Enforcement:** CI blocks merges if coverage drops below 80%
- [ ] (1 pt) **Chaos Engineering:** Failure injection tests (e.g., network partitions) are included in E2E suite

---

### 3.6 Deployment & Operations
**Applicable to:** `spec/deployment/ci-cd.md`, `infrastructure.md`, `rollback.md`

#### Completeness Checklist (40 points)
- [ ] (10 pts) **CI/CD Pipeline:** Build, test, security scan, deploy stages with tooling (GitHub Actions, ArgoCD)
- [ ] (10 pts) **Infrastructure as Code:** IaC tool (Terraform/Bicep), resource definitions, environment configs (dev/staging/prod)
- [ ] (10 pts) **Rollback Procedures:** Automated rollback triggers (SLO breach), manual rollback steps, RTO/RPO targets
- [ ] (10 pts) **DR Plan:** Disaster recovery strategy, backup schedules, restore procedures, drill cadence (quarterly)

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **IaC Validation:** Terraform plan output is reviewed and matches expected infrastructure
- [ ] (5 pts) **Deployment Verification:** Post-deploy smoke tests confirm service health
- [ ] (5 pts) **Rollback Testing:** Rollback procedure has been tested in staging (documented date)
- [ ] (5 pts) **SLO Correlation:** Deployment metrics align with `spec/testing/metrics.md` SLOs

#### Clarity Checklist (15 points)
- [ ] (5 pts) **Runbook Precision:** Step-by-step instructions with screenshots/command examples
- [ ] (5 pts) **Decision Trees:** Flowcharts for when to rollback vs. hotfix vs. roll forward
- [ ] (5 pts) **On-Call Guidance:** Escalation paths, contact info (PagerDuty schedule), incident severity classification

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Environment Parity:** Dev/staging/prod configs differ only in resource sizes and secrets
- [ ] (5 pts) **Naming Conventions:** Resource names follow standard pattern (e.g., `hl-prod-session-orch-db`)

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Automated Gates:** Deployment pipeline has automated quality gates (tests pass, no CVEs)
- [ ] (5 pts) **Post-Mortem Template:** Incident post-mortem process is defined with timeline for completion (72h)

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Build Exclusion:** Build process does not compile C/C++ code
- [ ] (2 pts) **Secret Management:** Secrets are stored in Vault/KMS, never in Git or plaintext configs
- [ ] (1 pt) **Audit Logging:** Deployment events are logged to immutable audit trail

---

### 3.7 UI/UX Specifications
**Applicable to:** `spec/ui/overview.md`, `wireframes.md`, `visual-design.md`, `animations.md`, `accessibility.md`

#### Completeness Checklist (40 points)
- [ ] (10 pts) **Wireframes:** All 5 screens (device list, pairing, dashboard, policy builder, metrics hub) with mobile/tablet/desktop variants
- [ ] (10 pts) **Design Tokens:** Colors, typography, spacing, shadows defined and exported for Tailwind CSS
- [ ] (10 pts) **Accessibility Annotations:** ARIA labels, keyboard shortcuts, focus order documented
- [ ] (10 pts) **Animation Specs:** Duration, easing, trigger conditions, `prefers-reduced-motion` fallbacks

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **WCAG 2.2 AA Compliance:** Contrast ratios ≥4.5:1, keyboard navigable, screen reader compatible (verified by axe-core)
- [ ] (5 pts) **Responsive Breakpoints:** Layouts tested at 375px (mobile), 768px (tablet), 1920px (desktop)
- [ ] (5 pts) **i18n Readiness:** Text strings are externalized, 30% expansion accounted for in layouts
- [ ] (5 pts) **Design-Dev Handoff:** Figma file version matches documented spec version

#### Clarity Checklist (15 points)
- [ ] (5 pts) **User Flows:** Task flows (e.g., "Pair a new device") with decision points and error states
- [ ] (5 pts) **Component Library:** Reusable components (Button, Card, Modal) documented with props and variants
- [ ] (5 pts) **Visual Examples:** Screenshots or Figma embeds for each wireframe

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Design System Adherence:** All screens use defined tokens; no ad-hoc color/spacing values
- [ ] (5 pts) **Brand Alignment:** "Honey" theme (warmth, softness) is evident in color palette and imagery

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Developer Assets:** SVG/PNG exports, icon library, component code snippets (React/TypeScript)
- [ ] (5 pts) **Prototype:** Interactive Figma prototype or deployed Storybook for stakeholder review

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **C/C++ Front-End Exclusion:** No native C/C++ UI frameworks (Qt, GTK); pure web tech (React/TypeScript)
- [ ] (2 pts) **Accessibility Sign-Off:** UX accessibility specialist has approved all screens
- [ ] (1 pt) **Localization:** English and Japanese translations are complete; Spanish and Chinese are planned

---

### 3.8 Meeting Notes & Governance
**Applicable to:** `spec/notes/meeting-notes.md`, `governance.md`, `decision-log.md`, `attendance-system.md`

#### Completeness Checklist (40 points)
- [ ] (10 pts) **Attendee List:** Role IDs for all attendees (no personal data), apologies for absences
- [ ] (10 pts) **Agenda:** Topics discussed with time allocations and owners
- [ ] (10 pts) **Action Items:** Each action has ID, description, owner, due date, status
- [ ] (10 pts) **Decisions & Risks:** Key decisions with ADR references, risks with severity and mitigation

#### Accuracy Checklist (20 points)
- [ ] (5 pts) **Action Traceability:** Action IDs link to action register and close within sprint (90% target)
- [ ] (5 pts) **ADR Cross-Reference:** Decisions mentioned in notes have corresponding ADR entries in `decision-log.md`
- [ ] (5 pts) **Date/Time Accuracy:** Meeting date, start/end times are correct and in JST timezone
- [ ] (5 pts) **Follow-Up Verification:** Previous meeting's action items are reviewed for closure

#### Clarity Checklist (15 points)
- [ ] (5 pts) **Summary Quality:** Each agenda item has a brief summary of discussion and outcome
- [ ] (5 pts) **Disambiguation:** Technical acronyms are spelled out on first use (e.g., "QoS (Quality of Service)")
- [ ] (5 pts) **Parking Lot:** Off-topic items are noted for future discussion, not lost

#### Consistency Checklist (10 points)
- [ ] (5 pts) **Template Adherence:** Follows `spec/notes/meeting-notes.md` template structure
- [ ] (5 pts) **Tone Neutrality:** Notes are factual, avoid subjective language ("X disagreed" → "X raised concerns about Y")

#### Actionability Checklist (10 points)
- [ ] (5 pts) **Action Item Specificity:** Actions are SMART (Specific, Measurable, Achievable, Relevant, Time-bound)
- [ ] (5 pts) **Next Steps:** Clear agenda for next meeting or handoff to another WG

#### Non-Negotiables Checklist (5 points)
- [ ] (2 pts) **Attendance Tracking:** Attendance rate ≥90% per `spec/notes/attendance-system.md` enforcement
- [ ] (2 pts) **72h Publishing:** Notes are published within 72 hours of meeting (monitored by Slackbot)
- [ ] (1 pt) **PII Exclusion:** No personal email addresses or phone numbers in notes (use role IDs)

---

## 4. Scoring and Verification Process

### 4.1 Self-Assessment (Mandatory for All Documents)
**When:** Before submitting a document for review (PR creation)  
**Who:** Document author  
**How:**
1. Fill out this template for the appropriate document type (e.g., Module Specification checklist)
2. Assign points honestly; mark items as `[ ]` (incomplete) or `[x]` (complete)
3. Calculate total score and verify ≥90 points
4. Attach completed checklist to PR description

**Example Self-Assessment Output:**
```markdown
## DoD Self-Assessment: Session Orchestrator Module Spec

- Completeness: 38/40 (missing 1 dependency example, -2 pts)
- Accuracy: 20/20
- Clarity: 13/15 (error handling examples too terse, -2 pts)
- Consistency: 10/10
- Actionability: 10/10
- Non-Negotiables: 5/5

**Total Score: 96/100 → PASS (100% DoD Satisfied)**

Remaining work before final approval:
- Add 1 upstream dependency example in Section 5
- Expand error handling examples in Section 3.2
```

### 4.2 Peer Review (Mandatory for All Documents)
**When:** During PR review process  
**Who:** At least 1 domain expert from the relevant WG  
**How:**
1. Reviewer independently scores the document using the same checklist
2. If reviewer's score differs by ≥10 points from author's self-assessment, discuss discrepancies
3. Reviewer leaves inline comments on specific checklist items that need improvement
4. Reviewer approves PR only if score ≥90 points

### 4.3 WG Chair Final Approval (Mandatory for High-Impact Documents)
**Applicable to:** Architecture specs, security specs, API docs, any document with cross-WG impact  
**Who:** WG chair or designated senior reviewer  
**How:**
1. Chair reviews both author self-assessment and peer review scores
2. Chair spot-checks 3-5 random checklist items for accuracy
3. Chair confirms all non-negotiables (C/C++ exclusion, security sign-off, etc.) are met
4. Chair merges PR and updates `decision-log.md` if applicable

### 4.4 Automated Validation (CI/CD Integration)
**Implemented in:** GitHub Actions workflow `.github/workflows/doc-quality-gate.yml`  
**Checks:**
- [ ] All hyperlinks are valid (no 404s)
- [ ] All referenced files exist in the repo
- [ ] Document has "Last Updated" date within 90 days (stale flag)
- [ ] No placeholder text (e.g., "TBD", "TODO", "FIXME") in final docs
- [ ] Markdown linting passes (markdownlint rules)
- [ ] Spell check passes (cspell with custom dictionary)

**Action on Failure:** PR cannot be merged until issues are resolved.

---

## 5. Continuous Improvement of DoD Criteria

### 5.1 Quarterly Review of Checklist
**When:** First Thursday of each quarter (Jan/Apr/Jul/Oct)  
**Who:** Operations WG + all WG chairs  
**Process:**
1. Analyze document rejection rate (PRs that failed ≥90 point threshold)
2. Identify checklist items that are frequently misunderstood or contentious
3. Propose refinements via ADR
4. Trial refined checklist for 1 sprint, gather feedback
5. Adopt or revert based on review completion rate and quality outcomes

### 5.2 Feedback Collection
**Method:** Google Form linked in PR template for optional feedback  
**Questions:**
- Was the checklist clear and actionable?
- Which checklist items were hardest to satisfy? Why?
- Are there missing quality dimensions not covered?
- How long did self-assessment take? (target: ≤30 min)

### 5.3 Success Metrics
- [ ] 100% of merged documents score ≥90 points
- [ ] Review rejection rate (score <90) decreases by 50% within 6 months
- [ ] Average self-assessment time ≤30 minutes
- [ ] Zero post-merge quality issues requiring significant rework

---

## 6. Integration with Existing Processes

### 6.1 Links to Related Documents
- **Review Completion Checklist:** [spec/notes/review-completion-checklist.md](review-completion-checklist.md) - Tracks review timeliness (90% in 48h)
- **Decision Log:** [spec/notes/decision-log.md](decision-log.md) - ADR workflow and 72h completion rule
- **Governance Charter:** [spec/notes/governance.md](governance.md) - WG responsibilities and escalation paths
- **Module Template:** [spec/templates/module-template.md](../templates/module-template.md) - Standard structure for module specs

### 6.2 Workflow Integration
**Step 1: Draft Creation** → Author fills out template, performs self-assessment  
**Step 2: PR Submission** → Attach DoD self-assessment to PR description  
**Step 3: CI Validation** → Automated checks run (link validation, linting, spell check)  
**Step 4: Peer Review** → Domain expert scores independently, leaves feedback  
**Step 5: Revisions** → Author addresses feedback, updates self-assessment  
**Step 6: Chair Approval** → WG chair verifies ≥90 points, merges PR  
**Step 7: Post-Merge Audit** → Operations WG samples 10% of merged docs monthly for quality drift

---

## 7. Example: Completed DoD Review for a Module Spec

**Document:** `spec/modules/qos-scheduler.md`  
**Author:** ENG-PROTO-03 (QoS Scheduler Partner)  
**Reviewer:** ENG-ARCH-03 (Observability Architect)  
**Review Date:** 2025-10-01

### Author Self-Assessment
| Dimension | Score | Notes |
|-----------|-------|-------|
| Completeness | 40/40 | All sections filled, including test strategy and deployment steps |
| Accuracy | 18/20 | One FR ID typo (FR-05 should be FR-06), fixed in revision |
| Clarity | 15/15 | State machine diagram is clear, error handling examples provided |
| Consistency | 10/10 | Follows module template, terminology matches other modules |
| Actionability | 10/10 | Test coverage targets and rollback steps are concrete |
| Non-Negotiables | 5/5 | No C/C++ deps, Security WG approved, DoD section present |
| **Total** | **98/100** | **PASS** |

### Peer Review (ENG-ARCH-03)
| Dimension | Score | Notes |
|-----------|-------|-------|
| Completeness | 40/40 | Confirmed all sections are substantive |
| Accuracy | 20/20 | FR ID typo was corrected; verified against requirements.md |
| Clarity | 14/15 | One error example could use more detail (-1 pt) |
| Consistency | 10/10 | Naming and structure align with template |
| Actionability | 10/10 | Implementation guidance is sufficient |
| Non-Negotiables | 5/5 | C/C++ exclusion confirmed, Security WG sign-off verified |
| **Total** | **99/100** | **PASS** |

**Reviewer Comments:**
- "Excellent work! Only minor suggestion: expand the 'Buffer Overflow' error example in Section 3.2 with a recovery strategy."
- "Dependency graph correctly shows no circular dependencies."

**Chair Decision:** Approved and merged. Action item created to enhance error example in next revision (low priority).

---

## 8. Appendix: Quick Reference Scorecard

| Document Type | Completeness | Accuracy | Clarity | Consistency | Actionability | Non-Negotiables | Total |
|---------------|--------------|----------|---------|-------------|---------------|-----------------|-------|
| Architecture Spec | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| Module Spec | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| API Documentation | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| Security Spec | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| Testing Spec | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| Deployment Spec | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| UI/UX Spec | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |
| Meeting Notes | 40 pts | 20 pts | 15 pts | 10 pts | 10 pts | 5 pts | 100 pts |

**Passing Threshold:** ≥90 points for all document types = **100% DoD Satisfaction**

---

## 9. Document Control

**Version:** 1.0  
**Last Updated:** 2025-10-01  
**Owner:** Operations WG (OPS-LEAD-01)  
**Approval:** Governance Council (Arch + Sec + Ops Chairs)  
**Next Review:** 2026-01-01 (Quarterly)

**Change Log:**
| Date | Change | Author |
|------|--------|--------|
| 2025-10-01 | Initial version created | OPS-LEAD-01 |
