---
description: Fetch the ticket, research the codebase, post the implementation spec, then implement within it
---
# /shadow:load — Spec, Then Build

The crucial inversion for AI-era development: the spec is approved **before**
the code exists. The spec comment on the ticket is the design record (CC8.1) —
implementation that isn't in it doesn't ship.

## Phase 1: Fetch the ticket

Argument is a ticket ID (`[A-Z]{2,6}-[0-9]+` or `#N`). Read the full ticket
including existing comments (a spec may already exist from a prior session):
- Linear MCP: `get_issue(id)` + `list_comments(issueId)`.
- GitHub Issues: `gh issue view {N} --json title,body,url,state --comments`.

If no argument, derive the ticket from the current branch name
(`{TICKET-ID}-{slug}`); if the branch doesn't encode a ticket → STOP and run
`/shadow:start` first.

## Phase 2: Research the codebase

Before writing anything, understand where the change lands:
- Find the files/modules the ticket touches (grep for relevant names, read the entry points).
- Find existing patterns to follow (how similar features are structured, where their tests live).
- Note constraints: test framework, lint rules, module boundaries.

Keep it proportional — a one-line fix needs minutes of research, not an essay.

## Phase 3: Post the spec — BEFORE substantive code

Write the implementation spec and post it as a **ticket comment**:

```
## Implementation Spec

**Approach:** {2-5 sentences: what will change and why this way}

**Files to change:**
- `path/to/file` — {what changes}
- `path/to/new_file` — NEW — {purpose} (test: `path/to/test_file`)

**Tests:** {which tests are added/updated, what they prove}

**Out of scope:** {explicitly excluded things, if any}
```

- Linear MCP: `save_comment(issueId, body)`.
- GitHub Issues: `gh issue comment {N} --body-file /tmp/spec.md`.

**Interactive session:** show the spec to the user and get approval before
implementing. **Unattended:** post it and proceed — the spec still predates the
code, which is what the audit trail needs.

Trivial mechanical fixes (typo, obvious one-liner) still get a spec — it can be
three lines. Do not skip the comment.

## Phase 4: Implement within the spec

- Implement exactly what the spec says, following the patterns found in Phase 2.
- **Every new source file gets a corresponding test file** — the `shadow-ci`
  checker scores −5 per untested source file. Write the test in the same
  session, not "later".
- Run the test suite and linter locally before declaring done.
- Commit incrementally with messages referencing the ticket: `{IDENTIFIER}: {what}`.

## Scope growth rule (hard)

If, mid-implementation, you discover the change must grow beyond the spec:
1. STOP writing code.
2. Update the ticket first — either edit the spec comment / post a revised spec
   (Linear `save_comment`, or `gh issue comment`), or if it's genuinely separate
   work, create a new ticket for it and leave it out of this branch.
3. Interactive: confirm with the user. Then resume.

Never let the diff outrun the ticket — `/shadow:finish` will block on exactly
that misalignment, so fixing it now is strictly cheaper.

## STOP conditions
- Ticket not found in tracker → STOP.
- Spec cannot be posted (tracker API failure) → STOP; do not implement unspecced.
- Acceptance criteria contradict what the code research shows is possible → STOP, discuss on the ticket, do not silently reinterpret.

When implementation is complete and tests pass, tell the user to run
`/shadow:finish`.
