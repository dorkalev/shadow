# The Dictated SDLC — simple, AI-first, SOC 2-compliant

This is the one software development lifecycle the shadow tool installs and enforces. It is deliberately opinionated: our users are 1–10-person AI-first startups, so we don't adapt to an existing process — we *are* the process. Every step exists because a specific criterion demands it (mapping at the bottom). Nothing here is ceremony.

Design lineage: a spec-first agent workflow (`start → load → finish → fix-compliance → fix-pr → release`, plus `hotfix`), backed by a deterministic per-PR compliance gate and post-merge archives with bypass detection, distilled to its minimum.

## Roles

At minimum one human (the founder) and one or more AI agents. **Honest SoD note:** when the only reviewer is an automated agent (no independent human), segregation of duties is not fully satisfied — this is a *compensating control* (branch protection, immutable archives, release confirmation, post-hoc review) and a **disclosed limitation** in the system description, not a claim of satisfied SoD. Add a human non-author approver on production the moment a second person exists. The agent may write all the code; the human owns approvals that need a person: merging to production, releases, incident declarations, quarterly reviews. Where a second reviewer is impossible, an independent review bot (a second LLM with no ability to push) plus the compliance agent stand in — and the report's system description says so honestly.

## Branch topology

```
main      ← production. Fast-forward only, from staging. Never pushed directly (except documented hotfix).
staging   ← integration. All PRs target staging.
{TICKET-ID}-{slug}  ← one branch per ticket, off staging.
compliance-archives ← append-only evidence branch (never merged anywhere).
```

## The loop (per change)

### 1. Ticket first
No change without a ticket (Linear or GitHub Issues). The ticket states intent and acceptance criteria. AI agents are given the ticket, not a vibe.
*Auditor translation: authorization of change (CC8.1).*

### 2. Branch + draft PR
Branch `{TICKET-ID}-{slug}` off `staging`; open a **draft PR to staging** immediately. The PR is the audit artifact from minute one.

### 3. Spec before code
The implementation plan is posted to the ticket as a comment **before** substantive code lands. For AI development this is the crucial inversion: the human (or reviewing agent) approves the *spec*, then the AI implements. Cheap at this size, priceless at audit.
*Auditor translation: design + documentation of change (CC8.1).*

### 4. Implement, test, self-verify
Code + tests in the same PR. New source files need corresponding tests (the compliance agent checks per-file). UI-facing changes get a browser-driven verification with screenshots attached to the ticket.

### 5. PR body = traceability record
Required structure — the compliance agent parses this:

```
## Summary
## Tickets
| Ticket | Title | Status |
## Changes            ← EVERY changed file listed under its ticket
## Test Plan
```

### 6. Gates (required status checks — merges are impossible without them)
1. **CI**: build + tests green.
2. **Independent review**: review bot must post; unresolved CRITICAL/MAJOR findings block.
3. **Compliance agent** (the shadow's per-PR auditor):
   - ticket(s) in PR title/body exist in the tracker (deterministic regex extraction — crash-safe),
   - every changed file traceable to a ticket via the Changes section,
   - test coverage for changed source files,
   - PR body ≥ 20 chars, structured,
   - confidence score: start 100; −10/invalid ticket; −10/unspecced file; −5/untested file; −5/missing reviewer; **fail below 70**,
   - hard gates regardless of score: no valid ticket, empty description, unresolved critical/major review findings.

### 7. Merge ⇒ archive (automatic)
On merge, a workflow writes `pr-{n}-{ticket}-{date}.json` + `.md` to the `compliance-archives` branch: full PR metadata, reviews, comments, check runs, files changed, commits, the compliance report, and a **bypass analysis** — required checks are read from the *live* branch ruleset, and any merge that landed with a required check failed/missing is flagged `is_bypass: true` and announced (Slack). Bypasses aren't forbidden — emergencies exist — but they are never silent.
*Auditor translation: the complete, tamper-evident population of changes for Type II sampling.*

### 8. Release (staging → main)
Human-triggered. Preconditions: CI green on staging, no unexplained bypasses. The release run: builds a summary of commits/PRs/tickets since last release → human confirms by typing a random confirmation word → writes `releases/release-{date}.json+md` to `compliance-archives` → creates a release ticket listing everything shipped → `git merge --ff-only` staging into main → comments the release link on every included ticket. If fast-forward fails, stop — never force-push.
*Auditor translation: approval + implementation of change into production (CC8.1), segregation of duties in spirit (CC6.3).*

### 9. Hotfix (the emergency valve)
Direct push to `main` allowed only for genuine emergencies, and it costs paperwork by design: an **incident ticket** (what broke, impact, root cause, why the process was bypassed), a **backport PR** to staging cherry-picking the fix through the normal gates, and the bypass flag in the archive. An undocumented direct push is the one thing the shadow escalates loudest.
*Auditor translation: exceptions exist but are documented and remediated (CC7.4, CC8.1).*

## The clock (not per-change)

| Cadence | Ritual | Criterion |
|---|---|---|
| Daily (automated) | Shadow verify run: all automated checks, gauge recompute, regression tickets | CC4.1 |
| Weekly (automated) | Dependabot/secret-scanning/CodeQL alert triage ticket if any open criticals | CC7.1 |
| Quarterly (human, ~1h) | Access review (org members, cloud IAM, tracker seats — export, confirm, sign), vendor register review | CC6.2/6.3, CC9.2 |
| Quarterly (human, ~1h) | Management review: gauge trend, incidents, exceptions, risk register touch-up. Minutes filed as evidence | CC1.2, CC4.2 |
| Annually | Risk assessment refresh, policy re-approval + re-attestation by all staff, restore-from-backup test, incident tabletop | CC3.x, CC5.3, A1.3, CC7.5 |
| On join/leave | Onboarding/offboarding checklist ticket (access grants/revocations enumerated, MFA verified) | CC6.2 |

## Criterion coverage map

| SDLC element | Criteria satisfied (evidence generated) |
|---|---|
| Ticket-first + spec comment | CC8.1 (authorize, design, document) |
| PR gates (CI, review, compliance agent) | CC8.1 (test, approve), CC4.1 (ongoing evaluation) |
| Branch protection / rulesets, ff-only main | CC8.1, CC6.3 (least privilege over prod) |
| compliance-archives + bypass detection | CC8.1, CC4.1, CC2.1 (quality information) |
| Hotfix procedure | CC7.4, CC7.5, CC8.1 |
| Release records + tickets | CC8.1, CC2.2 |
| Access reviews, on/offboarding tickets | CC6.1–CC6.3 |
| Alert triage cadence | CC7.1, CC7.2 |
| Management review minutes | CC1.2, CC4.2 |
| Risk register + annual refresh | CC3.1–CC3.4, CC9.1 |
| Vendor register + reviews | CC9.2 |
| Policy repo (versioned, attested) | CC5.3, CC1.1, CC2.2 |
| Backup config + restore test log | A1.2, A1.3 |

## AI-specific rules

- Agents operate through the same gates as humans — same tickets, same PRs, same checks. No agent-only side doors.
- Agent-authored changes are never self-approved: the approving identity (human or independent bot) must differ from the authoring identity.
- Agent credentials are scoped tokens (no org admin), inventoried in the access register, rotated like any credential (CC6.1–6.3).
- Prompts/specs that drove a change live on the ticket — that's the design record for AI work (CC8.1).
