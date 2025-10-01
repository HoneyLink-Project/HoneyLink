# docs/architecture/dataflow.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> This document abstractly defines HoneyLink™'s dataflow and processing flow. Does not include any implementation code, CLI commands, or C/C++-dependent protocol stacks.

## Table of Contents
- [Overall Flow Overview](#overall-flow-overview)
- [Sequence: Initial Pairing](#sequence-initial-pairing)
- [Sequence: Multi-Stream Control](#sequence-multi-stream-control)
- [Sequence: IoT Power-Saving Mode](#sequence-iot-power-saving-mode)
- [Data Consistency and Idempotency](#data-consistency-and-idempotency)
- [Transactions and Retry Strategy](#transactions-and-retry-strategy)
- [Monitoring Points and Telemetry](#monitoring-points-and-telemetry)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Overall Flow Overview
```
[Beacon Broadcast]
        |
        v
[Session Orchestrator] ---(Handshake Events)---> [Crypto & Trust]
        |
        v
[Policy Engine] ---(Profile Decision)---> [QoS Scheduler]
        |
        +---> [Telemetry & Insights]
        |
        v
[Transport Abstraction] ---(Packets)---> [Physical Adapter]
```

## Sequence: Initial Pairing
1. **Beacon Detection:** Device transmits Beacon messages (public key hash + Capability) at 100ms intervals.
2. **Connection Request:** Client sends connection request event to Session Orchestrator.
3. **Key Exchange:** Crypto & Trust Anchor generates shared secret via X25519.
4. **Profile Negotiation:** Policy Engine calculates profile candidates from client request and environmental conditions.
5. **Confirmation Notification:** Session Orchestrator notifies finalized profile and session ID.
6. **Telemetry Registration:** Telemetry & Insights generates initial session metrics.

```
Client        Orchestrator     Crypto       Policy       Telemetry
  |                |             |             |             |
  |---Beacon------>|             |             |             |
  |---Connect Req--------------->|             |             |
  |                |---Key Req-->|             |             |
  |                |<--Key Ack---|             |             |
  |                |---Profile Req----------->|             |
  |                |<--Profile Resp-----------|             |
  |<--Session Ack--|             |             |---Init----->|
```

## Sequence: Multi-Stream Control
1. Stream QoS Scheduler queues main streams in priority order.
2. Transport Abstraction determines FEC coefficients and batch sizes per stream.
3. Pull network status (RTT/loss rate) from Telemetry & Insights.
4. If policy change is needed, send event notification to Policy Engine.
5. Session Orchestrator distributes update information to each client.

## Sequence: IoT Power-Saving Mode
1. IoT device sends "power-saving heartbeat" at 10-minute intervals.
2. Policy Engine re-evaluates power budget and adjusts transmission frequency and batch transfer time slots.
3. Telemetry & Insights indexes power-saving effectiveness and updates SLI (average current).
4. Switch to batch mode defined in [docs/performance/scalability.md](../performance/scalability.md) if necessary.

## Data Consistency and Idempotency
- **Session Management:** Session IDs use time-ordered formats like UUIDv7. Attach idempotency-key during retransmission to prevent duplicate processing.
- **Stream Control:** QoS updates are versioned differential events. Apply only the latest version.
- **Telemetry:** Time-series data is append-only. Aggregation processing performed by idempotent aggregator.

## Transactions and Retry Strategy
| Operation | Transaction Boundary | Retry Strategy | Backpressure |
|-----------|---------------------|----------------|--------------|
| Handshake | Session Orchestrator + Crypto | Exponential backoff (max 3 times) | Continue Beacon monitoring during retry interval |
| Profile Update | Policy Engine | Sequential re-evaluation, max 2 times | QoS Scheduler maintains old settings |
| Telemetry Export | Telemetry & Insights | Persistent queue, dead letter | Coarsen summary granularity during acquisition delay |

## Monitoring Points and Telemetry
- **Handshake Latency:** Primary SLI. Refer to [docs/deployment/rollback.md](../deployment/rollback.md) when threshold exceeded.
- **Loss Recovery Rate:** Measurement of FEC efficiency. Data analyzed in [docs/performance/benchmark.md](../performance/benchmark.md).
- **Power Budget Utilization:** Key indicator for IoT scenarios. Results dashboarded in [docs/testing/metrics.md](../testing/metrics.md).
- **Security Event Stream:** Coordinate anomaly detection with matrix in [docs/security/vulnerability.md](../security/vulnerability.md).

## Acceptance Criteria (DoD)
- Present 3 or more representative sequence diagrams explaining critical branches and exceptions.
- Idempotency and retry strategy consistent with functional requirements FR-02, FR-03.
- Monitoring points tied to measurable SLIs.
- No processing involving C/C++ dependencies exists, with alternative means specified as necessary.
- References to other architecture documents (overview, interfaces, dependencies) are accurate.
