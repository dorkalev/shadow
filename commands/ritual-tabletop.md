---
description: Annual incident tabletop run by the agent — a fresh scenario for this stack, staged live against the runbook, gaps ticketed and the exercise record archived (CC7.4/CC7.5)
---
# /shadow:ritual-tabletop — The Annual Incident Tabletop

Runs the annual incident-response exercise (CC7.4 response, CC7.5 recovery /
post-incident) with the agent as facilitator. Three phases, strictly ordered:
**gather** (zero questions), **interview** — which IS the exercise — and
**file** (zero questions). State plainly to the user at the start and in the
record: **this exercise satisfies the tabletop evidence auditors ask for under
CC7.5.**

**Segregation of duties — absolute:** the agent facilitates and scores; it
NEVER answers a stage on the participants' behalf, never fills in "what they
would probably do", and participants' names go on the record. A stage with no
human answer is recorded **OPEN — no response captured**, and counts as a gap.

## Phase 1: GATHER (no questions asked)

**Step 1 — read the response machinery:** the incident-response runbook and
the hotfix runbook (`/shadow:hotfix` path) from the policies/SDLC repos. Note
the runbook's expected sequence — detect, triage/sever, contain, communicate
(internal + customer thresholds), eradicate, recover, post-mortem — and any
named roles, channels, or SLAs. These are the answer key.

**Step 2 — read the real architecture:** the system description (if drafted)
and/or the scan report from `agent/01-scan-platform.md` output — actual cloud
provider, data stores, LLM vendors, CI/CD, secret stores.

**Step 3 — check archives for prior exercises:**
`git fetch origin compliance-archives` then search `evidence/*/` for previous
tabletop records. **Never reuse last year's scenario.**

**Step 4 — generate the scenario for THIS stack:** plausible, specific, and
grounded in real components (e.g. a leaked service-account key from a CI log +
anomalous egress from the production Cloud SQL instance; or a compromised
GitHub App token pushing to a repo; or an LLM-vendor breach notice covering
prompts containing customer data). Write 4 stages with concrete fake artifacts
(log lines, alert text, timestamps) and ONE mid-exercise complication (e.g.
the person with the needed GCP role is unreachable; the revocation also kills
production auth). Keep the complication secret until stage 3.

## Phase 2: INTERVIEW — the exercise itself

First record participants: "who is at the table?" (names — they go on the
record). Then run the stages, one at a time, in character:

> **Stage 1 — 09:14.** Billing alert: egress from `prod-sql` is 40x baseline.
> Simultaneously, secret-scanning flags a service-account key in a public gist.
> **What do you do, right now?** (who acts, what command/console, what first?)

For each stage capture the response verbatim, then ask the four probes the
runbook must survive: **detect** (how would you have noticed without this
alert?), **contain** (exact revocation/isolation step), **communicate** (who
is told, when do customers hear, who decides?), **recover** (path back to
known-good, and how do you know it's clean?).

Inject the complication mid-way (stage 3). After the final stage, compare
every answer against the runbook's answer key: where they matched, where they
improvised something better than the runbook says, where nobody knew — each
mismatch is a **gap** (runbook gap or knowledge gap, note which).

Do not correct participants mid-stage; the scoring happens at the end, in the
open, with the runbook text quoted.

## Phase 3: FILE (no questions asked)

1. **Exercise record to archives** under `evidence/{YYYY}/` on
   `compliance-archives`: scenario (all stages + complication), participants,
   date/duration, verbatim responses per stage, the runbook comparison, gaps
   found, and the CC7.4/CC7.5 statement that this is the annual tabletop.
   Commit message: `tabletop {YYYY}: {n} participants, {n} gaps`.
2. **Runbook amendments as a PR** to the policies repo through the normal
   gates — one PR containing the concrete text changes the exercise exposed
   (missing revocation command, undefined customer-comms threshold, etc.).
   Only propose amendments grounded in a recorded gap.
3. **One ticket per gap** — Linear MCP (`save_issue`) when available,
   otherwise `gh issue create`: the gap, the stage that exposed it, owner if
   named during the exercise, link to the archived record.
4. **One-screen summary:** scenario headline, participants, stages completed,
   gaps (with ticket IDs), runbook amendments PR URL, archive commit SHA, and
   the reminder that next year's run must use a new scenario.

## STOP conditions
- No incident-response runbook exists → stop; a tabletop without a runbook
  tests nothing. Open a ticket to write it first (CC7.4 gap).
- No participants respond past stage 1 → file the record as ABORTED with what
  was captured, all remaining stages OPEN, and one ticket to reschedule. An
  aborted exercise is evidence of an attempt, not of a control.
