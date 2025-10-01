# docs/architecture/tech-stack.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Technology selection is pure specification and does not include any implementation code or C/C++-dependent components. Comparisons based on abstract capabilities and delivery formats.

## Table of Contents
- [Selection Policy](#selection-policy)
- [Technology Candidate Comparison Table](#technology-candidate-comparison-table)
- [Adopted Technologies and Rationale](#adopted-technologies-and-rationale)
- [Risks and Mitigation](#risks-and-mitigation)
- [Exit Strategy](#exit-strategy)
- [Operations and Security Evaluation](#operations-and-security-evaluation)
- [Performance Evaluation](#performance-evaluation)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Selection Policy
1. **C/C++ Independence:** Language runtime has garbage collection or memory safety mechanisms, sandboxed if native integration needed.
2. **Pluggable:** Define port/adapter abstractions for easy alternative implementation.
3. **Standards Compliance:** Cryptography and authentication comply with international standards and are third-party auditable.
4. **Operational Ease:** Metrics/trace mechanisms that integrate easily with observability stacks.
5. **Long-term Support:** Community/enterprise support expected for 5+ years.

## Technology Candidate Comparison Table
| Layer | Candidates | Pros | Cons | C/C++ Dependency Avoidance | Decision |
|-------|------------|------|------|---------------------------|----------|
| Language Runtime | Rust (pure), Kotlin/JVM, Go | Memory safety, async performance | Learning curve | Standard library only, FFI prohibited | **Adopt: Rust** |
| Cryptographic Library | RustCrypto Suite, WebCrypto Service | Security audited, latest algorithms | Limited RSA compatibility | Eliminate C dependency with SaaS | **Adopt: RustCrypto + SaaS fallback** |
| Messaging | NATS JetStream, Kafka (Managed) | Easy scaling | Kafka OSS has Java dependency | Use managed Kafka, internal abstraction | Conditionally adopted |
| Configuration Management | HashiCorp Consul (API use), Own YAML | API flexibility | OSS binary is Go | Use service version or cloud managed | **Adopt: Managed Consul API** |
| Observability | OpenTelemetry (OTLP), Prometheus SaaS | Standardized | C++ exporter in self-hosted | Use SaaS or Rust Exporter | **Adopt: OTLP + SaaS** |
| Data Store | CockroachDB (SQL), ScyllaDB (NoSQL) | Distributed fault tolerance, low latency | Complex operations | Fully managed version, use Rust client | **Adopt: CockroachDB (Managed)** |

## Adopted Technologies and Rationale
- **Language Runtime:** Assume Rust (standard toolchain only). Reasons: Memory safety, async performance, minimal C/C++ dependencies.
- **Cryptography:** RustCrypto + external KMS service. Provide X25519/ChaCha20-Poly1305 in pure implementation, achieve hardware acceleration via cloud KMS.
- **FEC:** Adopt Rust implementations of RaptorQ / Reed-Solomon as specification baseline. Libraries limited to pure implementations.
- **Observability:** Standardize OpenTelemetry OTLP output, use SaaS (e.g., Grafana Cloud) for dashboards.
- **Data Store:** CockroachDB for configuration data requiring procedural management, managed TimescaleDB for time-series metrics.

## Risks and Mitigation
| Risk | Content | Mitigation |
|------|---------|------------|
| Rust Ecosystem Maturity | Some protocol libraries immature | In-house specification of core parts, interoperability tests ([docs/testing/integration-tests.md](../testing/integration-tests.md)) |
| Managed Service Dependency | Unavailability due to regional regulations | Prepare alternative OSS + Rust agents, define switching procedures in operations SOP |
| High Performance Requirements | Optimization needed even with Rust | Document profiling plan at specification level in [docs/performance/benchmark.md](../performance/benchmark.md) |

## Exit Strategy
- If technology fails to meet KPIs, consider alternative stacks with the following steps:
  1. Record failure cause in [docs/notes/decision-log.md](../notes/decision-log.md).
  2. Add specification evaluation of candidate technologies (e.g., Erlang/Elixir, Java) and verify no C/C++ dependencies occur.
  3. Update compatibility layer (port/adapter) and maintain interface backward compatibility.

## Operations and Security Evaluation
- **Key Management:** Store cryptographic keys in cloud KMS (e.g., Hashicorp Vault SaaS), rotation policy referenced in [docs/security/encryption.md](../security/encryption.md).
- **Audit:** Verify audit log availability even when using SaaS, coordinate with [docs/security/vulnerability.md](../security/vulnerability.md).
- **Operational Load:** Eliminate C/C++ build environment by adopting managed services.

## Performance Evaluation
- Expect to meet latency targets with Rust + async runtime (Tokio equivalent). Detailed throughput model referenced in [docs/performance/scalability.md](../performance/scalability.md).
- FEC implementation computational load benchmarked in [docs/performance/benchmark.md](../performance/benchmark.md).

## Acceptance Criteria (DoD)
- Compare at least 2 options per layer, clearly stating adoption rationale and exit strategy.
- C/C++ dependency avoidance measures described for all candidates.
- Bidirectional references exist with security, operations, and performance documents.
- Risk mitigation consistent with roadmap (milestones).
- Specification update checklist can be reflected in [docs/templates/module-template.md](../templates/module-template.md).
