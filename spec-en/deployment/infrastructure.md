# docs/deployment/infrastructure.md

**Badges:** `🚫 No Implementation Code` `🚫 No C/C++ Dependencies`

> Defines Infrastructure as Code (IaC) specification for HoneyLink™. Describes cloud provider strategy, network topology, resource management, and security hardening without implementation code or C/C++ dependencies.

## Table of Contents
- [IaC Goals and Principles](#iac-goals-and-principles)
- [Cloud Provider Strategy](#cloud-provider-strategy)
- [Network Topology](#network-topology)
- [Resource Management](#resource-management)
- [Security Hardening](#security-hardening)
- [Cost Optimization](#cost-optimization)
- [Disaster Recovery](#disaster-recovery)
- [Acceptance Criteria (DoD)](#acceptance-criteria-dod)

## IaC Goals and Principles
- **Declarative Definition:** All infrastructure defined in code (Terraform, Pulumi, or CloudFormation).
- **Version Control:** Infrastructure changes tracked in Git. GitOps workflow enforced.
- **Idempotency:** Apply IaC multiple times yields same result.
- **Blast Radius Minimization:** Separate state files per environment (dev, staging, prod).
- **No Manual Changes:** Console changes prohibited. All modifications via IaC PR.
- **No C/C++ Dependencies:** IaC tools and providers pure Rust or Go implementations.

## Cloud Provider Strategy
- **Primary:** AWS (Control-Plane, Data-Plane).
- **Multi-Cloud:** Azure or GCP for DR/Geographic redundancy.
- **Rationale:** Leverage managed services (RDS, EKS, EventBridge) to reduce operational burden.
- **Provider-Agnostic Layers:** Abstract cloud-specific APIs in [docs/architecture/interfaces.md](../architecture/interfaces.md).

### Service Mapping
| Component | AWS Service | Alternative (Azure/GCP) |
|-----------|-------------|-------------------------|
| Compute | EKS (Kubernetes) | AKS / GKE |
| Database | RDS (PostgreSQL + TimescaleDB) | Azure Database / Cloud SQL |
| Messaging | EventBridge + SNS/SQS | Event Grid / Pub/Sub |
| Key Management | KMS | Key Vault / Cloud KMS |
| Storage | S3 | Blob Storage / Cloud Storage |
| CDN | CloudFront | Azure CDN / Cloud CDN |

## Network Topology
```
[Internet]
   ↓
[CloudFront (CDN)]
   ↓
[ALB (TLS termination)]
   ↓
[Public Subnet: NAT Gateway]
   ↓
[Private Subnet: EKS Worker Nodes]
   ↓
[Database Subnet: RDS (Multi-AZ)]
```

- **Multi-AZ Deployment:** Minimum 3 AZs for high availability.
- **VPC Peering:** Connect control-plane and data-plane VPCs. No internet traversal.
- **Security Groups:** Enforce least-privilege. Whitelist only required ports/protocols.
- **TLS Everywhere:** mTLS between services. Cert rotation automated via cert-manager.

## Resource Management
- **Tagging Strategy:**
  - `Environment`: dev/staging/prod
  - `Service`: honeylink-control-plane, honeylink-data-plane
  - `Owner`: team-name
  - `CostCenter`: budget-code
- **Auto-Scaling:**
  - Horizontal Pod Autoscaler (HPA) for Kubernetes workloads.
  - Target CPU: 70%, Memory: 80%.
  - Scale-out on demand spike, scale-in with 10-minute cooldown.
- **Resource Limits:**
  - Define CPU/memory requests and limits in all pod specs.
  - Prevents resource exhaustion and noisy neighbor issues.

## Security Hardening
- **Principle of Least Privilege:** IAM roles scoped per service. No wildcard policies.
- **Secrets Rotation:** Quarterly rotation for service account keys, database passwords.
- **Network Isolation:** Database accessible only from private subnets. No public IPs.
- **Image Scanning:** Trivy scan on all container images. Block images with critical CVEs.
- **Distroless Containers:** Minimize attack surface with distroless base images.
- **Audit Logging:** Enable CloudTrail, VPC Flow Logs, EKS Audit Logs. Forward to SIEM.

## Cost Optimization
- **Right-Sizing:** Analyze CloudWatch metrics monthly. Downsize overprovisioned instances.
- **Reserved Instances/Savings Plans:** Commit to 1-year term for stable workloads (20-40% savings).
- **Spot Instances:** Use for non-critical batch jobs (up to 90% savings).
- **Storage Lifecycle:** Auto-archive logs to Glacier after 90 days.
- **Cost Allocation Tags:** Track spending per team/service. Review in monthly FinOps meeting.

## Disaster Recovery
- **RTO (Recovery Time Objective):** < 4 hours.
- **RPO (Recovery Point Objective):** < 15 minutes.
- **Backup Strategy:**
  - Database: Automated daily snapshots + point-in-time recovery (PITR).
  - Configuration: GitOps repo backed up to secondary region.
- **DR Region:** Failover to secondary AWS region on regional outage.
- **DR Drills:** Quarterly full DR switchover tests. Document results in [docs/notes/decision-log.md](../notes/decision-log.md).

## Acceptance Criteria (DoD)
- IaC goals, principles, and provider strategy documented.
- Network topology, resource management, and security hardening specified.
- Cost optimization and disaster recovery plans defined.
- C/C++ dependency exclusion explicitly stated.
- Links to related documents consistent.
