---
id: CC2.1
family: CC2 — Communication and Information
category: Security (Common Criteria)
coso: COSO Principle 13
title: Relevant, Quality Information
weight: 2
automatable: partial
nature: technical
---

# CC2.1 — Relevant, Quality Information

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 13: The entity obtains or generates and uses relevant, quality information to support the functioning of internal control.

## What it means
Internal control only works if the people operating it can see what's happening. This criterion asks: does the company collect the data its controls depend on — logs, alerts, vulnerability findings, access lists, system inventories — and is that data complete, accurate, timely, and actually used? For a tiny startup, this maps to a small, concrete set: cloud audit logging enabled and retained, application/infra monitoring with alerts that reach a human, dependency and vulnerability scanning (Dependabot, cloud scanner findings), an asset/system inventory, and the compliance dashboard itself.

"Quality" matters as much as existence. Logs that are enabled but never retained, alerts that go to a dead Slack channel, or an asset inventory from two years ago all fail the spirit of the criterion. The auditor will check both configuration (is audit logging on?) and consumption (who looks at this, and can you show they did?).

For an AI-first shop there's a modern angle: information supporting control over the AI-assisted SDLC — which AI tools are in use, PR review coverage of AI-generated changes, secret-scanning results — is part of the relevant information set, because your controls claim to manage those risks.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Identifies information requirements** — the information needed to support internal control and each control's operation is identified.
- **Captures internal and external sources of data** — information systems capture data from inside the entity (logs, tickets, HR events) and outside (threat advisories, vendor notifications, customer reports).
- **Processes relevant data into information** — raw data is turned into usable, actionable information (dashboards, alerts, reports).
- **Maintains quality throughout processing** — information is complete, accurate, timely, protected, verifiable, and retained; data with these characteristics supports internal control.
- **Considers costs and benefits** — the nature, quantity, and precision of information is commensurate with objectives (proportionality — relevant for tiny teams).

## What the auditor will ask for
- Logging and monitoring policy or SDLC/operations policy section describing what is logged, retained, and monitored.
- Evidence that audit logging is enabled on cloud accounts (GCP Cloud Audit Logs / AWS CloudTrail config export) and log retention settings.
- Alerting configuration: what conditions alert, and where alerts go (Slack channel membership, PagerDuty/uptime monitor config).
- Sample alerts from the period and evidence of triage/response.
- Vulnerability/dependency scanning configuration and a sample of findings with disposition (Dependabot alerts → tickets/PRs).
- Asset/system inventory (repos, cloud projects, SaaS tools, data stores) with a recent review date.
- The compliance-monitoring dashboard itself and evidence it is reviewed (ties to CC4.1).

## How a tiny AI-first startup satisfies it
- Turn on the defaults and keep proof: GCP Cloud Audit Logs (Admin Activity is always on; enable Data Access logs for sensitive services) or CloudTrail in all regions; set a log retention/sink you can state in policy (e.g., 90 days hot, 1 year archived).
- Alerting that reaches humans: uptime checks + error alerting (Cloud Monitoring, Sentry) routed to a Slack channel both founders are in. Keep it small — three good alerts beat thirty ignored ones.
- Vulnerability information: GitHub Dependabot alerts + secret scanning + code scanning enabled org-wide; a weekly triage habit where alerts become Linear tickets or dismissals with reasons.
- Inventory as code: `inventory/systems.md` in the policies repo listing repos, cloud projects, SaaS tools (including AI tools), and data classifications; reviewed quarterly (a one-commit touch with review date bump is fine if nothing changed).
- Let the shadow tool be the information system: it pulls the above sources on schedule, produces the dashboard, and archives snapshots — simultaneously satisfying "generates and uses quality information" and creating its own evidence.
- Write the whole arrangement into `policies/logging-and-monitoring.md` (one page): sources, retention, alert destinations, review cadence.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Cloud audit logging enabled | GCP | `gcloud logging sinks list` + `gcloud projects get-iam-policy` audit config; or `gcloud alpha logging settings describe`; for AWS, `aws cloudtrail describe-trails` |
| Log retention meets stated policy | GCP | `gcloud logging buckets describe _Default --location=global` — retentionDays ≥ policy value |
| Monitoring/alerting policies exist and route to a live channel | GCP | `gcloud alpha monitoring policies list` + notification channels list; verify channel not empty |
| Dependabot alerts, secret scanning, code scanning enabled on all repos | GitHub | `gh api repos/{org}/{repo}` security_and_analysis fields, per repo |
| Open critical Dependabot alerts within SLA | GitHub | `gh api repos/{org}/{repo}/dependabot/alerts?state=open&severity=critical` — age < SLA days |
| Inventory file exists and reviewed in last quarter | GitHub | File existence + latest commit date on `inventory/systems.md` |
| Logging-and-monitoring policy exists, reviewed annually | GitHub | File existence + commit date via `gh api` |
| Alert triage happens | Linear API | Sample `alert-triage`/`vuln` labeled issues exist during period with closure |
| Alerts actually reach and are read by humans | MANUAL | Auditor observes Slack channel / interviews founders |
| Completeness/accuracy judgment of information | MANUAL | Auditor assessment |

## Evidence artifacts
- `policies/logging-and-monitoring.md` — one-page information/monitoring policy.
- `inventory/systems.md` — system and data inventory with quarterly review commits.
- `evidence/logging/` — exports: CloudTrail/audit-log config JSON, retention settings, monitoring policy list, snapshotted quarterly to `compliance-archives`.
- `evidence/vuln-management/` — Dependabot/secret-scanning status export and triage-ticket summary per quarter.
- Sample alert → Linear ticket → resolution chains (exported) demonstrating information is used.
- Shadow-tool dashboard snapshots (dated) in `compliance-archives` — the "processed into information" artifact.
