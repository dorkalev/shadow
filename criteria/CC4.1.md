---
id: CC4.1
family: CC4 — Monitoring Activities
category: Security (Common Criteria)
coso: COSO Principle 16
title: Ongoing and Separate Evaluations
weight: 3
automatable: full
nature: technical
---

# CC4.1 — Ongoing and Separate Evaluations

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 16: The entity selects, develops, and performs ongoing and/or separate evaluations to ascertain whether the components of internal control are present and functioning.

## What it means
Someone — or something — has to check that your controls actually work, continuously or periodically, not just at audit time. "Ongoing evaluations" are monitoring built into normal operations (automated compliance monitors, dashboards, alerts); "separate evaluations" are point-in-time exercises (an internal control self-assessment, a penetration test, a vulnerability scan, the audit itself).

This is the criterion a compliance-shadow tool most directly *is*. A GRC-style continuous monitor that runs the auditor's checklist daily — branch protection on? MFA enforced? access reviews done? policies reviewed on schedule? — with a persisted pass/fail history, is a textbook ongoing evaluation and typically the strongest control a tiny startup has here. The audit question then becomes: does the monitor cover the right controls, does someone look at it, and do failures get fixed (that follow-through is CC4.2)?

Tiny startups should pair the automated monitor with two lightweight separate evaluations: an annual founder-led control self-assessment (walk the control list, mark each present/functioning, note gaps) and an annual third-party pentest or at minimum authenticated vulnerability scanning. That combination — continuous automation plus periodic human review — satisfies the "ongoing and/or separate" language cleanly at any headcount.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Considers a mix of ongoing and separate evaluations, balanced against the rate of change in the business.
- Establishes a baseline understanding of the current state of internal control to measure against.
- Uses knowledgeable personnel to perform evaluations.
- Integrates ongoing evaluations with business processes and adjusts to changing conditions.
- Adjusts scope and frequency of separate evaluations depending on risk.
- Separate evaluations are performed objectively to provide unbiased feedback.
- (TSC-specific) Considers different types of evaluations — pentests, independent certifications, internal assessments — and evaluates controls at vendors/business partners whose services affect the system.

## What the auditor will ask for
- Description and configuration of continuous monitoring (which controls are checked, how often, by what).
- Monitoring output history covering the audit period — pass/fail results over time, not just today's dashboard.
- The most recent penetration test or vulnerability scan report, plus remediation status of findings.
- The annual internal control self-assessment (checklist with results, date, who performed it).
- Evidence someone reviews monitoring output (alert routing, review sign-offs, tickets opened from failures).
- Vendor SOC 2 reports / certifications reviewed for critical sub-processors (GCP, GitHub, model provider).
- Vulnerability scanning / dependency scanning configuration and sample results (Dependabot, `gcloud` Security Command Center, etc.).

## How a tiny AI-first startup satisfies it
- Run the compliance-shadow tool itself on a schedule (daily GitHub Actions cron): it executes every automated check in these criteria files, writes a dated JSON/markdown result to the `compliance-archives` branch, and opens a Linear ticket on any failure. That archive *is* your period-long monitoring evidence.
- Enable the free/native scanners: GitHub Dependabot alerts + security updates, secret scanning, CodeQL (or equivalent) on main repos; GCP Security Command Center standard tier. These are ongoing evaluations that cost nothing and export cleanly.
- Annual founder-led control self-assessment: iterate this criteria directory, mark each control present/functioning/gap, record as `evidence/self-assessment-YYYY.md` via reviewed PR. Two hours, once a year.
- Annual third-party pentest (or, pre-revenue, at least an authenticated automated web/API scan); store the report and a remediation Linear epic.
- Collect and skim vendor SOC 2 reports annually (GCP, GitHub, Anthropic/OpenAI, etc.); record one-line conclusions in `evidence/vendor-reviews-YYYY.md`.
- Route monitor failures to a Slack channel or Linear so "someone looks at it" is provable via ticket timestamps.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Scheduled monitor workflow exists and ran in last 7 days | GitHub | `gh api repos/{org}/{repo}/actions/workflows` + `runs` — cron workflow, recent successful run |
| Monitoring result history covers period (no gaps > 7 days) | GitHub | List files/commits on `compliance-archives` branch, check date continuity |
| Dependabot alerts enabled on all non-archived repos | GitHub | `gh api repos/{org}/{repo}/vulnerability-alerts` (204 = enabled) per repo |
| Secret scanning + push protection enabled | GitHub | `gh api repos/{org}/{repo}` — `security_and_analysis` fields |
| Code scanning (CodeQL) configured on primary repos | GitHub | `gh api repos/{org}/{repo}/code-scanning/analyses` returns results |
| GCP Security Command Center enabled | GCP | `gcloud scc settings describe` / services list includes `securitycenter.googleapis.com` |
| Monitor failures produce tickets | Linear | Query issues labeled `compliance-failure`; cross-check against archived failures |
| Annual self-assessment file exists for current year | GitHub | File existence `evidence/self-assessment-YYYY.md` |
| Pentest report present and < 12 months old | GitHub | File existence + date in `evidence/pentest/` |
| Pentest quality/scope adequacy; objectivity of evaluations | — | MANUAL |

## Evidence artifacts
- `compliance-archives` branch: dated monitor run outputs (JSON + human-readable summary) for the entire period — the primary CC4.1 artifact.
- `.github/workflows/compliance-monitor.yml` — the monitor's schedule and scope, versioned.
- `evidence/self-assessment-YYYY.md` — annual control self-assessment with per-control status.
- `evidence/pentest/report-YYYY.pdf` plus remediation Linear epic export.
- Dependabot/secret-scanning/CodeQL configuration exports and sample alert lists in `evidence/github/`.
- `evidence/vendor-reviews-YYYY.md` — vendor SOC 2 review conclusions, with the reports themselves stored (license permitting) in `evidence/vendor-reports/`.
