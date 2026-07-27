---
id: PI1.3
family: PI1 — Processing Integrity
category: Processing Integrity
coso: null
title: System processing correctness controls
weight: 1
automatable: partial
nature: technical
---

# PI1.3 — System processing correctness controls

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements policies and procedures over system processing to result in products, services, and reporting to meet the entity's objectives.

## What it means
This category is optional — it applies only if the company includes Processing Integrity in its SOC 2 scope.

This is the middle of the pipeline: given valid inputs (PI1.2), does the system transform them completely, accurately, and on time — and when it doesn't, do you find out and fix it? "Policies and procedures over processing" sounds heavyweight, but in practice it means: processing logic is specified and tested, production errors are detected and corrected (error monitoring, alerting, incident/bug workflow), and processing activity is recorded (logs, job statuses, audit trails) so failures are traceable.

For an AI-first startup there are two flavors of processing. Deterministic processing (billing math, aggregation, transformation) is covered by the classic stack: tests in CI, error tracking (Sentry), job-level success/failure records, and a bug-fix loop through Linear. LLM-in-the-loop processing needs an honest adaptation: you cannot unit-test a model into determinism, so the controls become output-schema validation on every model response, guardrail checks/eval suites run in CI or on a schedule, fallback/retry behavior on invalid outputs, and logging of model/prompt versions so a bad output is diagnosable. Auditors accept this framing when it's documented — what they don't accept is "the model handles it."

## Points of focus (2022 revision, summarized)
These are guidance to consider, not requirements:
- **Defines processing specifications and activities** — the processing steps required to produce the product/service are defined, consistent with product specifications (summary).
- **Processes inputs completely, accurately, and timely** — system processing is designed and operated to meet those three properties, including handling of exceptions (summary).
- **Detects and corrects production errors** — processing errors are identified promptly and corrected, with records of the errors and their resolution (summary).
- **Records system processing activities** — processing activity records are created and maintained completely and accurately (summary).

## What the auditor will ask for
- Description of key processing flows (a diagram or docs section is fine) and where their logic is specified.
- CI evidence: test suites covering processing logic, run on every change, with results for the period.
- Error monitoring configuration (Sentry or equivalent) and the triage workflow for production errors.
- A sample of production processing errors from the period traced to resolution (alert → ticket → fix PR → deploy).
- Job/batch processing records: schedules, success/failure history, retry and exception handling.
- For AI steps: output validation code, eval/guardrail results, and model/prompt version logging.
- Processing activity logs demonstrating traceability for a sampled transaction.

## How a tiny AI-first startup satisfies it
- **Tests as the specification-enforcement control**: unit/integration tests on processing logic, required green in CI before merge (branch protection). The `gh run` history is period-long operating evidence.
- **Error tracking wired to workflow**: Sentry (or GCP Error Reporting) capturing unhandled errors, alerting to Slack, with a norm that P0/P1 errors become Linear bugs. The Sentry→Linear trail is exactly what the auditor samples.
- **Job execution records**: Cloud Scheduler + Cloud Run jobs give per-execution status for free (`gcloud run jobs executions list`); failed executions alert; retries are configured explicitly.
- **Exception handling that surfaces, not swallows**: failed items go to DLQ/exception tables with alerts (shared with PI1.2), not bare `except: pass`.
- **LLM processing controls**: every model response validated against a schema (retry/fallback on failure); a small eval suite in CI or nightly with tracked pass rates; prompt files versioned in git; model + prompt version logged per request for diagnosis. Document this in `docs/ai-processing.md`.
- **Traceability**: request IDs propagated through logs from input to output so any single transaction's processing can be reconstructed.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| CI tests required and passing | GitHub | `gh api repos/<org>/<repo>/branches/main/protection` → required status checks include tests; `gh run list --json conclusion` → pass history |
| Error tracking integrated | GitHub | Repo search for Sentry/Error Reporting SDK init in prod entrypoints → present with DSN from env |
| Prod errors become tickets | Linear | Linear API: issues with label `bug`/`incident` in period exist and reference Sentry links (sampled cross-check) |
| Scheduled jobs succeeding | GCP | `gcloud run jobs executions list --job=<job> --format=json` → recent executions `Succeeded`; failures matched to tickets |
| Job failure alerting configured | GCP | `gcloud alpha monitoring policies list --format=json` → policy on job/execution failure or error-count metrics |
| LLM output validation in code | GitHub | Repo search: schema validation adjacent to model-call sites (structured outputs, parse+retry wrappers) → present |
| Eval suite runs and pass-rate tracked | GitHub | `gh run list --workflow=evals --json conclusion,createdAt` → scheduled/PR runs through period |
| Prompt/model versions in VCS and logs | GitHub | Prompts stored under `prompts/` with commit history; log fields include model+prompt version (code search) |
| Processing logic matches specifications | — | MANUAL — auditor walks a sampled transaction against the spec |
| Eval coverage adequacy | — | MANUAL — judgment on whether evals cover the material failure modes |

## Evidence artifacts
- Branch protection JSON export + CI run history export for the audit period.
- Sentry project settings/screenshot + a sampled error-to-resolution package (Sentry issue, Linear ticket, fix PR, deploy record).
- `gcloud run jobs executions list` JSON exports showing batch job success/failure history.
- `docs/ai-processing.md` — model-step controls: validation, retries/fallbacks, eval approach, versioning.
- Eval run artifacts (CI logs or `compliance/eval-results/` snapshots) with pass rates over time.
- Sampled structured logs reconstructing one transaction end-to-end via request ID.
