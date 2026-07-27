---
id: CC7.5
family: CC7 — System Operations
category: Security (Common Criteria)
coso: null
title: Recovery from Security Incidents
weight: 2
automatable: partial
nature: technical
---

# CC7.5 — Recovery from Security Incidents

## Criterion (AICPA TSP Section 100, verbatim)
> The entity identifies, develops, and implements activities to recover from identified security incidents.

## What it means

CC7.4 stops the bleeding; CC7.5 gets you back to normal — and makes sure "normal" is stronger than before. Recovery activities include restoring data and services (backups, redeploys, failover), verifying integrity after restoration, communicating the all-clear, and folding lessons learned back into controls. The criterion also implicitly covers your recovery *capability*: backups that exist, restore procedures that have been tested, and improvements actually implemented after incidents.

For a small cloud-native startup, recovery is mostly about three things: automated database/storage backups with a tested restore path, infrastructure-as-code plus CI/CD so any service can be rebuilt from the repo, and a lessons-learned loop where postmortem action items become merged changes. If you can demonstrate "we restored a backup to a scratch environment on this date and it worked" plus "these three postmortem items shipped as PRs," you're substantially done.

Auditors often test CC7.5 together with CC7.4 and A1.2/A1.3 (availability). Even in a security-only report, expect to show backup configuration and at least one restore test during the period. Untested backups are the classic finding here.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Restores the affected environment to a functional state; recovery activities are defined in advance where practicable.
- Communicates information about the event and recovery status to appropriate parties.
- Determines root cause of the event and, where needed, remediates it so recurrence is addressed, improving controls based on lessons learned.
- Incorporates incident lessons into the incident-response plan and recovery procedures.
- Periodically tests recovery capability (e.g., backup restoration exercises).

## What the auditor will ask for
- Recovery/backup procedures (may live inside the IR plan or BC/DR policy) including RTO/RPO targets, even informal ones.
- Backup configuration evidence: automated backup schedules, retention, and storage location for databases and critical buckets.
- Evidence of at least one restore test in the period: date, who ran it, outcome, time taken.
- For actual incidents: recovery steps executed, verification that service/data integrity was restored, and the all-clear communication.
- Postmortem action items and proof they were implemented (merged PRs / closed tickets).
- Evidence the IR plan or runbooks were updated after incidents or exercises.

## How a tiny AI-first startup satisfies it
- **Automated backups**: Cloud SQL automated daily backups + PITR enabled; critical GCS buckets with versioning or scheduled transfer to a backup bucket; retention ≥ 30 days. All declared in Terraform so configuration is reviewable and drift-detectable.
- **Everything-from-repo recovery**: services deploy via CI from git; Terraform recreates infrastructure. The recovery runbook is short because the honest answer is "run the pipeline against a clean project" — but write that down, with the manual steps (DNS, secrets seeding) enumerated.
- **Annual restore test** (semi-annual if you can): restore latest DB backup into a scratch instance, run a row-count/integrity script, record results in `evidence/restore-tests/`. The compliance shadow nags when the last test ages past policy.
- **Lessons-learned loop**: postmortem action items are Linear tickets; each ships through the CC8.1 SDLC, so "we improved controls after the incident" is provable via archive records rather than asserted.
- **BC/DR policy** (`policies/bcdr.md`) states RTO/RPO honestly (e.g., RTO 24h, RPO 24h for a seed-stage product), single-region posture and its accepted risk, and the escalation path if the region or a critical vendor (GitHub, GCP) is down.
- **All-clear communication**: the incident ticket's Recover phase requires a closing note; customer-facing recovery notices reuse the CC7.4 communication matrix.

## Automated shadow checks

> Datastore commands are per-stack: Cloud SQL shown. On Firestore stacks (the blessed `provision/gcp`), the equivalents are `gcloud firestore databases describe` (PITR, delete protection, state) and `gcloud firestore backups list` / `backups schedules list` (schedule present, recent snapshots).

| Check | Source | Method |
|---|---|---|
| Cloud SQL automated backups + PITR enabled | GCP | `gcloud sql instances describe` → `backupConfiguration` |
| Recent successful backup exists | GCP | `gcloud sql backups list --instance=...` latest status/time |
| Bucket versioning / backup transfer configured | GCP | `gcloud storage buckets describe` versioning; transfer job list |
| Backup retention meets policy | GCP | describe output vs policy value parsed from `policies/bcdr.md` |
| Restore test performed within policy window | evidence | file-existence + date of latest `evidence/restore-tests/*.md` |
| Backup resources defined in Terraform (no console-only config) | repo+GCP | grep Terraform for backup blocks; diff vs live describe output |
| Incident tickets have completed Recover phase | Linear | checklist item status on `incident` tickets |
| Postmortem action items implemented | Linear+GitHub | linked tickets closed with merged, archived PRs |
| IR plan/runbook updated after SEV1/SEV2 | repo | git log of `policies/incident-response.md` after incident dates |
| BC/DR policy exists, reviewed <12mo | repo | file-existence + frontmatter review date |
| Restore-test data integrity verification quality | — | MANUAL — human review of test script and results |

## Evidence artifacts
- `policies/bcdr.md` — RTO/RPO, backup standards, recovery runbook pointer, review date.
- `evidence/restore-tests/YYYY-MM-DD.md` — restore exercise records with timings and integrity check output.
- `evidence/backups/YYYY-MM-DD.json` — periodic shadow export of backup configuration and latest-backup status.
- Terraform files defining backup configuration (git history = configuration evidence over the period).
- Linear: incident tickets with Recover-phase completion; postmortem action-item tickets.
- `compliance-archives` branch — archive records proving lessons-learned changes shipped through the controlled SDLC.
