---
id: CC7.2
family: CC7 — System Operations
category: Security (Common Criteria)
coso: null
title: Anomaly Monitoring and Security Event Analysis
weight: 3
automatable: partial
nature: technical
---

# CC7.2 — Anomaly Monitoring and Security Event Analysis

## Criterion (AICPA TSP Section 100, verbatim)
> The entity monitors system components and the operation of those components for anomalies that are indicative of malicious acts, natural disasters, and errors affecting the entity's ability to meet its objectives; anomalies are analyzed to determine whether they represent security events.

## What it means

Where CC7.1 is about scanning for *vulnerabilities* (things that could be exploited), CC7.2 is about watching *operations* for anomalies (things that may already be going wrong): a spike in 500s, a login from an impossible location, a service crash-looping, a burst of failed auth attempts, a Cloud Function suddenly egressing gigabytes. You need monitoring that surfaces these, and — the part small teams miss — a documented habit of *analyzing* what surfaced and deciding whether it's a security event or noise.

For a 1–10 person startup the realistic stack is: Cloud Monitoring alerting policies for infrastructure health (uptime checks, error-rate, resource saturation), Sentry for application errors, cloud audit logs retained and queried for admin/IAM anomalies, and GitHub audit log for repo/org anomalies. Alerts route somewhere a human actually looks (Slack channel, PagerDuty-lite, email). The "analysis" half is satisfied by a lightweight triage convention: every alert that isn't obviously benign gets a Linear ticket with a one-line disposition ("transient GCS blip, no security relevance" is fine).

The auditor's core question for a Type II is: "show me your alerting was on all period, show me alerts fired, show me someone looked." Silence is suspicious — an environment that produced zero anomalies in twelve months usually means monitoring is decorative. A handful of triaged false positives is *good* evidence.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Implements detection policies, procedures, and tools; designs detection measures to identify anomalies in system operation.
- Monitors detection tools themselves for effective operation (the monitor is monitored).
- Implements filters/rules to analyze data collected across sources and identify anomalies indicative of malicious acts, natural disasters, or errors.
- Designates responsibility for analyzing detected anomalies to determine whether they represent security events.
- Considers logs and events from infrastructure, software, and physical/environmental sources where relevant.

## What the auditor will ask for
- Inventory of monitoring tools and what each covers (infra, app errors, audit logs, endpoint if any).
- Export/screenshots of alerting policies in Cloud Monitoring and Sentry alert rules, with creation/modification dates showing coverage across the period.
- Alert notification channel configuration (where alerts go, who receives them).
- A sample of fired alerts from the period with triage evidence — Linear tickets or Slack thread exports showing analysis and disposition.
- Log retention configuration for cloud audit logs (Admin Activity, Data Access where enabled) and GitHub org audit log access.
- Uptime/availability reporting for in-scope services.
- Procedure describing how anomalies are escalated to the security-event evaluation process (CC7.3).

## How a tiny AI-first startup satisfies it
- **Cloud Monitoring**: uptime checks on public endpoints, alerting policies for error-rate (>2% 5xx over 5m), latency P95, instance/container restarts, and budget anomaly alerts (cost spikes are a real compromise indicator). All policies notify a `#alerts` Slack channel.
- **Sentry** on every deployed service: new-issue alerts and regression alerts to the same channel; release tagging so errors map to deploys (ties into CC8.1 archives).
- **Log-based alerts** on cloud audit logs: IAM policy changes, service-account key creation, firewall rule changes, logging-sink deletion — each fires a notification. These are the "malicious act" detectors.
- **GitHub anomaly feed**: the compliance shadow polls the org audit log for deploy-key creation, branch-protection/ruleset changes, member permission escalations, and force pushes — and specifically for **bypass merges** to staging/main that lack an archive record (its CC8.1 detector doubles as a CC7.2 anomaly source).
- **Triage convention** (documented in the incident response plan): any alert not auto-resolved within its dedupe window gets a Linear ticket labeled `anomaly`, with a disposition field: `noise | error | security-event`. Tickets marked `security-event` flow into CC7.3.
- Weekly recurring Linear ticket: "review alert channel + open anomalies" — closes with a one-line summary; this is your evidence that a human analyzes, not just receives.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Alerting policies exist and enabled | GCP | `gcloud alpha monitoring policies list` — count enabled, diff vs baseline set |
| Uptime checks configured for prod endpoints | GCP | `gcloud monitoring uptime list-configs` |
| Notification channels valid | GCP | `gcloud alpha monitoring channels list` — verified state, non-empty |
| Audit-log-based alerts present (IAM, SA keys, firewall, sink deletion) | GCP | `gcloud logging metrics list` + matching alert policies |
| Admin Activity log retention ≥ audit period | GCP | `gcloud logging buckets describe _Default` retention days |
| Sentry alert rules active | Sentry API | GET `/api/0/projects/{org}/{proj}/rules/` (token in shadow config) |
| GitHub org audit anomalies (ruleset changes, force pushes, deploy keys) | GitHub | `gh api /orgs/{org}/audit-log` filtered by action list |
| Bypass merges without archive record | GitHub | compare merged PRs / direct pushes on staging+main vs `compliance-archives` records |
| Anomaly tickets triaged within 5 business days | Linear | Linear API: `anomaly` label, created→disposition time |
| Weekly review ticket completed | Linear | recurring ticket completion history |
| Detection tooling itself healthy | GCP/CI | last successful run timestamps of shadow's own polling jobs; alert on staleness |
| Physical/environmental monitoring | — | MANUAL — N/A for pure-cloud; note inheritance from GCP/AWS SOC 2 report |

## Evidence artifacts
- `evidence/monitoring/policies-YYYY-MM-DD.json` — periodic export of Cloud Monitoring alert policies and channels.
- `evidence/github-audit/YYYY-MM.jsonl` — monthly org audit-log pulls with anomaly annotations.
- Linear: `anomaly` tickets with dispositions; weekly review ticket history.
- `policies/incident-response.md` §Triage — the documented anomaly-analysis procedure.
- Sentry alert-rule export in `evidence/sentry/`.
- `compliance-archives` branch: bypass-detection reports (absence of bypass events is itself period evidence).
