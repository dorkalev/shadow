---
description: Quarterly management review as a dialogue — walk the agenda, record decisions and owners, file the minutes (CC1.2/CC4.2)
---
# /shadow:ritual-mgmt — The Quarterly Management Review

Runs the quarterly management review (CC1.2/CC4.2) as an interview. For a
2-person startup **this dialogue IS the meeting** — say so to the user up
front: the questions below are the agenda, their answers are the minutes, and
the archive commit is the oversight evidence. Three phases: **gather** (zero
questions), **interview** (judgment only), **file** (zero questions).

**Segregation of duties:** the agent never answers its own agenda questions,
never records a decision nobody made, and attendees are the humans who
actually responded — the agent is not an attendee. Unanswered agenda items are
recorded **OPEN** with no decision.

## Phase 1: GATHER (no questions asked)

**Step 1 — generate the packet:**
```bash
ARCHIVES_PUSH=1 .shadow/ci/target/release/shadow-ci mgmt-packet
```
It lands under `evidence/{YYYY}/{QN}/` on `compliance-archives`. Read the
generated packet in full: gauge trend (last 13 readings), bypass merges this
quarter, open Dependabot/CodeQL/secret-scanning counts, incidents, open
shadow-regression tickets, agenda checklist, empty decisions/minutes blocks.

**Step 2 — enrich each agenda item with evidence** so no interview question
arrives naked:
- For each **bypass merge**: pull the archived record from
  `compliance-archives` (PR number, author, what check was missing/failed) and
  check whether an incident ticket already links to it (Linear MCP search or
  `gh issue list --search`).
- For each **alert count** (Dependabot/CodeQL/secret-scanning): list the items
  and their ages; flag those over the response SLA in the policy pack.
- For each **incident**: check whether a postmortem exists in the archives
  (`evidence/` postmortem files) or on the ticket.
- Scan the quarter's incidents, bypasses, and regressions for **risk-register
  candidates** — new risks or likelihood changes worth proposing.

## Phase 2: INTERVIEW (the meeting — item by item, evidence inline)

Walk the agenda in packet order. Batch where natural; show the evidence behind
every question; offer a sensible default for judgment calls — but never for
approvals.

1. **Gauge trend** — show the 13 readings. Any drop or plateau: *"Score fell
   from X to Y in {month}, coinciding with {evidence}. Accept this explanation,
   or is there another cause to record?"*
2. **Bypass merges** — per bypass: *"PR #N merged past {check} on {date} by
   {author}. Incident ticket: {link / NONE}."* No ticket is itself a decision
   to record — ask whether to open one now (default: yes, open it in Phase 3).
3. **Alerts over SLA** — per item: fix now (owner?), accept the risk (record
   why), or defer (new date). One decision each.
4. **Incidents** — per incident: postmortem filed? If not: *"File one this
   quarter (owner?), or record why none is needed?"*
5. **Risk register** — present the candidates from Phase 1: *"Add to the
   register / adjust likelihood / reject?"* Accepted changes become a PR
   against the register in the policies repo.
6. **Action items** — restate every action produced above and ask for an
   **owner and a due quarter** for each. An action without an owner is OPEN.
7. **Attendees** — *"Who is present for this review?"* Names on the record.

## Phase 3: FILE (no questions asked)

1. **Fill the packet's minutes/decisions blocks**: per-item decision text,
   owners, OPEN markers for anything unanswered, attendee names, date.
2. **Commit to `compliance-archives`** (worktree:
   `git worktree add /tmp/archives compliance-archives`), message
   `mgmt-review {YYYY}-{QN}: minutes filed — attendees {names}`, push.
3. **Open a ticket for every action item** — Linear MCP (`save_issue`) when
   available, otherwise `gh issue create` — title from the decision, body with
   the evidence, the recorded owner, and a link to the archived minutes.
   Bypass merges without incident tickets (where the humans said yes) get
   their incident ticket here too.
4. **Risk-register changes** the attendees accepted → branch + PR against the
   register in the policies repo (content changes to policies always go via
   PR, never direct commit). Link the PR from the minutes.
5. **Report a one-screen summary:** items reviewed, decisions made, tickets
   opened (IDs), register PR (if any), OPEN items, attendees, archive commit
   SHA.

## STOP conditions
- `mgmt-packet` fails → fix that first; do not hand-assemble the packet.
- Nobody answers → file the packet with all items OPEN, no attendees invented,
  and tell the user the review has not happened yet. Minutes written by the
  agent alone are not management oversight.
