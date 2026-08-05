# Adopting the Shadow — CI actions & scripts

This directory is what a project copies to become shadow-compliant. Two halves, deliberately split:

- **Deterministic half (Rust, `shadow-ci/`)** — the per-PR gates and the evidence archive. No LLM in CI: every verdict is reproducible from the same inputs, which is exactly what an auditor wants from a control.
- **Judgment half (markdown, `../agent/` + `../commands/`)** — scanning, setup, deep reviews, and the developer workflow. An LLM is explicitly authorized only for bounded, judgment-only work.

No Python anywhere. Runtime dependencies of `shadow-ci`: `gh`, `git`, `curl` — all preinstalled on GitHub Actions runners.

## What `shadow-ci` does

### `shadow-ci check` — the per-PR compliance audit

Runs on every PR event, in two phases (`REVIEW_PHASE`):

| Check | Rule | Effect |
|---|---|---|
| Ticket traceability | `TICKET_PATTERN` regex over PR title+body only (crash-safe, never from the diff; default accepts `ABC-123` and `#42`). Verification by shape: `#N` → GitHub issue must exist, be an issue (not a PR), and not be this PR itself (no self-authorization); `ABC-123` → Linear API if `LINEAR_API_KEY` is set; otherwise extraction is the evidence | no valid ticket ⇒ **hard fail**; −10 per invalid ticket |
| Description | body ≥ 20 chars; sections `## Summary / ## Tickets / ## Changes / ## Test Plan` | too short ⇒ **hard fail**; missing sections reported |
| Change traceability | every changed file's path or basename must appear in the PR body (lock files exempt) | −10 per unspecced file |
| Test coverage | changed source files need a matching test file (`test_*`, `*_test.*`, `*.test.*`, `*.spec.*`, `tests/`) or inline `#[cfg(test)]`; `TEST_EXCLUDE_PATHS` exempts prefixes | −5 per untested file |
| Review gate | unresolved review-bot threads classified CRITICAL/MAJOR by regex; explicitly configured reviewer logins must have posted (post-review phase). The built-in review workflow is optional and advisory; only `<!-- shadow-review:complete -->` counts as a completed run, while unavailable/legacy markers never satisfy the gate | findings ⇒ **hard fail**; −5 per configured missing reviewer (+hard fail post-review) |

Score starts at 100; fail below `CONFIDENCE_THRESHOLD` (default 70) or on any hard gate. A `compliance:exempt` label records the checks without enforcing them. The result is posted as one self-updating PR comment per phase (markers `<!-- shadow-ci:audit -->`, `<!-- shadow-ci:review-gate -->`) plus `compliance_report.json`.

### `shadow-ci access-review` — the quarterly access packet (CC6.2/CC6.3)

Assembles the access-review evidence from live system state: GitHub org admins/members, outside collaborators, per-repo direct grants, deploy keys, GCP IAM bindings, and user-managed service-account key ages (`GCP_PROJECTS` csv). Sections degrade gracefully to "unavailable" where credentials are missing, and non-API systems (Workspace 2SV, tracker seats) are listed as manual attachments. Output includes a sign-off block — the human reads the diff against last quarter (the archives branch history *is* the diff), lists revocations, and signs by committing. **`USER_FILTER=<login-or-email>` turns it into a per-person grant report — run it before and after an offboarding to prove revocation.**

### `shadow-ci mgmt-packet` — the quarterly management-review packet (CC1.2/CC4.2)

Pre-fills the meeting: gauge trend (last 13 readings from `shadow.db`), bypass merges this quarter (grepped from the archives branch), open Dependabot/CodeQL/secret-scanning counts, incidents and open shadow-regression tickets. Ships with an agenda checklist and empty decisions/minutes blocks — the humans meet, decide, fill, commit. The commit is the oversight evidence.

### `shadow-ci release-record` — the release evidence record (CC8.1)

Gathers commits, PR numbers, tickets, and diff stats between `RELEASE_FROM` (the prior release SHA) and `RELEASE_TO` (default `origin/main`) and writes `releases/release-{ts}.json+md` to the archives branch. Called by the `release` command after the founder confirmation word.

All three write locally by default; `ARCHIVES_PUSH=1` commits to the archives branch (packets land under `evidence/{year}/{quarter}/`).

### `shadow-ci attest` — the CPA's fieldwork, on demand (CC8.1, CC4.1)

For any audit window (`SINCE`/`UNTIL`): builds the change population from three reconciled GitHub records — merged PRs, `main` history, and protected archive records — then tests **100% of the population**: T1 authorized by a valid ticket created before the PR, T2 merged by the authenticated founder/management actor (the author may be the same person), T3 gated with no bypass, T4 documented, and T5 delivered through the main PR path. Direct pushes, missing archives, and orphan records surface as exceptions. `ARCHIVES_PUSH=1` files the resulting report under `evidence/attestations/`.

### `shadow-ci archive` — the post-merge evidence record

On every merged PR: one JSON + one MD record (full PR metadata, reviews, comments, check runs, files, commits, and the compliance comment verbatim) committed to protected **`compliance-archives`**. Required status checks are the union of active repository rulesets and classic branch protection, so any merge with a failed or missing required check is flagged `is_bypass: true`. The branch blocks force pushes and deletion; normal records are appended by Actions, all writers remain attributable, and population reconciliation detects omissions/extras. This is tamper-evident source evidence for CC8.1, not a claim of legal immutability or exclusive machine write access.

## Distribution — three ways a repo consumes the shadow

1. **Vendored (default, what genesis.sh and runbook 02 do):** copy `.shadow/` into the repo; workflows build `shadow-ci` there (rust-cache makes it ~20s after the first run). The control's code lives in the repo's own reviewed history — the audit-friendliest shape.
2. **Central:** put this platform repo on GitHub (public or org-internal); adopting repos use thin caller workflows (`uses: org/shadow/.github/workflows/compliance.yml@v1`), download the prebuilt `shadow-ci` from Releases, and the deep review checks out the corpus as a second `actions/checkout` step. One place to upgrade every repo.
3. **Template repo:** mark a starter as a GitHub template — `gh repo create myapp --template org/shadow-template` — and new repos are born compliant.

Start vendored (works before this platform is even on GitHub); graduate to central/template when there is more than a handful of repos.

## Install (what runbook 02 automates)

1. **Vendor the tooling** into the adopting repo:
   ```
   .shadow/ci/            ← copy of actions/shadow-ci (Cargo.toml + src/)
   .shadow/agent/         ← copy of agent/ (runbooks 01–05, incl. the async-interview protocol)
   .shadow/criteria/      ← copy of criteria/ (61 files; the daily audit's test plan)
   .shadow/commands/      ← copy of commands/ (the SDLC commands + the 11 ritual interviews — read by the shadow agent)
   .shadow/site/          ← copy of website/app (the gauge; manual deep review can render it statically on GitHub)
   .shadow/procedures/    ← copy of procedures/ (PROCEDURES.md — seeds the machinery ledger)
   .claude/commands/shadow/ ← same commands, for optional interactive use from Claude Code
   .github/workflows/     ← copy of actions/workflows/*.yml
   .github/pull_request_template.md  ← the 4 sections
   ```
2. **Secrets**: LLM keys are optional (`ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, or `OPENAI_API_KEY`) and stay inert unless a repo variable explicitly enables the relevant LLM workflow. `LINEAR_API_KEY` is optional for ticket verification; `SLACK_WEBHOOK_URL` is optional for bypass/merge alerts.
3. **Rulesets**: after the first green runs, require `ci`, `dependency-review`, `compliance-audit`, and `compliance-review-gate` on `main`; require PRs; block force pushes and deletion. Protect `compliance-archives` separately against force pushes and deletion; record/reconcile every append.
4. **Tune** the env block in `compliance.yml`: `TICKET_PATTERN` (e.g. `#[0-9]+` for GitHub Issues), reviewers, `TEST_EXCLUDE_PATHS`.

## The workflows

| Workflow | Trigger | What it runs |
|---|---|---|
| `compliance.yml` | PR opened/updated/edited/labeled | `shadow-ci check` twice (audit + review-gate phases), report artifact 90 days |
| `test.yml` | PRs to and pushes on `main` | Rust unit tests plus machine-readable coverage measurements for the control binary and gauge app |
| `post-merge-archive.yml` | PR closed & merged | `shadow-ci archive` → compliance-archives + Slack |
| `daily-verify.yml` | manual only | Explicitly authorized, 12-turn Claude deep review on `agent/03-verify-compliance.md` |
| `deterministic-verify.yml` | cron daily + manual | `shadow-ci verify` plus a static dashboard artifact — live GitHub MFA, protected branches, and open security findings; no LLM or model key |
| `quarterly-rituals.yml` | cron, 1st of each quarter | `shadow-ci access-review` + `mgmt-packet` → evidence packets to archives, then dispatches the interview kickoffs to shadow-agent |
| `restore-test.yml` | cron, quarterly (template) | Firestore: restore latest daily backup into a throwaway database, prove ACTIVE, evidence to archives, delete it (A1.3) |
| `deploy.yml` | push to main | Keyless WIF deploy to Cloud Run — the only path to production (needs `provision/gcp` applied once) |
| `review.yml` | PR opened/updated | The built-in reviewer: an LLM (Claude/Gemini/Codex key) reviews the diff, posts severity-prefixed inline threads + a marker summary. Critical/Major threads block the merge via the gate; no key ⇒ a loud "NOT reviewed" notice instead of silence. |
| `drill.yml` | quarterly cron + manual | Fire drill: pushes a synthetic change through the standard route — proves the gate REJECTS a non-compliant PR (negative control), then ACCEPTS the fixed one, merges, and verifies the archive record. Needs `DRILL_TOKEN` (PAT/App token) so the gates actually trigger. |
| `shadow-agent.yml` | issue comments · incident closes · annual cron · weekly cron | The autonomous interviewer: kicks off and continues ritual interviews over GitHub issues, files signed evidence, sweeps reminders. LLM-pluggable (Claude / Gemini / Codex key). |

## Autonomy — nobody runs compliance

With `shadow-agent.yml` installed, no human ever invokes a compliance command. The division of labor:

| Brain | Doings |
|---|---|
| **Rust (`shadow-ci`), no LLM** | PR gates, post-merge archives + bypass detection, evidence packets, release records, populations, reminder sweeps. Deterministic — the controls an auditor tests are reproducible byte-for-byte, and there is no API key in the merge path. |
| **LLM, headless and opt-in** (Claude recommended for judgment work; Gemini or Codex accepted) | A manually authorized deep review; composing interview questions from gathered evidence; parsing human answers into filed artifacts; drafting postmortems, tabletop scenarios, and the system description. Never approves anything. |
| **Founder, async** | The sole human answers judgment questions and signs through an authenticated GitHub identity. The system discloses this concentration and relies on preventive machine gates, logs, reconciliation, and CPA scrutiny as compensating controls; bots are never counted as organizational segregation of duties. `/finalize` files gaps as OPEN, never assumed. |

The interview protocol (issue format, state marker, answer parsing, idempotence, cost discipline) is `agent/05-async-interviews.md` — the same ritual commands run synchronously in a Claude Code session or asynchronously over issues, unchanged.

## Design notes

- **Why deterministic CI?** Earlier designs ran an LLM agent per PR and per criterion. Mechanically checkable controls now live in `shadow-ci`: cheaper, faster, no API key in the hot path, and byte-for-byte reproducible verdicts. LLM judgment is manual and bounded to semantic deep review or ritual drafting.
- **Scoring**: −10 per invalid ticket, −10 per unspecced file, −5 per untested file, −5 per missing reviewer, threshold 70 — stable across releases so historical gauges stay comparable.
- **Unit-tested policy**: ticket extraction, traceability, test matching, severity classification, scoring, and bypass classification are pure functions with tests (`cargo test` in `shadow-ci/`).
