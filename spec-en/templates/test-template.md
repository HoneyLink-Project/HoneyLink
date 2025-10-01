# docs/templates/test-template.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Template for HoneyLink™ test plan. Describes test purpose, scope, environment, and metrics in a unified format.

---

## 1. Basic Information
- **Test Name:** <!-- e.g., HoneyLink vX.Y Release Regression Test -->
- **Version/Build:** <!-- e.g., build-2025-09-29 -->
- **Test Owner:** <!-- Team/Person responsible -->
- **Related Documents:** [docs/testing/metrics.md](../testing/metrics.md), [docs/deployment/ci-cd.md](../deployment/ci-cd.md)

## 2. Purpose and Scope
- **Test Purpose:** <!-- e.g., Defect detection, Regression verification -->
- **In-Scope:** <!-- System/Feature/Component to be tested -->
- **Out-of-Scope:** <!-- Areas not tested and rationale -->

## 3. Test Scope Details
| Item | Description | Related Requirements |
|------|-------------|----------------------|
| Functional Requirements | <!-- --> | [docs/requirements.md](../requirements.md) |
| Non-Functional Requirements | <!-- --> | [docs/performance/scalability.md](../performance/scalability.md) etc |
| Security | <!-- --> | [docs/security/vulnerability.md](../security/vulnerability.md) |

## 4. Test Types and Approach
| Test Type | Purpose | Approach | Reference |
|-----------|---------|----------|-----------|
| Unit | | | [docs/testing/unit-tests.md](../testing/unit-tests.md) |
| Integration | | | [docs/testing/integration-tests.md](../testing/integration-tests.md) |
| E2E | | | [docs/testing/e2e-tests.md](../testing/e2e-tests.md) |
| Performance | | | [docs/performance/benchmark.md](../performance/benchmark.md) |
| Security | | | [docs/security/auth.md](../security/auth.md) |

## 5. Test Environment
- **Infrastructure:** <!-- Environment name, Region, Scale -->
- **Configuration Management:** [docs/deployment/infrastructure.md](../deployment/infrastructure.md)
- **Data:** <!-- Test data generation/anonymization policy -->
- **Constraints:** <!-- Maintenance windows, etc. -->

## 6. Tools and Dependencies
- **Test Tools:** <!-- Rust-based test runners, etc. -->
- **Reporting:** <!-- JUnit XML, HTML, etc. -->
- **Prohibitions:** Do not use C/C++-made tools/libraries.

## 7. Schedule
| Phase | Duration | Owner |
|-------|----------|-------|
| Preparation | | |
| Execution | | |
| Result Review | | |
| Follow-up | | |

## 8. Resources and Roles
| Role | Personnel | Responsibilities |
|------|-----------|------------------|
| QA Lead | | Plan development, Progress management |
| Test Engineer | | Execution, Reporting |
| SRE | | Environment provisioning |
| Product | | Acceptance determination |

## 9. Risks and Mitigation
- **Risk:** <!-- e.g., Test data preparation delay -->
- **Mitigation:** <!-- Alternative data/auto-generation, etc. -->
- **Residual Risk:** <!-- Acceptable level -->

## 10. Metrics and Evaluation Criteria
| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Test Progress Rate | ≥ 95% | Completed test cases / Total test cases |
| Defect Detection Rate | ≥ 70% | QA-detected defects / Total defects |
| Defect Density | ≤ 0.3 | Defects / Function points |
| Escalation Response Time | ≤ 2h | Incident log |

## 11. Entry/Exit Criteria
- **Start Conditions:** <!-- Design review complete, Environment ready, etc. -->
- **Exit Conditions:** <!-- Zero critical defects, Test cases complete, SLO achieved, etc. -->
- **Sign-off:** <!-- Approvers -->

## 12. Communication Plan
- **Status Updates:** <!-- Daily standup/reports -->
- **Report Distribution:** <!-- Slack/Email/Confluence -->
- **Post-Analysis:** Record in [docs/notes/decision-log.md](../notes/decision-log.md).

## 13. Attachments List
- Test case inventory
- Test data dictionary
- Risk review results

## 14. Acceptance Criteria (DoD)
- Test purpose, scope, environment, and metrics are clearly defined.
- Links to related documents are consistent.
- C/C++ dependency exclusion is explicitly stated.
- Risks and evaluation criteria are defined, and sign-off conditions are set.
