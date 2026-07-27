---
id: CC6.6
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: External Boundary Protection
weight: 3
automatable: partial
nature: technical
---

# CC6.6 — External Boundary Protection

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements logical access security measures to protect against threats from sources outside its system boundaries.

## What it means

CC6.1 governs access controls generally; CC6.6 zooms in on the boundary between your system and the internet. It asks: what stops an outside attacker? The expected answers are boundary controls (firewalls/security groups, load balancers, private networking), hardened authentication for anything reachable from outside (MFA everywhere, no password-only access), and protection of credentials as they cross the boundary (TLS, no basic auth over HTTP).

For a tiny cloud startup, the boundary is: your production ingress (load balancer / Cloud Run URL / API gateway), your cloud consoles and SaaS admin panels, your GitHub org, and remote access paths (SSH, database ports). The strong minimal posture is: exactly one intended public entry point (the app, behind HTTPS), everything else private — databases on private IPs, SSH replaced by IAP tunnels or SSM Session Manager or removed entirely with serverless, and every human-facing console behind SSO+MFA. Because your workforce is remote, "external access" describes literally all access, which is why MFA enforcement is the single most load-bearing control here.

The auditor's practical test is an attack-surface review: enumerate public IPs, open ports, and DNS records; check each is intended, TLS-protected, and authenticated. Your shadow tool can run nearly the same enumeration continuously — this is one of the most automatable criteria in CC6.

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Restricts access from outside the boundary** — boundary protection systems (firewalls, security groups, demilitarized zones) limit external connections to authorized ones.
- **Protects identification and authentication credentials** — credentials transmitted across the boundary are protected (encrypted transport, no cleartext protocols).
- **Requires additional authentication or credentials for external access** — access from outside the boundary requires stronger authentication, e.g., MFA.
- **Implements protections against external threats** — measures such as hardened configurations, intrusion detection/prevention, or equivalent monitoring address outside threats.

## What the auditor will ask for

- Network architecture diagram showing system boundary, ingress points, and trust zones (a simple one-pager is fine).
- Firewall rules / security group export with justification for every internet-facing rule.
- Inventory of public endpoints (IPs, DNS, load balancers) and confirmation each is intended and TLS-only.
- MFA enforcement evidence for all externally reachable admin surfaces: Workspace, GitHub, GCP/AWS console, and any VPN/remote-access tooling.
- How remote administrative access works (IAP/SSM/VPN configuration; or attestation that no SSH exists in serverless setups).
- Evidence of protection against external threats: Cloud Armor/WAF config, rate limiting, or documented rationale plus monitoring/alerting on suspicious authentication events.
- Recent vulnerability scan or external port-scan results, if performed.

## How a tiny AI-first startup satisfies it

- **One front door.** All production traffic enters via a single HTTPS load balancer or managed serverless endpoint (Cloud Run, App Runner). HTTP redirects to HTTPS; TLS 1.2+ minimum; certificates managed by the platform.
- **Default-deny everything else.** GCP firewall / AWS security groups: no `0.0.0.0/0` on 22, 3389, 5432, 3306, 6379, 27017, etc. Databases and caches on private IP only, reachable solely from the app's network. Serverless architectures can honestly state "no VMs, no open ports."
- **No raw SSH.** If VMs exist, access is via IAP TCP forwarding (GCP) or SSM Session Manager (AWS) — IAM-authenticated, MFA-backed, logged. Otherwise remove the ingress rule entirely.
- **MFA on every console.** Workspace 2SV enforced; GitHub org 2FA required; cloud console access rides Workspace SSO. Password-only access to anything is nonexistent.
- **Basic hardening beats absent hardening.** Cloud Armor or provider WAF on the public endpoint if budget allows; otherwise document platform-level DDoS protections (inherited) and alert on anomalous auth activity (Workspace suspicious-login alerts on; GCP log-based alert on repeated IAM auth failures).
- **Know your surface.** Keep `inventory/endpoints.yaml` listing every public DNS name and IP with purpose; the shadow tool diffs reality against it and screams on drift.

## Automated shadow checks

> Datastore commands are per-stack: Cloud SQL shown. On Firestore stacks (the blessed `provision/gcp`), the equivalents are `gcloud firestore databases describe` (PITR, delete protection, state) and `gcloud firestore backups list` / `backups schedules list` (schedule present, recent snapshots).

| Check | Source | Method |
|---|---|---|
| No firewall rule allows 0.0.0.0/0 to admin/DB ports | GCP | `gcloud compute firewall-rules list --format=json` → parse `sourceRanges` + `allowed` ports against denylist |
| All public forwarding rules terminate HTTPS | GCP | `gcloud compute forwarding-rules list --format=json` → port/target proxy type |
| SQL instances have no public IP (or authorized networks empty) | GCP | `gcloud sql instances list --format=json` → `ipAddresses`, `settings.ipConfiguration` |
| Cloud Run / services not exposing unintended unauthenticated access | GCP | `gcloud run services get-iam-policy` → `allUsers` invoker only on intended public services per `endpoints.yaml` |
| Public endpoint inventory matches reality | GCP + DNS | `gcloud compute addresses list`, `gcloud dns record-sets list` diffed vs. `inventory/endpoints.yaml` |
| TLS config on each public endpoint (version, cert validity) | Network | Scripted TLS probe (`openssl s_client`/curl) against inventory endpoints |
| HTTP→HTTPS redirect enforced | Network | `curl -sI http://<endpoint>` → expect 301 to https |
| GitHub org 2FA enforced | GitHub | `gh api /orgs/{org}` → `two_factor_requirement_enabled` |
| Workspace 2SV enrollment/enforcement 100% | Google Workspace | Admin SDK Reports API `is_2sv_enrolled`/`is_2sv_enforced` per user |
| IAP/SSM used instead of open SSH | GCP | Firewall export: SSH allowed only from IAP range `35.235.240.0/20`, nowhere else |
| Auth-failure alerting configured | GCP | `gcloud alpha monitoring policies list` / `gcloud logging metrics list` → expected alert policy exists |
| External port scan matches expected surface | Network | MANUAL (or tool-run nmap against inventory IPs where legally cleared) |
| WAF/DDoS posture documented | Docs | File/grep check on `policies/network-security.md`; substantive adequacy is MANUAL |

## Evidence artifacts

- `inventory/endpoints.yaml` — declared public surface, version-controlled.
- `evidence/gcp/firewall-rules-<date>.json`, `forwarding-rules-<date>.json`, `sql-instances-<date>.json` — periodic exports.
- `evidence/network/tls-scan-<date>.txt` — TLS probe output for each endpoint.
- `evidence/workspace/mfa-report-<date>.csv` and `evidence/github/org-settings.json` — shared with CC6.1.
- `docs/architecture.md` — boundary diagram (drawn once, updated on change; git history shows currency).
- `evidence/network/portscan-<date>.txt` — external scan results, if run.
- Alerting policy export `evidence/gcp/monitoring-policies-<date>.json`.
