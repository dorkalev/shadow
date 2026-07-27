---
description: Incident postmortem scaffold — draft the factual half from evidence, interview only for judgment, file to archives with prevention tickets (CC7.3/CC7.5)
---
# /shadow:postmortem — Facts First, Judgment Asked

Builds an incident postmortem (CC7.3/CC7.5) where the factual half is
assembled from evidence and only the judgment half is asked. **Args:** the
incident ticket ID (`/shadow:postmortem OPS-42` or `#87`).

**The one absolute rule: never invent facts for the timeline.** Every timeline
entry cites its source (ticket comment, PR event, commit, alert). Where the
evidence has a hole — detection time unknown, no record of who paged whom —
the entry is marked **GAP** and either asked about in the interview or left
**OPEN** in the filed document. A postmortem with honest gaps beats a smooth
fiction.

Three phases: **gather** (zero questions), **interview** (judgment only),
**file** (zero questions). Segregation of duties: root cause, severity, and
prevention are the humans' calls — the agent proposes candidates from
evidence, never decides, and the responding human's name goes on the record.

## Phase 1: GATHER (no questions asked)

1. **The ticket + full comment timeline.** Linear MCP (`get_issue`,
   `list_comments`) when available, otherwise
   `gh issue view {N} --comments --json title,body,createdAt,closedAt,comments`.
   Extract every timestamped event.
2. **Related PRs and hotfix backports.** Search the ticket ID across PRs
   (`gh pr list --search "{ID}" --state all`) and grep the
   `compliance-archives` branch for records referencing the ticket — hotfix
   incidents (`/shadow:hotfix` output), bypass-flagged merges, and the archived
   PR records give exact merge times, authors, and diffs:
   ```bash
   git fetch origin compliance-archives
   git grep -l "{TICKET-ID}" origin/compliance-archives
   ```
3. **Alert/monitoring context** if referenced from the ticket or PRs (linked
   alert IDs, pasted graphs, Slack excerpts quoted in comments). Only what is
   actually referenced — do not guess at telemetry.
4. **Draft the factual half automatically** into a working postmortem:
   - **Timeline** — merged, chronological, every entry `[source]`-cited;
     holes marked **GAP: {what's missing}**.
   - **Impact window** — first evidence of impact → confirmed resolution, from
     the record only (GAP if either end is unevidenced).
   - **What was changed** — the fix commits/PRs/backports, from the archives.

## Phase 2: INTERVIEW (judgment only, batched, evidence inline)

Present the draft, then ask exactly these:

1. **GAPs** — list each timeline gap: *"Can you fill this from memory, or does
   it stay OPEN?"* (A remembered fact is recorded as *"per {name}"* — still
   sourced.)
2. **Root cause** — offer candidates read from the evidence (*"the fix touched
   only {file}, suggesting {candidate A}; the alert lag suggests {candidate
   B}"*): *"Which is the root cause — or state another?"*
3. **Why defenses missed it** — walk the gates the change passed (review,
   tests, shadow-ci, alerts): *"Each of these was green/silent — why?"*
4. **Prevention actions** — propose candidates, ask for the accepted list
   **with an owner each**. Every accepted action becomes a ticket.
5. **Severity** — propose a severity from the impact window and ask for
   confirmation or correction.

Collect the responding human's name for the record. Unanswered items → OPEN.

## Phase 3: FILE (no questions asked)

1. **Write the postmortem** as
   `evidence/{YYYY}/{QN}/postmortem-{TICKET-ID}-{date}.md` on
   `compliance-archives` (worktree:
   `git worktree add /tmp/archives compliance-archives`): timeline (with
   sources and any remaining GAP/OPEN marks), impact window, what changed,
   root cause, why defenses missed it, severity, prevention actions with
   owners and ticket IDs, participant names, date. Commit
   `postmortem {TICKET-ID}: filed by {name}`, push.
2. **Link it from the incident ticket** — Linear comment (or attachment) with
   the archive path and commit SHA, otherwise `gh issue comment`. If the
   incident is resolved and no OPEN items remain, note the ticket can close.
3. **Open one ticket per prevention action** — Linear MCP or
   `gh issue create` — body carries the evidence line that motivated it, the
   owner from the interview, and a link back to the postmortem.
4. **Report a one-screen summary:** severity, impact window, root cause (or
   OPEN), prevention tickets (IDs + owners), remaining GAP/OPEN items, archive
   commit SHA.

## STOP conditions
- Ticket ID doesn't resolve in the tracker → stop; never fabricate an incident.
- The ticket has no usable timeline and no linked PRs → file a stub postmortem
  that is mostly GAP markers and say so — that emptiness is itself a CC7.3
  finding about incident recording.
- Nobody answers the interview → file the factual half with root cause,
  severity, and prevention all OPEN; the facts are preserved while fresh.
