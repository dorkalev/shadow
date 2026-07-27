---
description: Pre-push gate — cleanup, spec alignment, commit, sync staging, push, PR ready, ticket to In Review
---
# /shadow:finish — The Pre-Push Gate

**NEVER use:** `git stash`, `git checkout --` / `checkout .`, `git reset --hard`,
`git clean -fd`, `git push --force`, `rm` on user files. If one seems needed →
STOP and ask the user.

## Phase 1: Discover & cleanup

Run in parallel:

```bash
git branch --show-current
git fetch origin staging
git log origin/staging..HEAD --oneline
git diff origin/staging..HEAD --stat
git status --short
```

And fetch the ticket + its spec comment (authoritative record of what was
approved): Linear MCP `get_issue` + `list_comments`, or
`gh issue view {N} --comments`.

Cleanup:
- Remove temp/debug artifacts from the diff: `*.log`, `.DS_Store`, `__pycache__`, commented-out debug code, stray `console.log`/`print` added during this work.
- **Secret scan the diff** — do not skip:
  ```bash
  git diff origin/staging..HEAD | grep -inE 'aws_secret|api[_-]?key|token|password|BEGIN (RSA|EC|OPENSSH) PRIVATE KEY|secret' || echo "no obvious secrets"
  ```
  A real secret in the diff → STOP. Remove it, and if it was ever committed/pushed, treat as an incident (rotate the credential, then `/shadow:hotfix` discipline applies).
- Run the project's linter and test suite. Failures → fix before proceeding.

## Phase 2: Spec-alignment gate (BLOCKING)

Compare the **ticket + spec comment** against the **actual diff**
(`git diff origin/staging..HEAD`). Classify every mismatch:

| Type | Meaning | Resolution |
|---|---|---|
| UNSPECCED | In the diff, not in the spec | Update spec comment / ticket, split to a new ticket, or remove the code |
| INCOMPLETE | In the spec, not in the diff | Implement now, or descope with an explicit ticket comment |
| SCOPE_CREEP | Beyond the ticket's intent | Expand ticket scope (with comment), new ticket, or remove the code |

Interactive: present each mismatch and resolve with the user. Unattended:
resolve conservatively (update the ticket for small drift; remove code for
genuine scope creep). **Do NOT proceed until ticket ⇄ spec ⇄ diff agree.**
Pushing misalignment just moves this conversation into `shadow-ci`'s red check.

## Phase 3: Commit & sync

```bash
git add -A
git commit -m "{IDENTIFIER}: {short description}"   # skip if nothing uncommitted

git fetch origin staging
git merge origin/staging --no-edit    # NEVER rebase pushed work
```

Conflicts: resolve preferring our branch unless staging's side is a bug fix.
Non-trivial conflict → STOP and ask the user. Re-run tests after the merge.

## Phase 4: Push, finalize PR, update ticket

```bash
git push origin "$(git branch --show-current)"
gh pr ready {number}
```

Write the final PR body — `shadow-ci` parses this structure strictly, and the
**Changes section must list EVERY file in `git diff origin/staging..HEAD --name-only`**
under its ticket (lockfiles too, e.g. "package-lock.json — regenerated for the
dependency bump"):

```markdown
## Summary
{what and why, 2-4 sentences — well over the 20-char hard gate}

## Tickets
| Ticket | Title | Status |
|--------|-------|--------|
| [{IDENTIFIER}]({URL}) | {TITLE} | In Review |

## Changes
### {IDENTIFIER}
- `path/one` — {what changed}
- `path/two` — {what changed}

## Test Plan
- [ ] {how this was verified — suite run, manual steps, screenshots on ticket}
```

Apply with `gh pr edit {number} --body-file /tmp/pr_body.md`.

Move the ticket to In Review: Linear MCP `save_issue(id, state: "In Review")`,
or `gh issue edit {N} --remove-label in-progress --add-label in-review`. Comment
the PR URL on the ticket if not already linked.

## Done

Tell the user:

```
Pushed and PR is ready for review. CI will now run shadow-ci.
  /shadow:fix-compliance   if the compliance check goes red
  /shadow:fix-pr           for review-bot findings
```
