---
description: Promote green staging into main (ff-only) with a release record, release ticket, and per-ticket links
---
# /shadow:release — Ship Staging to Production

Human-triggered only. This command produces the CC8.1 approval evidence for
moving change into production; every step below is part of that record.

## Phase 1: Preflight (all must pass, else STOP)

```bash
git fetch origin main staging compliance-archives
```

1. **CI green on staging:**
   ```bash
   gh run list --branch staging --limit 5 --json workflowName,conclusion,headSha
   ```
   Any failure on the tip commit of staging → STOP. Fix staging first.
2. **No unexplained bypasses.** Check recent archives:
   ```bash
   git log origin/compliance-archives --oneline -20
   git grep -l '"is_bypass": true' origin/compliance-archives -- '*.json' || echo "no bypasses"
   ```
   For each bypass found since the last release: verify it has an incident
   ticket + backport PR (the `/shadow:hotfix` paper trail). Unexplained bypass →
   STOP, run `/shadow:hotfix` for it before releasing.
3. **Fast-forward is possible:** `git merge-base --is-ancestor origin/main origin/staging` — non-zero exit → STOP (main has diverged; investigate, never force).

## Phase 2: Build the release summary

```bash
git log origin/main..origin/staging --oneline --no-merges
gh pr list --base staging --state merged --json number,title,url,mergedAt --limit 100
```

From these, assemble: every commit, every merged PR, and every ticket ID
(extract `[A-Z]{2,6}-[0-9]+` or `#N` from PR titles/bodies) since the last
release. Resolve ticket titles via Linear MCP `get_issue` or `gh issue view`.
Present the full summary to the user as a table: `| Ticket | Title | PR | Commits |`.

## Phase 3: Human confirmation (case-sensitive, random word)

Generate a random confirmation word (e.g. `openssl rand -hex 3` → prefix with a
readable word like `ship-a1b2c3`). Tell the user:

> Releasing {N} commits / {M} PRs / {K} tickets to production.
> Type exactly: **{WORD}** to proceed.

The reply must match **case-sensitively and exactly**. Anything else → ABORT
the release entirely and report. No retries within the same prompt; the user
can rerun the command. An agent never supplies this word for the user.

## Phase 4: Write the release record to compliance-archives

Preferred (deterministic — the vendored binary does the gathering, rendering, and worktree push):

```bash
RELEASED_BY="<the confirming human>" ARCHIVES_PUSH=1 \
  .shadow/ci/target/release/shadow-ci release-record
```

Fallback if the binary isn't vendored — a temporary worktree, never switching the working branch:

```bash
DATE=$(date +%Y-%m-%d)
WT=$(mktemp -d)/archives
git worktree add "$WT" origin/compliance-archives
mkdir -p "$WT/releases"
# releases/release-${DATE}.json: {date, released_by, staging_sha, main_sha_before,
#   commits[], prs[{number,title,url}], tickets[{id,title,url}], confirmation: "typed"}
# releases/release-${DATE}.md: human-readable rendering of the same
git -C "$WT" add releases/ && git -C "$WT" commit -m "release ${DATE}: staging -> main"
git -C "$WT" push origin HEAD:compliance-archives
git worktree remove "$WT" --force
```

If the same-day file exists, suffix `-2`, `-3`, … Append-only: never rewrite an
existing release file.

## Phase 5: Release ticket

Create a ticket titled `Release ${DATE}` whose body lists every included
ticket, PR, and commit, plus a link to the archive record:
- Linear MCP: `save_issue(...)`, state Done.
- GitHub Issues: `gh issue create --title "Release ${DATE}" --body-file /tmp/release.md`, then close it with a comment.

## Phase 6: Fast-forward main

```bash
git checkout main && git pull --ff-only origin main
git merge --ff-only origin/staging
git push origin main
```

**If `--ff-only` fails: STOP immediately.** Do not merge with a commit, do not
rebase, NEVER force-push main. Report the divergence to the user — someone
pushed to main directly (→ `/shadow:hotfix`) or history was rewritten
(→ incident).

## Phase 7: Close the loop on every ticket

Comment the release on every included ticket: "Released to production in
Release ${DATE}: {archive link / release ticket URL}".
- Linear MCP: `save_comment(issueId, body)` per ticket.
- GitHub Issues: `gh issue comment {N} --body "..."` per issue.

Report to the user: release date, main SHA, ticket count, archive path,
release ticket URL.

## STOP conditions (recap)
- Staging CI red · unexplained bypass in archives · ff not possible · confirmation word mismatch · archive push rejected (non-fast-forward on compliance-archives) → STOP in every case; a partial release is reported honestly, never patched over.
