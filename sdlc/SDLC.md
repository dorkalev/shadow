# The Dictated SDLC — simple, AI-first, SOC 2 readiness-shaped

This is the one software development lifecycle the shadow tool installs and enforces. It is deliberately opinionated: our users are 1–10-person AI-first startups, so we don't adapt to an existing process — we *are* the process. Every step exists because a specific criterion demands it (mapping at the bottom). Nothing here is ceremony.

Design lineage: a spec-first agent workflow (`start → load → finish → fix-compliance → fix-pr → release`, plus `hotfix`), backed by a deterministic per-PR compliance gate and post-merge archives with bypass detection, distilled to its minimum.

## Roles

There is exactly one human in the default operating model: the founder. AI tools, CI jobs, and service accounts are machine controls, not fictional employees and not organizational segregation of duties. The founder may develop and merge the same change. That concentration is disclosed and risk-accepted; protected pull requests, deterministic tests, restricted deploy credentials, tamper-evident archives, continuous monitoring, and periodic external examination are compensating controls. An optional read-only AI reviewer is an adviser, never an independent person or an approver.

## Branch topology

```
main      ← production source. Every ordinary change arrives through a protected PR.
{TICKET-ID}-{slug}  ← one branch per ticket, off main.
compliance-archives ← evidence-only branch: no force-push/deletion; Actions appends records.
```

## The loop (per change)

### 1. Ticket first
No change without a ticket (Linear or GitHub Issues). The ticket states intent and acceptance criteria. AI agents are given the ticket, not a vibe.
*Auditor translation: authorization of change (CC8.1).*

### 2. Branch + draft PR
Branch `{TICKET-ID}-{slug}` off `main`; open a **draft PR to main** immediately. The PR is the audit artifact from minute one.

### 3. Spec before code
The implementation plan is posted to the ticket as a comment **before** substantive code lands. The founder records the authorization and intended result; implementation may be produced by the founder, an AI coding tool, or both.
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
2. **Automated semantic review, when armed**: advisory AI findings are identified as machine output; unresolved CRITICAL/MAJOR findings block. An unavailable reviewer never counts as a completed review.
3. **Deterministic compliance gate**:
   - ticket(s) in PR title/body exist in the tracker (deterministic regex extraction — crash-safe),
   - every changed file traceable to a ticket via the Changes section,
   - test coverage for changed source files,
   - PR body ≥ 20 chars, structured,
   - confidence score: start 100; −10/invalid ticket; −10/unspecced file; −5/untested file; −5/missing explicitly-required automated adviser; **fail below 70**,
   - hard gates regardless of score: no valid ticket, empty description, unresolved critical/major review findings.

### 7. Merge ⇒ archive (automatic)
On merge, a workflow writes `pr-{n}-{ticket}-{date}.json` + `.md` to the `compliance-archives` branch: full PR metadata, reviews, comments, check runs, files changed, commits, the compliance report, and a **bypass analysis** — required checks are read from the *live* branch ruleset, and any merge that landed with a required check failed/missing is flagged `is_bypass: true` and announced (Slack). Bypasses aren't forbidden — emergencies exist — but they are never silent.
*Auditor translation: the complete, tamper-evident population of changes for examination sampling. Git history alone is not described as immutable.*

### 8. Founder release approval + keyless deploy
The founder merges the green PR to `main`; GitHub records the authenticated merge actor as management approval-of-record. CI deploys that exact commit using a restricted, keyless Workload Identity Federation principal. The deployer cannot change repository policy, and the coding tool receives no production credentials. This is not independent human approval; the disclosed solo-founder limitation remains.
*Auditor translation: management authorization + controlled implementation of the change (CC8.1).*

### 9. Hotfix (the emergency valve)
An emergency bypass costs paperwork by design: an **incident ticket** describing impact and why the protected flow was bypassed, an after-the-fact PR carrying the normal tests and documentation, and a bypass flag in the archive. An undocumented direct push is the one thing the shadow escalates loudest.
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
| Protected main + restricted keyless deploy | CC8.1, CC6.3 (least privilege over prod) |
| compliance-archives + bypass detection | CC8.1, CC4.1, CC2.1 (quality information) |
| Hotfix procedure | CC7.4, CC7.5, CC8.1 |
| Founder merge record + deployment record | CC8.1, CC2.2 |
| Access reviews, on/offboarding tickets | CC6.1–CC6.3 |
| Alert triage cadence | CC7.1, CC7.2 |
| Management review minutes | CC1.2, CC4.2 |
| Risk register + annual refresh | CC3.1–CC3.4, CC9.1 |
| Vendor register + reviews | CC9.2 |
| Policy repo (versioned, attested) | CC5.3, CC1.1, CC2.2 |
| Backup config + restore test log | A1.2, A1.3 |

## AI-specific rules

- Agents operate through the same gates as humans — same tickets, same PRs, same checks. No agent-only side doors.
- AI review is advisory. It never approves, never creates human independence, and an unavailable/failed run never receives review credit.
- In the solo-founder profile the founder may both author and merge; that concentration is disclosed, monitored, and accepted rather than hidden behind bot identities.
- Agent credentials are scoped tokens (no org admin), inventoried in the access register, rotated like any credential (CC6.1–6.3).
- Prompts/specs that drove a change live on the ticket — that's the design record for AI work (CC8.1).
