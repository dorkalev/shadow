# Provision — the compliant runtime, as code

One blessed target — **GCP** — in two datastore variants, both fully annotated (every Terraform resource carries the criterion it evidences):

| Variant | Datastore | Cost at pilot scale | Secrets that exist |
|---|---|---|---|
| **`gcp/`** (default) | Firestore: PITR + daily backups, IAM-native auth | **$0** (always-free tier) | none — no SA keys (WIF), no DB credentials (IAM-native) |
| **`gcp-cloudsql/`** | Postgres 16: backups + PITR, `ENCRYPTED_ONLY`, connector-only access | ~$10/month (the instance) | exactly one — DATABASE_URL in Secret Manager, runtime-SA-readable only |

Pick by data model, not by cost: if the app fits documents, `gcp/` is free and secretless; if it needs SQL, `gcp-cloudsql/` is the same baseline with one well-caged secret. Each variant ships its own restore drill (`restore-test.yml` in actions/workflows is the Firestore one; `gcp-cloudsql/restore-test-cloudsql.yml` replaces it on SQL stacks). Everything else — WIF, deploy SA, Cloud Run, audit logs, uptime alerting — is identical.

## Why one target, and why this one

The provisioner's job is not convenience — it is **evidence surface**. The shadow verifies compliance via CLI/API (access reviews, backup config, SSL enforcement, audit logs, IAM as data), and GCP exposes all of it. Platforms with thinner APIs (however pleasant their DX) leave the shadow half-blind: "compliant but unverifiable" is against this platform's religion. Firebase is GCP underneath — take Firestore/Auth as Terraform resources in this same project if needed; never configure them through a console the shadow can't see.

A second target (AWS, Fly, …) earns its Terraform the day a customer demands it — by implementing the same check surface the criteria files define. Do not add speculative platforms.

## What `gcp/` creates (criteria in parentheses)

| Resource | Criterion |
|---|---|
| Runtime SA with `datastore.user` only | CC6.1, CC6.3 (least privilege) |
| Deploy SA reachable **only** via WIF from one GitHub repo — zero exported keys anywhere | CC6.1, CC8.1 (deploys only from gated CI) |
| Firestore: PITR + daily backup schedule (14d retention), delete protection, TLS-only by construction, zero connection secrets | A1.2, CC6.6, CC6.7, CC6.5, CC6.1 |
| Secret Manager enabled with a per-secret-accessor pattern for app secrets (commented template) | CC6.1, CC6.3 |
| Cloud Run, autoscaling 0–4, runtime SA | A1.1; CC6.4 + environmental inherited from GCP (CSOC) |
| Data-access audit logs on datastore writes + secret reads; 90-day log retention | CC7.2, CC2.1, CC6.3 |
| Uptime check + email alert policy | CC7.2 → CC7.3 |

## The flow

1. **Once, by a human**: `/shadow:provision` (or `terraform apply` by hand) — the only step that can't be keyless, because WIF can't bootstrap itself. ~10 minutes.
2. **Forever after**: `deploy.yml` ships `main` to Cloud Run via WIF — no keys, no human deploy access, the merge gates are the release control.
3. **Watched**: add the project to `quarterly-rituals.yml` (`GCP_PROJECTS`) and `restore-test.yml` (`GCP_PROJECT`/`LOCATION`) — the shadow's access reviews, restore tests, and daily verify then cover this environment automatically.

## What this deliberately does not include

Multi-environment plumbing (make a second project + `terraform workspace` when staging infra is truly needed — Cloud Run revisions cover most tiny-startup staging needs), Terraform remote state (use a GCS bucket when more than one human applies), and any resource whose only purpose is to look enterprise.
