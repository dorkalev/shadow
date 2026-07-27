---
id: PI1.5
family: PI1 — Processing Integrity
category: Processing Integrity
coso: null
title: Complete and accurate data storage
weight: 1
automatable: partial
nature: technical
---

# PI1.5 — Complete and accurate data storage

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements policies and procedures to store inputs, items in processing, and outputs completely, accurately, and timely in accordance with system specifications to meet the entity's objectives.

## What it means
This category is optional — it applies only if the company includes Processing Integrity in its SOC 2 scope.

This criterion covers data at rest across the whole pipeline: the inputs you received, intermediate state while work is in flight, and the outputs you produced must be stored so nothing is lost, corrupted, or altered outside the intended process. In auditor terms it overlaps heavily with the availability backup criteria (A1.2/A1.3) — same backups, same restore tests — but viewed through an integrity lens: is stored data protected from loss *and* from unauthorized or accidental modification, and are storage activities recorded?

For a tiny startup on managed infrastructure, most of the heavy machinery is inherited or already built for other criteria: Cloud SQL durability + automated backups + PITR, GCS versioning, checksummed uploads (GCS does this natively), and transactional writes in Postgres (an item in processing is either committed or it isn't). What's genuinely yours to add: durable handling of in-flight items (queue messages persisted with acks, not fire-and-forget in memory), constraints that keep stored data internally consistent (foreign keys, NOT NULL, uniques — schema-as-control), least-privilege write access so only the application can mutate records, and audit trails (updated_at/audit columns or Cloud Audit Logs) recording storage activity.

## Points of focus (2022 revision, summarized)
These are guidance to consider, not requirements:
- **Protects stored items** — stored inputs, items in processing, and outputs are protected against theft, corruption, destruction, or deterioration (summary).
- **Archives and protects system records** — records are archived and protected consistent with retention requirements (summary).
- **Stores data completely and accurately** — storage processes preserve completeness and accuracy of data, and errors in storage are detected and corrected (summary).
- **Creates and maintains records of system storage activities** — records of storage activity are kept completely and accurately (summary).

## What the auditor will ask for
- Storage architecture description: where inputs, in-flight items, and outputs are persisted (DB, queues, buckets).
- Backup and PITR configuration for those stores, plus restore-test evidence (typically shared with A1.2/A1.3).
- Database schema constraints and transaction usage demonstrating integrity of stored data.
- Durable queue configuration (persistence, ack deadlines, DLQ) for items in processing.
- Write-access control: who/what can modify stored data (IAM bindings, DB roles), reviewed during the period.
- Audit logging configuration for storage systems (Cloud Audit Logs / data-access logs) and sampled entries.
- Retention/archival settings matching the retention policy.

## How a tiny AI-first startup satisfies it
- **Reuse the A1.2/A1.3 stack and cite it**: automated backups, PITR, and documented restore tests are the loss-protection evidence here too — one control set, two criteria mappings. Say so explicitly in the control matrix instead of inventing duplicates.
- **Schema as an integrity control**: foreign keys, NOT NULL, CHECK, and unique constraints in migrations; writes wrapped in transactions. The migrations directory is reviewable evidence that stored data can't silently go inconsistent.
- **Durable in-flight state**: Pub/Sub (persistent by design) with explicit ack deadlines and a DLQ; no business-critical work held only in process memory. For multi-step jobs, a state table with status transitions rather than implicit state.
- **Native checksums**: GCS validates MD5/CRC32C on upload — note the inheritance; for critical file outputs, store the checksum alongside and verify on read.
- **Least-privilege writes**: application service account is the only writer to prod DB/buckets; humans get read or break-glass only. `roles/editor`-on-everything is the anti-pattern the shadow tool should flag.
- **Storage activity records**: enable Cloud Audit Logs (admin activity is on by default; enable data-access logs for sensitive stores), plus `created_at`/`updated_at` and, where it matters, an audit/history table.
- **Retention/archival**: GCS lifecycle transitions to archive classes per the retention schedule (shared evidence with C1.2 when Confidentiality is also in scope).

## Automated shadow checks

> Datastore commands are per-stack: Cloud SQL shown. On Firestore stacks (the blessed `provision/gcp`), the equivalents are `gcloud firestore databases describe` (PITR, delete protection, state) and `gcloud firestore backups list` / `backups schedules list` (schedule present, recent snapshots).
| Check | Source | Method |
|---|---|---|
| Backups + PITR on primary datastore | GCP | `gcloud sql instances describe <inst> --format=json` → backupConfiguration.enabled and pointInTimeRecoveryEnabled `True` (shared with A1.2) |
| Restore test evidence current | GitHub | `gh api repos/<org>/<repo>/contents/compliance/restore-tests/` → newest log < 365 days (shared with A1.3) |
| Bucket versioning on data stores | GCP | `gcloud storage buckets describe gs://<bucket> --format=json` → versioning enabled or softDeletePolicy set |
| Integrity constraints in schema | GitHub | Fetch migrations dir via `gh api` → grep for FK/NOT NULL/UNIQUE/CHECK on core tables → present |
| Durable queue + DLQ for in-flight items | GCP | `gcloud pubsub subscriptions describe <sub> --format=json` → ackDeadlineSeconds set, deadLetterPolicy present |
| Only service accounts can write prod data | GCP | `gcloud projects get-iam-policy <proj> --format=json` → no human principals with write roles on SQL/storage; flag `roles/editor` grants |
| Data-access audit logs enabled | GCP | `gcloud projects get-iam-policy <proj> --format=json` → auditConfigs include DATA_WRITE for storage/SQL services |
| Lifecycle/archival rules match retention | GCP | `gcloud storage buckets describe gs://<bucket> --format="json(lifecycle)"` → rules align with retention schedule |
| Stored data is actually consistent | — | MANUAL — auditor samples records / reconciliation output for consistency |
| Archive restorability | — | MANUAL — confirm archived data can be retrieved when needed |

## Evidence artifacts
- Control-matrix cross-reference note mapping A1.2/A1.3 backup and restore-test evidence to PI1.5.
- Migrations directory permalink highlighting integrity constraints on core tables.
- `gcloud pubsub subscriptions describe` JSON export — durability, ack, and DLQ settings for processing queues.
- Project IAM policy JSON export showing write access limited to application service accounts.
- Audit-log configuration export (`auditConfigs`) plus sampled Cloud Audit Log entries for data writes.
- Bucket lifecycle/versioning exports aligned to the retention schedule.
