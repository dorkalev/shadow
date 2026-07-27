---
id: CC7.1
family: CC7 — System Operations
category: Security (Common Criteria)
coso: null
title: Vulnerability Detection and Configuration Monitoring
weight: 3
automatable: full
nature: technical
---

# CC7.1 — Vulnerability Detection and Configuration Monitoring

## Criterion (AICPA TSP Section 100, verbatim)
> To meet its objectives, the entity uses detection and monitoring procedures to identify (1) changes to configurations that result in the introduction of new vulnerabilities, and (2) susceptibilities to newly discovered vulnerabilities.

## What it means

Two failure modes are in scope here. First: you change something — an IAM policy, a firewall rule, a dependency, a Dockerfile — and that change itself opens a hole. Second: you change nothing, but the world does — a CVE is published against a library you already run, and yesterday's safe configuration is today's exposure. CC7.1 requires you to have *procedures that detect both*, continuously, not as a one-off pentest.

For a tiny AI-first startup this is one of the cheapest criteria to satisfy properly, because the entire control set can be turned on in GitHub and your cloud provider in an afternoon: Dependabot alerts for newly disclosed CVEs in dependencies, CodeQL for vulnerabilities introduced by your own code changes, secret scanning with push protection for leaked credentials, and Prowler (or Security Command Center / AWS Security Hub) runs against your cloud account for misconfigurations. The auditor does not expect a security team; they expect the tooling to be enabled, alerts to be triaged, and a paper trail showing findings get fixed or accepted within a defined SLA.

The trap is "enabled but ignored." A Dependabot alert sitting open for 200 days is worse evidence than no scanner at all, because it proves you detect and don't act. Your vulnerability management policy should define severity-based remediation SLAs (e.g., critical 7 days, high 30, medium 90) and your ticket history must roughly honor them.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance from the AICPA, not requirements. Summarized:*
- Uses defined configuration standards — baseline configurations exist against which changes and drift can be judged.
- Monitors infrastructure and software for noncompliance with those standards that could introduce vulnerabilities.
- Implements change-detection mechanisms (e.g., file integrity monitoring, config drift detection) to alert on unauthorized modification of critical files or configurations.
- Detects unknown or unauthorized components introduced into the environment.
- Conducts vulnerability scans periodically and after significant changes, and remediates or accepts findings.

## What the auditor will ask for
- Vulnerability management policy including severity definitions and remediation SLAs.
- Evidence that dependency scanning, SAST, and secret scanning are enabled on all in-scope repositories (settings screenshots or API exports, point-in-time and configuration history).
- A sample of vulnerability alerts from the audit period with disposition: fix commit/PR, dismissal reason, or documented risk acceptance, with dates to test SLA adherence.
- Cloud configuration scan reports (Prowler/Security Hub/SCC) from multiple points in the period, plus remediation tickets for failed findings.
- The current open-alert list, to check nothing critical is aging beyond SLA at the audit date.
- Baseline/hardening standard for infrastructure (can be "we follow GCP org policy defaults + this Terraform repo is the baseline").
- Evidence of periodic review — e.g., a recurring Linear ticket "monthly vuln review" with completion history.

## How a tiny AI-first startup satisfies it
- **Dependabot alerts + security updates** enabled org-wide; auto-PRs for patch bumps merge through the normal SDLC (see CC8.1), so every remediation has a ticket, review, and archive record.
- **CodeQL default setup** on every production repository, running on PRs to staging and on the default branch — code-change-introduced vulnerabilities are caught pre-merge.
- **Secret scanning + push protection** enabled org-wide; any bypassed push-protection event is treated as a security event (feeds CC7.2/CC7.3).
- **Prowler** (or gcloud SCC findings) run on a schedule (weekly cron in CI); the report JSON is committed to the evidence store, and new FAILs above severity threshold open Linear tickets automatically.
- **Configuration drift**: infrastructure is Terraform-in-repo; any console change shows up as drift on the next `terraform plan` run in CI, and cloud audit logs (Admin Activity) capture who changed what outside IaC.
- **Vulnerability management policy** (one page): tooling list, SLAs by severity, risk-acceptance procedure requiring founder sign-off recorded on the Linear ticket.
- The compliance shadow runs the checks below continuously and flags regressions (tool disabled, alert past SLA) the day they happen — turning a Type II period into something you can't silently fail mid-year.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Dependabot alerts enabled on all repos | GitHub | `gh api /repos/{org}/{repo}/vulnerability-alerts` (204 = enabled) per repo |
| Dependabot security updates enabled | GitHub | `gh api /repos/{org}/{repo}/automated-security-fixes` |
| Open Dependabot alerts within SLA | GitHub | `gh api /repos/{org}/{repo}/dependabot/alerts?state=open`; compare `created_at` + severity against SLA table |
| CodeQL enabled and passing on default branch | GitHub | `gh api /repos/{org}/{repo}/code-scanning/default-setup` and `.../code-scanning/analyses` recency |
| Open code-scanning alerts within SLA | GitHub | `gh api /repos/{org}/{repo}/code-scanning/alerts?state=open` |
| Secret scanning + push protection enabled | GitHub | `gh api /repos/{org}/{repo}` → `security_and_analysis` block |
| No open secret-scanning alerts | GitHub | `gh api /repos/{org}/{repo}/secret-scanning/alerts?state=open` |
| Prowler scan recency and failure delta | evidence store | file-existence + mtime of latest `evidence/prowler/*.json`; diff FAIL count vs previous run |
| SCC / cloud findings triaged | GCP | `gcloud scc findings list` filtered to ACTIVE high/critical |
| Terraform plan clean (no drift) | CI | latest scheduled drift-check workflow conclusion via `gh api /repos/{org}/{repo}/actions/runs` |
| Vulnerability management policy exists and reviewed <12mo | repo | file-existence `policies/vulnerability-management.md` + last-reviewed frontmatter date |
| Risk acceptances have founder approval | Linear | MANUAL — spot-check acceptance tickets for sign-off comment |

## Evidence artifacts
- `evidence/prowler/YYYY-MM-DD.json` — scheduled cloud scan outputs, committed by CI.
- `evidence/github-security/YYYY-MM-DD.json` — daily shadow export of alert counts, tool-enablement state per repo.
- `policies/vulnerability-management.md` — policy with SLA table and review date.
- Linear: `vuln` label tickets with fix PR links (each fix PR also archived on `compliance-archives`, see CC8.1).
- `compliance-archives` branch — remediation PRs carry full audit records proving fixes went through the controlled SDLC.
