# docs/testing/integration-tests.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines integration testing strategy for HoneyLink™. Describes inter-service coordination and protocol compatibility verification methods, excluding implementation code and C/C++ dependencies.

## Table of Contents
- [Test Purpose and Scope](#test-purpose-and-scope)
- [Test Environment and Dependent Services](#test-environment-and-dependent-services)
- [Key Scenarios](#key-scenarios)
- [Data Management Policy](#data-management-policy)
- [Toolchain and Orchestration](#toolchain-and-orchestration)
- [Verification Points and Evaluation Criteria](#verification-points-and-evaluation-criteria)
- [Automation Pipeline](#automation-pipeline)
- [Risks and Fallback](#risks-and-fallback)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Test Purpose and Scope
- Purpose: Verify inter-service contracts, encrypted channels, QoS control, and device lifecycle consistency.
- Scope: Control-plane API, Data-plane Broker, Edge Gateway, Observability stack.
- Excluded: UI layer (separate [docs/ui/overview.md](../ui/overview.md)), fine-grained logic in [docs/testing/unit-tests.md](unit-tests.md).

## Test Environment and Dependent Services
- Reproduce staging environment in multi-AZ (3 AZ), adjust intra-region network latency to 10±2ms.
- Dependent services: Managed database, messaging bus, KMS. All built with IaC, no C/C++-made middleware adopted.
- Compare configuration diff with [docs/deployment/infrastructure.md](../deployment/infrastructure.md) before test execution to verify no drift.

## Key Scenarios
| Scenario | Purpose | Success Criteria |
|----------|---------|------------------|
| Secure Pairing Flow | Device registration ~ key distribution | 99.5% success rate, key rotation notification reaches broker |
| Telemetry Stream QoS | Bandwidth guarantee per priority | critical channel latency < 80ms, bulk latency < 500ms |
| Command Fan-out | Management portal → 10k devices | 95% delivery within 1 minute, error rate < 0.2% |
| OTA Coordinated Rollout | Batch delivery + rollback | Rollback success within 5 minutes on failure |
| Incident Telemetry Failover | Regional failure switchover | Failover to DR region within 2 minutes, zero data loss |

## Data Management Policy
- Generate test data synthetically, do not include PII/PHI.
- Capture before/after state snapshots and automatically evaluate differences.
- Execute cleanup job after test completion to delete residual topics and keys.

## Toolchain and Orchestration
- Orchestration: GitHub Actions + Rust-based Orchestrator CLI.
- Use WASM-sandboxed simulators to reproduce dependent service behavior. Simulators written in C/C++ prohibited.
- Containers use distroless base to minimize attack surface.

## Verification Points and Evaluation Criteria
- Contract verification: gRPC/protobuf schema compatibility inspection.
- Security: Verify no cases where mTLS handshake does not complete.
- Observability: Verify ≥95% of OpenTelemetry spans are aligned.
- State consistency: Verify event sourcing logs are unique and processed in correct order.

## Automation Pipeline
1. Pull Request → Run integration test job in nightly build.
2. Store results as artifacts (Junit XML, JSON, HTML dashboard).
3. Automatically generate draft entry in [docs/notes/decision-log.md](../notes/decision-log.md) on failure.
4. Proceed to next step (E2E) in [docs/deployment/ci-cd.md](../deployment/ci-cd.md) after success.

## Risks and Fallback
- Dependent service failure: Switch to synthetic mocks, label test results as "reference value."
- Environment drift: Immediately reapply IaC, allow retry up to once.
- Baseline not met: Submit RCA within 2 business days and set provisional block.

## Acceptance Criteria (DoD)
- Integration test scope, purpose, and scenarios comprehensively described.
- Environment definition and dependencies consistent with other documents.
- Automation pipeline and fallback procedures documented.
- C/C++ dependency exclusion explicitly stated.
- Success criteria and metrics integration defined.
