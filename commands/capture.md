---
description: Distill the current planning conversation into a ticket — title, problem, approach, acceptance criteria — with the technical spec as the first comment
---
# /shadow:capture — Planning Conversation → Ticket

Run after a feature or fix has been discussed and planned in this conversation.
The ticket is the durable record of what was agreed; the conversation is not.
Capturing it *before* any code exists is what makes the ticket-first audit
trail honest.

**Hard rule: never invent decisions that weren't made.** If the conversation
left something undecided, it goes under **Open questions** — an invented
decision in a ticket is a fabricated design record, which is worse than a gap.

## Phase 1: Distill the conversation

Re-read the discussion and extract:
- **Title** — imperative, specific (`Add rate limiting to /api/export`, not "API improvements").
- **Problem statement** — WHAT is wrong or missing, and WHY it matters. From the conversation, not embellished.
- **Agreed approach** — the HOW that was actually settled on, 2-5 sentences.
- **Acceptance criteria** — checkboxes, each independently checkable, phrased as observable outcomes (these are what `/shadow:verify` will later walk in a browser).
- **Out of scope** — everything explicitly deferred or rejected in the discussion.
- **Open questions** — anything raised but not resolved. Do not resolve them yourself.

No clear feature in the conversation → STOP and ask the user what to capture.

## Phase 2: Create the ticket

Ticket body:

```markdown
## Problem
{problem statement}

## Approach
{agreed approach}

## Acceptance criteria
- [ ] {observable outcome}
- [ ] {observable outcome}

## Out of scope
- {explicitly excluded}

## Open questions
- {unresolved — decide before/while implementing}
```

- **Linear MCP:** `save_issue(title, description, teamId)` — pick the team via
  `list_teams` if ambiguous (interactive: ask the user; also ask priority/labels
  if the conversation didn't set them).
- **GitHub Issues:** `gh issue create --title "..." --body-file /tmp/ticket.md`
  in the code repo.

**Extract the ticket ID from the tracker's response** (`[A-Z]{2,6}-[0-9]+` or
`#N`). Never fabricate or guess an ID. Creation fails → STOP, report the
error, post nothing else.

## Phase 3: Post the technical spec as the first comment

Everything technical the conversation produced goes in a comment, not the
ticket body — same shape `/shadow:load` will later refine:

```markdown
## Technical Spec (from planning)

**Overview:** {how the approach maps to this codebase}

**Files likely to change:**
- `path/to/file` — {what changes}

**Edge cases discussed:** {from conversation}

**Testing strategy:** {from conversation}

**Open questions:** {carried over — still open}
```

- Linear MCP: `save_comment(issueId, body)`.
- GitHub Issues: `gh issue comment {N} --body-file /tmp/spec.md`.

Only include sections the conversation actually covered — omit, don't pad.
Do NOT save local issue/spec files; the ticket is the record.

## Phase 4: Report

Tell the user:

```
Captured: {TICKET-ID} — {title}
{ticket URL}
Spec posted as first comment. Open questions: {n or none}.

Next: /shadow:start {TICKET-ID}   (branch off staging + draft PR, then load)
```

## STOP conditions
- No plannable content in the conversation → STOP, ask.
- Tracker unavailable (no Linear MCP and no `gh` repo) → STOP, report; do not
  stash the ticket as a local file.
- Issue creation fails → STOP; never post a spec comment to a ticket that
  doesn't exist.
