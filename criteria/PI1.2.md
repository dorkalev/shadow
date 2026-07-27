---
id: PI1.2
family: PI1 — Processing Integrity
category: Processing Integrity
coso: null
title: System input completeness and accuracy
weight: 1
automatable: partial
nature: technical
---

# PI1.2 — System input completeness and accuracy

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements policies and procedures over system inputs, including controls over completeness and accuracy, to result in products, services, and reporting to meet the entity's objectives.

## What it means
This category is optional — it applies only if the company includes Processing Integrity in its SOC 2 scope.

Inputs are where processing integrity is won or lost: if garbage enters the system, no downstream control saves you. The criterion asks for controls that ensure inputs are **complete** (nothing that should have been captured was dropped — a webhook missed, a file upload truncated, a batch partially ingested) and **accurate** (inputs conform to expected formats, types, ranges, and are attributable to the right customer/tenant).

For a tiny startup this maps to engineering practices you likely half-have already: schema validation at the API boundary that rejects malformed input with clear errors, idempotency keys so retries don't create duplicates, durable queues with dead-letter handling so failed ingestion is visible rather than silent, and reconciliation for batch/webhook sources (counts received vs. counts processed). The audit work is mostly making these visible: validation lives in code (point at the schema layer), completeness lives in DLQ alerts and reconciliation jobs, and records of inputs live in request logs and queue metadata.

## Points of focus (2022 revision, summarized)
These are guidance to consider, not requirements:
- **Defines characteristics of processing inputs** — required characteristics (format, type, source, ranges) of inputs are defined and are consistent with the specifications in PI1.1 (summary).
- **Evaluates processing inputs** — inputs are checked against defined characteristics; nonconforming inputs are rejected or flagged and corrected (summary).
- **Creates and maintains records of system inputs** — records of inputs received are kept completely and accurately, supporting traceability and reconciliation (summary).

## What the auditor will ask for
- Documented input validation approach (can be the schema layer itself plus a short policy paragraph).
- Code/config demonstrating boundary validation (API schemas, file-format checks) and rejection behavior for a sample endpoint.
- Evidence of duplicate/idempotency handling for retried or replayed inputs.
- Dead-letter queue configuration and alerting, plus handling of any DLQ events during the period.
- Reconciliation evidence for batch or webhook input sources (received vs. processed counts), if applicable.
- Input records: request logs / ingestion logs demonstrating inputs are traceable, sampled across the period.
- Error-rate monitoring on input endpoints (4xx/validation-failure metrics).

## How a tiny AI-first startup satisfies it
- **Validate at the edge, in one place**: Pydantic/Zod schemas on every API route and every queue consumer; unknown fields rejected or logged; framework returns structured 422s. CI enforces types so the layer can't be bypassed silently.
- **Idempotency**: idempotency keys on mutating endpoints (or natural unique constraints in Postgres — `ON CONFLICT DO NOTHING` with logging), so client retries and webhook redelivery can't double-process. A unique index is auditor-legible evidence.
- **Durable ingestion with dead-lettering**: Pub/Sub subscription with a dead-letter topic (or SQS DLQ) and an alert when DLQ depth > 0. This is the completeness control — failures become tickets, not silence.
- **Reconciliation where inputs are countable**: for webhook providers (e.g., Stripe) run a daily/weekly job comparing provider event counts to ingested rows; log the result. For file ingestion, record row counts in vs. rows accepted vs. rows rejected.
- **Keep input records**: structured request logs with request IDs and tenant IDs, retained per the retention policy — enough to answer "did input X arrive, and what happened to it?"
- **LLM angle**: user-supplied text destined for a model still gets structural validation (size limits, encoding, tenant attribution) — semantic garbage-in is a product problem, but truncated or misattributed input is a PI1.2 failure.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Validation schemas at API boundary | GitHub | `gh api` code search for route decorators + schema models (framework-specific patterns) → validation layer present on input routes |
| CI enforces type/schema checks | GitHub | `gh run list --workflow=ci --json conclusion` → type-check/test jobs green on default branch |
| Idempotency/unique constraints exist | GitHub | Search migrations for `UNIQUE` constraints / idempotency-key columns on ingestion tables → present |
| Dead-letter topic configured | GCP | `gcloud pubsub subscriptions describe <sub> --format="json(deadLetterPolicy)"` → deadLetterTopic set, maxDeliveryAttempts sane |
| DLQ depth alerting exists | GCP | `gcloud alpha monitoring policies list --format=json` → policy on dead-letter topic message count |
| DLQ currently empty / drained | GCP | Pull metric via monitoring API: `pubsub.googleapis.com/topic/send_message_operation_count` on DL topic → zero or matched to remediation tickets |
| Validation-error monitoring | GCP | Alert policy or dashboard on 4xx/422 rate for input endpoints exists |
| Reconciliation job runs and passes | GitHub | `gh run list --workflow=reconciliation --json conclusion,createdAt` → scheduled runs succeeding through period (or check job's log artifact) |
| Rejected-input handling is correct | — | MANUAL — auditor samples a rejected input and traces the error/correction path |
| Reconciliation coverage is sufficient | — | MANUAL — judgment on which input sources need reconciliation |

## Evidence artifacts
- Permalinks to the schema/validation layer and a sample route showing rejection behavior.
- Migration files or schema dump showing unique/idempotency constraints on ingestion tables.
- `gcloud pubsub subscriptions describe` JSON export — dead-letter policy configuration.
- Alert policy exports (DLQ depth, validation-error rate) plus one fired-alert example with its ticket.
- Reconciliation job source + a sample of run logs/outputs across the period (CI artifacts or `compliance/reconciliation/` logs).
- Sampled structured request logs demonstrating input records with request/tenant IDs.
