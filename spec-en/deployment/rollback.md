# docs/deployment/rollback.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines rollback strategy for HoneyLink™. Describes triggers, procedures, data consistency, and communication plan without implementation code or C/C++ dependencies.

## Table of Contents
- [Rollback Philosophy](#rollback-philosophy)
- [Rollback Triggers](#rollback-triggers)
- [Rollback Procedures](#rollback-procedures)
- [Data Consistency and Schema Migration](#data-consistency-and-schema-migration)
- [Communication Plan](#communication-plan)
- [Post-Rollback Actions](#post-rollback-actions)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## Rollback Philosophy
- **Fail Fast:** Detect anomalies early and roll back immediately.
- **Automated:** Rollback triggered by monitoring alerts without manual intervention.
- **Safe by Default:** Prefer reverting to last known good state over partial fixes.
- **Data Integrity:** Prioritize data consistency. Coordinate rollback with database migration rollback.
- **Transparent:** Notify stakeholders and users promptly. Document incident thoroughly.

## Rollback Triggers
| Trigger | Threshold | Action |
|---------|-----------|--------|
| Error Rate Spike | >1% sustained 5 min | Auto-rollback |
| Latency Degradation | P99 >200ms sustained 10 min | Auto-rollback |
| Critical Alert | PagerDuty critical severity | Manual review + potential rollback |
| Failed Health Check | 3 consecutive failures | Auto-rollback |
| Security Incident | CVE exploited in prod | Immediate rollback + patching |
| Manual Request | SRE/On-call decision | Manual rollback |

- Trigger conditions aligned with [docs/deployment/ci-cd.md](ci-cd.md) deployment gates.

## Rollback Procedures
### Application Rollback (Blue/Green)
1. **Switch Traffic:** Update load balancer to route 100% traffic to Blue (previous version).
2. **Drain Green:** Gracefully terminate Green pods (30s drain period).
3. **Verify Health:** Run smoke tests on Blue to confirm stability.
4. **Notify:** Send alert to Slack + create incident ticket.

### Application Rollback (Rolling Update)
1. **Halt Rollout:** Stop further pod replacements.
2. **Revert Deployment:** Apply previous manifest version (stored in Git).
3. **Monitor:** Watch for error rate and latency to return to baseline.
4. **Complete:** Once stable, mark rollback complete in ticket.

### Database Rollback
- **Forward-Only Migration:** Preferred approach. Design migrations as additive (new columns, tables).
- **Rollback Script:** If destructive migration required, prepare rollback SQL script beforehand.
- **Procedure:**
  1. Stop application traffic to database.
  2. Execute rollback script (restore previous schema).
  3. Restore data from PITR (Point-In-Time Recovery) if data corrupted.
  4. Restart application.
- **Validation:** Run schema diff tool to confirm rollback success.

### Configuration Rollback
- **GitOps:** Revert Git commit for config change.
- **Auto-Sync:** ArgoCD/FluxCD auto-applies previous config within minutes.
- **Secrets:** Rotate secrets if leaked. Do not roll back to leaked version.

## Data Consistency and Schema Migration
- **Backward Compatibility:** Design APIs and schemas to support N-1 version during rollback window.
- **Event Sourcing:** Replay events from log to reconstruct state if needed.
- **Database Snapshots:** Take snapshot before migration. Auto-restore on rollback.
- **Idempotency:** Ensure rollback script idempotent (safe to run multiple times).
- **Validation Checkpoints:**
  - Pre-migration: Run validation query to capture baseline state.
  - Post-rollback: Re-run query and compare results.

## Communication Plan
| Stakeholder | Notification Method | Timing |
|-------------|---------------------|--------|
| Engineering Team | Slack #incidents | Immediately on trigger |
| SRE/On-call | PagerDuty | Immediately |
| Product Manager | Slack DM + Email | Within 15 min |
| External Users | Status page update | Within 30 min (if user-facing) |
| Executive Team | Email summary | Within 1 hour (critical incidents) |

- **Incident Ticket:** Create Jira ticket immediately with:
  - Trigger condition
  - Rollback steps taken
  - Current status
  - ETA for resolution
- **Post-Mortem:** Conduct blameless post-mortem within 48h. Document in [docs/notes/decision-log.md](../notes/decision-log.md).

## Post-Rollback Actions
1. **Root Cause Analysis (RCA):** Identify why new version failed. Document findings.
2. **Preventive Measures:** Add new tests, improve monitoring, update runbooks.
3. **Re-Deployment Plan:** Fix root cause, re-test in staging, schedule new production deployment.
4. **Metrics Update:** Record rollback event in [docs/testing/metrics.md](../testing/metrics.md).
5. **Stakeholder Update:** Share RCA summary and prevention plan with team.

## Acceptance Criteria (DoD)
- Rollback philosophy, triggers, and procedures documented.
- Application, database, and configuration rollback steps specified.
- Data consistency strategy and schema migration rollback defined.
- Communication plan and post-rollback actions described.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
