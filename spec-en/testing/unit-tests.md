# docs/testing/unit-tests.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines unit testing policy for HoneyLink™. Describes Rust-based testing frameworks and mock guidelines, excluding implementation code and C/C++ dependencies.

## Table of Contents
- [Testing Purpose and Scope](#testing-purpose-and-scope)
- [Test Design Principles](#test-design-principles)
- [Target Components and Priority](#target-components-and-priority)
- [Testing Tools and Dependencies](#testing-tools-and-dependencies)
- [Mock and Stub Policy](#mock-and-stub-policy)
- [Automation and Gates](#automation-and-gates)
- [Quality Metrics and Tracking](#quality-metrics-and-tracking)
- [Workflow](#workflow)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Testing Purpose and Scope
- Purpose: Verify correctness of core domain logic, crypto handlers, QoS schedulers, and config parsers.
- Out of Scope: Network boundaries, distributed algorithm consistency → handled in [docs/testing/integration-tests.md](integration-tests.md).
- Results aggregated in [docs/testing/metrics.md](metrics.md) to track SLO achievement.

## Test Design Principles
- Enforce AAA (Arrange-Act-Assert) structure in all tests.
- Introduce property-based testing for critical modules (e.g., Key exchange, topic routing).
- Utilize data-driven (table-driven) tests to cover boundary values and error cases.
- 1 test = 1 behavior. Even with multiple assertions, intent should be singular.

## Target Components and Priority
| Component | Priority | Target Coverage | Notes |
|-----------|----------|-----------------|-------|
| Crypto Key Management (Rust) | High | 95% | X25519/HKDF handlers |
| QoS Scheduler | High | 90% | Multi-queue decision logic |
| Config Parser | Medium | 85% | TOML/YAML parsing |
| Telemetry Normalization | Medium | 85% | Unit conversion and validation |
| Payload Serializer | Low | 80% | Compatibility verification |

- Coverage measured for both instruction and branch. Includes exception handling and log paths.

## Testing Tools and Dependencies
- Test Runner: Rust `cargo test` equivalent. C/C++-made test runners prohibited.
- Property-Based: `proptest` or `quickcheck` (Rust pure implementation).
- Coverage: Rust tools like `cargo-llvm-cov`. Generated reports in HTML + JSON.
- Static Analysis: Run Clippy, cargo-audit before unit tests and block diff.

## Mock and Stub Policy
- Place mocks at port/adapter boundaries. Use Trait-based mock generation crates (Rust).
- Fully stub network calls, do not use actual HTTP/QUIC networks.
- Inject time-dependent logic with `MockClock` and reset before/after tests.
- C/C++ bindings or FFI mocks prohibited.

## Automation and Gates
- Run unit tests per PR in CI pipeline ([docs/deployment/ci-cd.md](../deployment/ci-cd.md)).
- Integrate execution results with GitHub Checks. Block merge on failure.
- Speed up with caching (sccache). KPI: test time within 5 minutes.

## Quality Metrics and Tracking
- Coverage threshold: High priority modules ≥90%, others ≥80%.
- Flake rate: <1%. Upon exceeding, create stabilization task as highest priority.
- Defect detection rate: ≥70% of defects detected by unit tests before release.
- Performance metrics updated weekly in [docs/testing/metrics.md](metrics.md).

## Workflow
1. Test case design → Review (QA + Dev) → Implementation.
2. Attach test evidence (Coverage report, Clippy results) to Pull Request.
3. Re-run all tests upon main branch merge + archive results.
4. Supplement regression detection with nightly scheduled runs.

## Acceptance Criteria (DoD)
- Unit test scope, priority, and tools clearly documented.
- Mock/stub policy and automation gates defined.
- Quality metrics coordinated with test metrics document.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
