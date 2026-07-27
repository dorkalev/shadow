---
description: Command map for the compliance shadow SDLC — which command to run when
---
# Shadow Commands

These commands live in `.claude/commands/shadow/` and drive the dictated SDLC
(see `sdlc/SDLC.md`). They are the entire per-change workflow: if you are doing
development work in this repo, you are inside one of these commands.

## The commands

| Command | When to use |
|---|---|
| `/shadow:capture` | A feature was just planned in conversation. Distills it into a ticket (problem, approach, acceptance criteria) with the spec as first comment. |
| `/shadow:start` | Beginning any piece of work. Picks or creates a ticket, branches off `staging`, opens a draft PR. No ticket, no branch. |
| `/shadow:load` | After start (or when handed a ticket). Researches the codebase, posts the implementation spec to the ticket, then implements within it. |
| `/shadow:verify` | UI-facing work is implemented. Walks each acceptance criterion in a real browser, screenshots to `.shadow-evidence/`, posts the verified story to the ticket (CC8.1 "tested"). |
| `/shadow:finish` | Work is done locally. Cleanup, spec-alignment gate, commit, sync with staging, push, mark PR ready, ticket to In Review. |
| `/shadow:fix-compliance` | The `shadow-ci` check failed on the PR. Reads the actual CI report and fixes each failure type mechanically. |
| `/shadow:fix-pr` | The review bot left findings. Triages, fixes Critical/Major, loops until clean. |
| `/shadow:release` | Human decides to ship. Promotes green `staging` into `main` (ff-only) with a full release record in `compliance-archives`. |
| `/shadow:hotfix` | Someone pushed directly to `main` (emergency). Files the incident ticket and backport PR that the bypass detector will demand. |

## The rituals (interview commands)

Everything SOC 2 traditionally makes humans do by hand, reduced to a dialogue. Every ritual follows the same three-phase pattern — **GATHER** (zero questions: the tool collects every discoverable fact via `shadow-ci`, `gh`, `gcloud`, registers, archives), **INTERVIEW** (judgment only: batched questions with the evidence attached to each, sensible defaults offered), **FILE** (zero questions: signed artifact committed, follow-up tickets opened, one-screen summary). Segregation of duties is the design, not the excuse: the agent never answers its own questions, never defaults an approval to yes, and the responding human's name goes on the record. Unanswered items are filed as OPEN, never assumed.

| Command | Cadence | What the human's job shrinks to |
|---|---|---|
| `/shadow:ritual-access` | quarterly | Approve/flag the *new* grants since last quarter; one blanket confirm for unchanged. ~10 min. (CC6.2/6.3) |
| `/shadow:ritual-mgmt` | quarterly | The dialogue *is* the management review meeting: accept/challenge each pre-filled agenda item, name decisions + owners. (CC1.2/4.2) |
| `/shadow:ritual-risks` | annual | Confirm/adjust each risk's rating; accept/reject candidates derived from the year's real incidents, bypasses, and drift. (CC3.x, CC9.1) |
| `/shadow:ritual-vendors` | annual | Per vendor: still used, risk rating, DPA/SOC 2 current? New vendors auto-detected from GitHub Apps + secrets. (CC9.2, P6.4/6.5) |
| `/shadow:ritual-policies` | annual | Reapprove policies (diffs shown); staff sign attestation issues carrying auto-generated micro-training + quiz. (CC5.3, CC1.4) |
| `/shadow:ritual-tabletop` | annual | Play the incident scenario the agent runs against your real stack; gaps become tickets. (CC7.4/7.5) |
| `/shadow:onboard` | per joiner | Confirm the least-privilege grant set; the checklist ticket, MFA step, attestation issue, and day-14 verification are generated. (CC6.2) |
| `/shadow:offboard` | per leaver | Say y/n per system; revocation commands execute, before/after proof files itself. (CC6.2) |
| `/shadow:postmortem` | per incident | Answer only root cause + prevention; timeline and impact are drafted from evidence. (CC7.3/7.5) |
| `/shadow:system-description` | once + on change | Answer only commitments/CUECs/boundaries; DC1–DC7 draft assembles from what the scan already knows. |
| `/shadow:audit-binder` | when the audit starts | Answer only "where is X?" for missing artifacts; the binder index + sampling populations build themselves. |

## The flow

```
(planning conversation) ──► /shadow:capture
                                  │
                                  ▼
/shadow:start ──► /shadow:load ──► /shadow:verify ──► /shadow:finish ──► CI runs shadow-ci
                                   (browser evidence,
                                    UI-facing changes)
                                                          │
                              ┌───────────────────────────┤
                              ▼                           ▼
                    /shadow:fix-compliance          /shadow:fix-pr
                    (compliance check red)        (review findings)
                              │                           │
                              └───────────┬───────────────┘
                                          ▼
                                   PR merged → auto-archived to
                                   compliance-archives (bypass-checked)
                                          │
                                          ▼
                                   /shadow:release  (staging → main, human-confirmed)

Emergency path (direct push to main):
  hotfix lands ──► /shadow:hotfix (incident ticket + backport PR) ──► normal gates
```

## Branch topology (fixed — never improvise)

```
main                 production. Fast-forward only, from staging. Never pushed directly (except documented hotfix).
staging              integration. ALL PRs target staging.
{TICKET-ID}-{slug}   one branch per ticket, off staging.
compliance-archives  append-only evidence branch. Never merged anywhere.
```

## Tracker

Ticket IDs match `[A-Z]{2,6}-[0-9]+` (Linear) or `#N` (GitHub Issues). Every
command works with either: use Linear MCP tools when they are available in the
session, otherwise `gh issue` commands. Never fabricate a ticket ID — always
extract it from the tracker's response.

## Philosophy

Evidence is a side effect of working, not a phase. Every change moves
ticket-first through a draft PR, a spec comment, tests, an independent review,
and the `shadow-ci` compliance check — so the audit trail writes itself at the
time the work happens, never reconstructed later. Agents go through the same
doors as humans: same tickets, same PRs, same gates, and never self-approval —
the approving identity (human or independent review bot) must differ from the
authoring identity. When a gate is red, the fix is to satisfy the gate, never
to weaken it; a gate change is itself a ticketed, reviewed change.
