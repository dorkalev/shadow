---
id: CC5.2
family: CC5 — Control Activities
category: Security (Common Criteria)
coso: COSO Principle 11
title: General Controls over Technology
weight: 3
automatable: full
nature: technical
---

# CC5.2 — General Controls over Technology

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 11: The entity also selects and develops general control activities over technology to support the achievement of objectives.

## What it means
Almost every control a software startup has runs *on* technology — CI gates, IAM, MFA, logging. CC5.2 asks whether the technology layer itself is controlled: who can access the infrastructure, how changes to systems are managed, how the security tooling is configured, and how new technology is acquired and maintained. In COSO language these are ITGCs (IT general controls): access to technology, technology infrastructure controls, security management, and technology acquisition/development/maintenance.

For a GitHub + GCP + Workspace startup this is the most concrete criterion in the CC5 family and the most automatable in the whole framework. Nearly every expectation maps to a setting an API can read: SSO/2SV enforcement, org-level GitHub security settings, branch protection and required CI, IAM bindings without wildcard owners, no long-lived service-account keys, encryption defaults, backup configuration, dependency update automation.

The AI-first wrinkle: your "technology acquisition and development" process now includes AI coding agents and model APIs. The general controls extend to them — agent credentials are scoped and rotated like any service account, AI-generated code passes the same required review and CI as human code (branch protection doesn't care who wrote the diff, which is exactly why it's the right control), and adding a new model provider goes through vendor review. Auditors in 2025–26 ask about this directly; the good news is the existing ITGCs answer it if they're actually enforced.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Determines the dependency between business-process controls and technology general controls — which controls rely on which systems.
- Establishes relevant technology infrastructure control activities (availability, integrity, and continuity of the underlying platforms).
- Establishes relevant security management process control activities — restricting access rights to authorized users commensurate with job responsibilities, protecting from external threats.
- Establishes relevant technology acquisition, development, and maintenance process control activities — controls over obtaining, building, and changing technology.

## What the auditor will ask for
- Inventory of in-scope systems and the general controls over each (usually satisfied by `assets.md` + `controls.md`).
- Access control configuration: Workspace 2SV/SSO enforcement report, GitHub org member list with 2FA status, GCP IAM policies.
- Change management configuration: branch protection settings, required status checks, sample PRs showing review + CI before merge.
- Infrastructure controls: backup configuration and restore test evidence, encryption-at-rest confirmation, logging/monitoring configuration.
- Service account and API key inventory with rotation evidence (includes AI/model API keys).
- Patch/dependency management: Dependabot configuration and merge history.
- Provisioning/deprovisioning records for the period (ties to CC6.2/CC6.3 but sampled here).

## How a tiny AI-first startup satisfies it
- **Identity layer**: Google Workspace as IdP; enforce 2SV org-wide (Admin console policy, not per-user choice); GitHub org requires 2FA (`two_factor_requirement_enabled`); GCP access via Workspace identities only — no unmanaged accounts.
- **Change layer**: branch protection on every production repo — ≥1 required review, required CI status checks, no force pushes, `enforce_admins` on. This is your entire change-management control and it's free. AI-generated code gets zero exemptions.
- **Infrastructure layer**: GCP defaults do encryption at rest; enable automated backups (Cloud SQL PITR or equivalent) and run one documented restore test per year; retain Cloud Audit Logs; pin infrastructure in Terraform (or at minimum export configs) so drift is diffable.
- **Credential hygiene**: no user-managed service-account keys (`constraints/iam.disableServiceAccountKeyCreation` org policy where possible); secrets in GCP Secret Manager or GitHub Actions secrets, never in code (secret scanning + push protection on); model/API keys scoped per environment and rotated on personnel departure.
- **Acquisition/maintenance**: Dependabot auto-PRs merged through the normal review gate; new SaaS/model vendors require a Linear ticket with a mini vendor review before credentials are issued.
- Document all of the above in one page — `policies/technology-controls.md` — so the auditor gets narrative plus the config exports as proof.

## Automated shadow checks

> Datastore commands are per-stack: Cloud SQL shown. On Firestore stacks (the blessed `provision/gcp`), the equivalents are `gcloud firestore databases describe` (PITR, delete protection, state) and `gcloud firestore backups list` / `backups schedules list` (schedule present, recent snapshots).
| Check | Source | Method |
|---|---|---|
| GitHub org requires 2FA for all members | GitHub | `gh api orgs/{org}` — `two_factor_requirement_enabled == true` |
| Workspace 2SV enforced org-wide | Google Workspace | Admin SDK Reports/Directory API — 2SV enforcement policy + per-user `isEnrolledIn2Sv` |
| Branch protection full config on every production repo | GitHub | `gh api repos/{org}/{repo}/branches/{default}/protection` — reviews, status checks, no force push, enforce_admins |
| Sampled merged PRs had review + passing checks before merge | GitHub | `gh api` pulls list → reviews + check-runs per PR |
| No user-managed service-account keys in GCP | GCP | `gcloud iam service-accounts keys list --managed-by=user` per SA — expect empty |
| No primitive `roles/owner` grants beyond break-glass account | GCP | `gcloud projects get-iam-policy --format=json` parse bindings |
| Secret scanning + push protection enabled | GitHub | `gh api repos/{org}/{repo}` `security_and_analysis` block |
| Dependabot security updates enabled | GitHub | `gh api repos/{org}/{repo}/automated-security-fixes` |
| Cloud SQL/database backups enabled (PITR) | GCP | `gcloud sql instances describe` — `backupConfiguration.enabled` |
| Audit logging not excluded/disabled | GCP | `gcloud logging sinks list` + check for exclusion filters on _Required bucket |
| Annual restore test performed | GitHub | File existence `evidence/restore-test-YYYY.md` |
| Restore test actually validated data integrity | — | MANUAL — review of test writeup |

## Evidence artifacts
- `policies/technology-controls.md` — one-page ITGC narrative.
- Daily config-snapshot exports from the shadow tool (org settings, branch protection, IAM policies, 2SV report) on the `compliance-archives` branch — proves controls operated all period, not just at audit time.
- `evidence/github/`: org settings JSON, per-repo protection JSON, sampled PR review/check evidence.
- `evidence/gcp/`: IAM policy exports, service-account key listings, backup configs, audit-log configuration.
- `evidence/workspace/`: 2SV enforcement and enrollment reports.
- `evidence/restore-test-YYYY.md` — annual backup restore test with steps, result, and duration.
- Secret/key inventory with rotation dates (`evidence/secrets-inventory.md`, values never included).
