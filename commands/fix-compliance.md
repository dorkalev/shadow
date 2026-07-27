---
description: Fix shadow-ci compliance failures by reading the actual CI report, never by weakening a gate
---
# /shadow:fix-compliance — Answer the Shadow

Fixes red `shadow-ci` checks from the ACTUAL report, not guesswork. The checker
scores from 100 (−10 per invalid ticket, −10 per unspecced changed file, −5 per
untested source file, −5 per missing required reviewer; fail below threshold,
default 70) plus hard gates that fail regardless of score: no valid ticket, PR
body under 20 chars, unresolved CRITICAL/MAJOR review findings, missing
required reviewer in the post-review phase.

## Phase 0: Read the report (MANDATORY FIRST STEP — do not skip, do not pre-analyze)

**Step 1 — check status:**
```bash
gh pr checks {pr} 2>&1
```
Both shadow-ci checks green → nothing to fix, exit.

**Step 2 — get the latest shadow-ci comment.** The checker maintains ONE
comment per phase, marked `<!-- shadow-ci:audit -->` (awaiting-review) and
`<!-- shadow-ci:review-gate -->` (post-review):
```bash
gh api repos/{owner}/{repo}/issues/{pr}/comments --paginate --jq '[.[] | select(.body | contains("shadow-ci:"))] | last | .body'
```
If both phases have comments, read both (drop `| last` and filter each marker).
Extract from the report: `compliant`, `score`, `threshold`, `invalid_tickets`,
`unspecced_changes`, `untested_files`, `missing_reviewers`,
`unresolved_findings`, `hard_gates`, `issues[]`.

**Step 3 — get the run log for file-level detail:**
```bash
gh run list --workflow compliance --branch "$(git branch --show-current)" --limit 1 --json databaseId --jq '.[0].databaseId'
gh run view {run_id} --log-failed 2>&1 | head -80
```

Only after Phase 0 do you act — and only on what the report actually says.

## Phase 1: Fix each failure type mechanically

**`unspecced_changes`** — a changed file is not traceable to a ticket via the
PR body's `## Changes` section. For each listed file: identify its ticket
(usually obvious from the path) and add a bullet for it under that ticket in
`## Changes`. Lockfiles get a one-liner under the ticket that changed deps.

**`invalid_tickets`** — a ticket referenced in title/body doesn't resolve in
the tracker. Verify: Linear MCP `get_issue`, or `gh issue view {N}`. Exists →
likely transient API failure; re-run/re-push to retry. Genuinely absent → remove
it from the PR title/body (and if the work is real, create the ticket properly
and reference the new ID).

**`untested_files`** — a changed source file has no corresponding test. Write
the test (this is the default answer). Only if the file is genuinely untestable
(config, generated code) does it belong in the repo's `test_exclude_paths` —
and changing that config is itself a ticketed, spec'd change on this PR,
justified in the PR body.

**Thin description hard gate** (body < 20 chars / missing sections) — rewrite
the PR body with all four sections: `## Summary`, `## Tickets` table,
`## Changes` (every changed file), `## Test Plan`. Apply with
`gh pr edit {pr} --body-file /tmp/pr_body.md`.

**`missing_reviewers`** — the required review bot hasn't posted. Summon it:
```bash
gh pr comment {pr} --body "<the bot's re-review command, if it has one>"
```
(Or push an empty commit to re-trigger it.) Then wait for it before expecting the
post-review phase to pass.

**`unresolved_findings`** (CRITICAL/MAJOR review-bot findings, hard gate) —
for each: fix the code, or if it's a false positive, reply on the review thread
with a **concrete** justification (point to the line/behavior that disproves
it — not "this is fine") and resolve the thread. This overlaps `/shadow:fix-pr`;
use that command's loop if there are many.

## Phase 2: Push and wait for the re-run

```bash
git add -A && git commit -m "{IDENTIFIER}: address compliance findings" && git push   # if code changed
```
(Body-only edits re-trigger the check without a push on `edited` events; if the
workflow only runs on `synchronize`, push an empty commit: `git commit --allow-empty -m "{IDENTIFIER}: re-run compliance" && git push`.)

Poll every 30 seconds, max 10 minutes:
```bash
gh pr checks {pr} --watch --fail-fast
```
Then re-read the shadow-ci comment (Phase 0, Step 2). Still red → loop back to
Phase 1 with the NEW report. After 10 minutes with no re-run → report status to
the user and stop polling.

## The one absolute rule

**NEVER weaken a gate to pass it.** Do not lower `confidence_threshold`, widen
`test_exclude_paths` to dodge a test, edit the workflow, ruleset, or ticket
regex, or delete the failing check. A gate change is a normal change: its own
ticket, its own spec, its own reviewed PR. If a gate is genuinely wrong, say so
to the user and open that ticket — while this PR satisfies the gate as it stands.

## STOP conditions
- No open PR for this branch → run `/shadow:finish` (or `/shadow:start`) first.
- No shadow-ci comment yet → CI hasn't run; wait, don't guess.
- Fix requires substantive new code beyond the spec → update the ticket/spec first (`/shadow:load` scope rule), then fix.
