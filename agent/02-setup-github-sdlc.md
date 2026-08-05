# Agent Runbook 02 — Install the SOC 2-Compliant SDLC

You are the compliance shadow's setup agent. Precondition: Runbook 01 ran and the user reviewed `shadow/scan-report.md`. Your job: install the SDLC defined in [`../sdlc/SDLC.md`](../sdlc/SDLC.md) onto the company's GitHub org and repos. This runbook **mutates settings** — present the full change list and get one explicit confirmation before executing; then execute all of it without further pauses.

## Step 0 — Confirm scope

Show the user: org name, repos to configure, tracker (Linear team key or GitHub Issues), ticket regex (e.g. `[A-Z]{2,5}-[0-9]+` or `#[0-9]+`), review bot available (the built-in shadow reviewer? a third-party bot app? human?). One confirmation, then go.

## Step 1 — Org-level hardening (CC6.1, CC6.2)

```bash
gh api -X PATCH /orgs/{org} -f two_factor_requirement_enabled=true
gh api -X PATCH /orgs/{org} -f default_repository_permission=read -f members_can_create_public_repositories=false
```
If enabling 2FA would eject members (API returns the list), report who and stop for the user — ejecting a founder mid-setup is not a good first impression.

## Step 2 — Branch topology + rulesets (CC8.1, CC6.3)

Per repo:
1. Ensure the default branch is `main`; feature branches start from and merge back to `main`.
2. Create the `compliance-archives` orphan branch with a README explaining it is protected, tamper-evident evidence.
3. Rulesets (`gh api -X POST /repos/{org}/{repo}/rulesets`):
   - **main**: require pull requests and status checks (`ci`, `dependency-review`, `compliance-audit`, `compliance-review-gate`), block force pushes and deletions, and require conversation resolution. An emergency founder bypass, if the plan permits one, is detected and documented per SDLC §9.
   - **compliance-archives**: block force pushes and deletions. The standard workflow appends records; GitHub audit logs attribute every direct append and reconciliation exposes missing or extra records. Do not claim the built-in `GITHUB_TOKEN` can be the sole bypass actor—repository rulesets do not accept that integration on every account/plan.

## Step 3 — CI gates (CC8.1)

Install the platform's native tooling (the full contract and file list live in [`../actions/README.md`](../actions/README.md)):

1. **Vendor the tooling** into the repo:
   - `actions/shadow-ci/` → `.shadow/ci/` (the Rust compliance binary: `check` + `archive` subcommands)
   - `agent/` → `.shadow/agent/` and `criteria/` → `.shadow/criteria/` (the daily audit's runbook + test plan)
   - `commands/` → `.claude/commands/shadow/` (the developer workflow: start, load, finish, fix-compliance, fix-pr, release, hotfix)
   - `actions/workflows/*.yml` → `.github/workflows/`
2. `.github/workflows/ci.yml` — build + test on PR. If the repo has no tests at all, create the harness and one real test; flag test debt in the report rather than faking coverage.
3. `compliance.yml` runs `shadow-ci check` in two phases (`audit` at awaiting-review, `review-gate` at post-review). Tune its env block: `TICKET_PATTERN` from Step 0, reviewers, `TEST_EXCLUDE_PATHS`, `CONFIDENCE_THRESHOLD` (default 70). Secret `LINEAR_API_KEY` only if Linear ticket verification is wanted — the gates are deterministic and need no LLM key.
4. `post-merge-archive.yml` runs `shadow-ci archive` on merge: JSON+MD record to the auto-created `compliance-archives` branch, **bypass detection** against the live branch ruleset, Slack alert via `SLACK_WEBHOOK_URL` if set.
5. `deterministic-verify.yml` runs the daily clock loop without an LLM. `daily-verify.yml` is a manual, explicitly approved deep review; use it only for judgment work and set a spend budget first.
6. Register all contexts from Step 2 as **required status checks** on `main` (do this after first successful runs so the context names are exact).
7. PR template (`.github/pull_request_template.md`) with the four required sections: Summary / Tickets table / Changes / Test Plan.

## Step 4 — Scanners on (CC7.1, CC6.8)

Per repo: enable Dependabot alerts + security updates, secret scanning + push protection, CodeQL default setup where the language is supported (`gh api -X PATCH /repos/{org}/{repo}` security fields + `/repos/{org}/{repo}/code-scanning/default-setup`).

## Step 5 — The paper layer (CC5.3, CC1.x, CC3.x, CC9.x)

Create (or adopt into) a private `policies` repo:
- Seed the policy pack from `../policies/README.md` (adapt the canon templates, filling in the company name, owner, review date). Policies merge through the same PR flow: that *is* the approval evidence.
- `risk-register.md` — seed from the scan's findings (each top gap is a risk with likelihood×impact 1–4, treatment, owner).
- `vendor-register.md` — seed from scan (cloud, GitHub, tracker, email, LLM providers — yes, Anthropic/OpenAI are vendors with data access: CC9.2).
- `access-register.md` — seed from Phase 1/2/3 scan output.
- `runbooks/` — onboarding.md, offboarding.md, incident-response.md, hotfix.md (from SDLC §9), restore-test.md.
- `evidence/` — directory convention: `evidence/{YYYY}/{QN}/` for quarterly artifacts (access reviews, management-review minutes).

## Step 6 — The clock (CC4.1)

Install the periodic verification, choosing what the company can host:
- GitHub Actions cron (daily) in the policies repo running Runbook 03 headlessly (`claude -p` with `03-verify-compliance.md`), or
- a local `/schedule` / launchd job on the founder's machine.
Also create recurring tracker tickets: quarterly access review, quarterly management review, annual risk refresh, annual restore test (CC-mapped, per SDLC clock table).

## Step 7 — Handover report

Write `shadow/setup-report.md`: every setting changed (before → after), every workflow installed, every file created, what remains manual, and the first-run instructions for Runbook 03. Post the summary to the user. The gauge is not computed here — that's Runbook 03's job, and running it now gives the baseline.
