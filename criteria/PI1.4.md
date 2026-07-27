---
id: PI1.4
family: PI1 — Processing Integrity
category: Processing Integrity
coso: null
title: Complete, accurate, timely output delivery
weight: 1
automatable: partial
nature: technical
---

# PI1.4 — Complete, accurate, timely output delivery

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements policies and procedures to make available or deliver output completely, accurately, and timely in accordance with specifications to meet the entity's objectives.

## What it means
This category is optional — it applies only if the company includes Processing Integrity in its SOC 2 scope.

Processing can be perfect and the product still fail if outputs don't reach the right party, intact, on time. "Output" for a SaaS/AI startup means API responses, webhooks you emit, generated documents/exports, dashboards, emails/notifications, and scheduled reports. The criterion wants three properties — complete (all of it arrives), accurate (what arrives is what was produced, delivered to the *intended* recipient only), and timely (within committed/expected timeframes) — plus protection of output in transit and at rest, and records of what was delivered.

The sharpest edge for small teams is delivery to the intended party: tenant isolation bugs (customer A sees customer B's export) are output-accuracy failures and among the worst incidents a B2B startup can have. Practical controls: authorization checks on every output path (tested), signed time-limited URLs for file outputs instead of public links, retry-with-backoff plus failure alerting on outbound webhooks, and delivery records (webhook attempt logs, email provider logs). Timeliness is covered by latency/freshness monitoring against whatever your docs or contracts promise.

## Points of focus (2022 revision, summarized)
These are guidance to consider, not requirements:
- **Protects output** — output is protected during storage and delivery against theft, corruption, destruction, or misdirection (summary).
- **Distributes output only to intended parties** — delivery mechanisms ensure the correct, authorized recipient receives the output (summary).
- **Distributes output completely and accurately** — output is delivered in full, without error, and in a timely manner per specifications (summary).
- **Creates and maintains records of system output activities** — delivery records are kept completely and accurately (summary).

## What the auditor will ask for
- Inventory of output types and delivery channels (API, webhooks, exports, emails, reports).
- Authorization/tenant-isolation controls on output endpoints, and tests covering them.
- Configuration for secure delivery: TLS enforcement, signed URLs for file outputs, webhook signing.
- Outbound webhook retry/failure handling configuration and alerting, plus period examples of failures handled.
- Delivery records: webhook attempt logs, email provider delivery logs, export access logs (sampled).
- Latency/freshness monitoring against committed timeframes (SLOs, report schedules) and breach handling.
- Any output-misdirection incidents during the period and their remediation.

## How a tiny AI-first startup satisfies it
- **AuthZ on every output path, tested**: tenant-scoping enforced in the data-access layer (not per-endpoint ad hoc), with integration tests asserting cross-tenant reads fail. The test file is the evidence.
- **Signed, expiring URLs for file outputs**: GCS signed URLs (v4, short TTL) instead of public objects; bucket-level public access prevention enforced so the safe path is the only path.
- **Webhook delivery discipline**: HMAC-sign outgoing webhooks, retry with exponential backoff, log every attempt with status, alert on exhausted retries; expose or record delivery status per event.
- **TLS everywhere**: HTTPS-only load balancer/Cloud Run (default), `requireSsl` on DB — trivially verifiable inheritance from the platform.
- **Timeliness monitoring**: uptime checks + latency SLO alerts on API outputs; for scheduled reports/exports, a freshness check (job succeeded within window — shares evidence with PI1.3 job records).
- **Delivery records for free**: use the email provider's delivery log (Postmark/SendGrid), webhook attempt tables, and Cloud Run request logs as the record of output activities — no new system needed, just retention per policy.
- **AI outputs**: generated content is delivered under the same tenant-scoped, signed-URL machinery; accuracy of content is a PI1.3 concern, accuracy of *routing* is PI1.4.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Public access prevention on output buckets | GCP | `gcloud storage buckets describe gs://<bucket> --format="value(public_access_prevention)"` → `enforced` |
| Signed-URL pattern used for exports | GitHub | Repo search for signed-URL generation (`generate_signed_url`, v4 signing) on export paths → present; no public-object fallbacks |
| Cross-tenant isolation tests exist and pass | GitHub | Repo search for tenant-isolation test files; `gh run list` → suites passing on main |
| HTTPS-only serving | GCP | `gcloud run services describe <svc> --format=json` → ingress settings; LB frontend has no HTTP-without-redirect (`gcloud compute forwarding-rules list`) |
| Webhook retry/signing implemented | GitHub | Repo search for outbound webhook module → HMAC signing + retry/backoff + attempt logging present |
| Failed-delivery alerting | GCP | `gcloud alpha monitoring policies list --format=json` → policy on webhook-failure/exhausted-retry metric or log-based alert |
| Latency SLO / uptime checks on API | GCP | `gcloud monitoring uptime list-configs --format=json` → checks exist for output endpoints; latency alert policies present |
| Scheduled report freshness | GCP | `gcloud run jobs executions list --job=<report-job> --format=json` → executions succeed within schedule window |
| Delivery records retained | — | MANUAL — sample webhook/email logs and confirm retention per policy |
| Outputs reached the correct party | — | MANUAL — auditor samples a delivery end-to-end (event → signed URL/webhook → recipient) |

## Evidence artifacts
- Output inventory (section in the system description or `docs/outputs.md`): output types, channels, recipients, timing commitments.
- `gcloud storage buckets describe` exports showing public access prevention; permalink to signed-URL generation code.
- Tenant-isolation test files + CI run history showing them passing through the period.
- Webhook delivery module permalink + sampled attempt logs (success, retry, exhausted-retry-with-ticket examples).
- `gcloud monitoring uptime list-configs` and latency alert-policy JSON exports; a period screenshot of the SLO dashboard.
- Email provider delivery log export (sampled) and any misdirection-incident postmortems (hopefully none).
