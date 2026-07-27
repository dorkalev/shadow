# Agent Runbook 03 — Shadow Audit (periodic verification)

You are the compliance shadow's auditor. This runbook runs on a clock (daily via cron/schedule) and on demand. Your job: re-test every automated check across the in-scope criteria, recompute the gauge, persist state for the website, and turn every regression into a ticket. You behave like the auditor: evidence or it didn't happen.

## Inputs

- `criteria/*.md` — for each in-scope criterion: its `weight`, and its **Automated shadow checks** table (that table is the test plan; execute it literally).
- Scope config: `shadow/scope.json` (`{"categories": ["security","availability","confidentiality"], "org": ..., "repos": [...], "cloud": "gcp", "ticket_pattern": ...}`).
- State: `shadow/shadow.db` (SQLite, schema in `../website/SPEC.md`) or `shadow/state/*.json` if no DB yet.

## Procedure

### 1. Run every automated check
For each in-scope criterion, execute its check table (gh api, gcloud, URL fetches, file-existence). Each check yields `{criterion, check, verdict: pass|fail|unknown, evidence, ts}`. `unknown` (no permission, API error) is reported, never counted as pass. Batch independent checks in parallel.

Cross-cutting checks that must always run (they feed the hard gates):
- Org 2FA requirement on (CC6.1/6.2).
- Rulesets intact on `main`/`staging` with the compliance contexts still required (CC8.1) — drift here is the classic silent failure.
- `compliance-archives` completeness: every PR merged since last run has an archive record; any record with `is_bypass: true` has a linked incident/hotfix ticket (SDLC §9).
- Open critical Dependabot/secret-scanning/CodeQL alerts older than the SLA in the vulnerability policy (CC7.1).
- Service-account keys older than 90 days; new `owner`-level IAM grants since last run (CC6.3).
- Backup configuration still on; last restore-test evidence younger than 12 months (A1.2/A1.3).
- Attestation freshness: quarterly artifacts (`evidence/{YYYY}/{QN}/access-review*`, `management-review*`) present for the current quarter once >2 weeks in; annual artifacts younger than 12 months. Stale attestation ⇒ criterion decays to `in_progress` (PLAN §9).

### 2. Score

Per criterion: all checks pass + evidence fresh ⇒ `verified` (1.0); controls present but evidence partial/stale ⇒ `implemented` (0.6); some checks pass ⇒ `in_progress` (0.25); else `not_started`/`failing` (0). Manual-only criteria take their status from the newest attestation row, with decay.

**Gauge = Σ(weight × credit) / Σ(weight)** over in-scope criteria, then apply hard-gate caps (PLAN §4): org 2FA off, or production branch protection missing, or unexplained bypass merge in last 30 days ⇒ cap 79%. Round down. Never exceed the cap silently — state the cap and its reason in the report.

### 3. Persist

Upsert into `shadow/shadow.db` (`checks`, `criteria`, `events` tables) or POST to the website's `/ingest` endpoint if running. Append a gauge-history row (date, gauge, cap-reason) — the one-pager plots the trend.

### 4. React (this is what makes the shadow proactive, not a dashboard)

- **Regression** (criterion left `verified`): open a tracker ticket, priority by weight, titled `Shadow: {ID} regressed — {check}` with the evidence and the fix from the criterion file. One ticket per criterion, deduplicated against open shadow tickets.
- **Bypass merge without incident ticket**: highest-priority ticket + Slack (if configured). This is the loudest alarm we have.
- **Upcoming decay** (attestation expiring within 21 days): open/refresh a reminder ticket for the quarterly ritual.
- **All green**: no tickets, one line of output. The shadow is silent when things are fine.

### 5. Report

Write `shadow/verify-{date}.md`: gauge (and delta), caps in effect, regressions, new tickets, unknowns needing permissions. Keep it under a page — this is the artifact a founder actually reads and, later, hands to the real auditor as evidence that monitoring operated all period (which is itself CC4.1 evidence — the shadow audits itself into the report).
