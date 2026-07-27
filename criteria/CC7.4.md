---
id: CC7.4
family: CC7 — System Operations
category: Security (Common Criteria)
coso: null
title: Incident Response Program Execution
weight: 3
automatable: partial
nature: technical
---

# CC7.4 — Incident Response Program Execution

## Criterion (AICPA TSP Section 100, verbatim)
> The entity responds to identified security incidents by executing a defined incident-response program to understand, contain, remediate, and communicate security incidents, as appropriate.

## What it means

Once CC7.3 declares something an incident, CC7.4 requires that what happens next is a *program*, not improvisation: a written plan that assigns roles, walks through understand → contain → remediate → communicate, and is actually followed. "Defined" is the key word — the auditor tests both that the plan exists and that real incidents (if any occurred) followed it, or that you exercised it if none did.

For a 1–10 person company the plan is short and honest: the on-call/founder is incident commander, containment playbooks cover your actual attack surface (revoke GCP service-account key, rotate secret, disable compromised Google Workspace account, roll back deploy, take service offline), communication rules cover customers, and — importantly for AI-first startups — regulators/DPAs if personal data is involved. Don't write an enterprise IR plan with a CISO and a war room you don't have; auditors flag plans that can't be executed by the org that wrote them.

The two artifacts that make or break this criterion in a Type II: incident tickets showing the plan's phases were executed with timestamps, and a **postmortem** for each SEV1/SEV2. If you had zero incidents in the period (plausible for a small startup), you need a **tabletop exercise** — one hour, a written scenario ("Anthropic API key leaked in a public repo"), notes on who did what, and lessons filed as tickets. That exercise is your operating-effectiveness evidence.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Assigns roles and responsibilities for incident response, including designated leadership for response activities.
- Contains and responds to security incidents using defined procedures; mitigates ongoing harm.
- Ends threats posed by incidents through containment, then remediates identified vulnerabilities exploited by the incident.
- Communicates incidents to affected parties — internal stakeholders, customers, regulators — as required by policy, contract, or law.
- Restores operations affected by the incident and evaluates the effectiveness of the response (feeds CC7.5).
- Develops and tests the incident-response plan periodically; improves it from lessons learned.

## What the auditor will ask for
- The incident response plan: roles, severity levels, phase-by-phase procedures, communication/notification matrix (including breach-notification obligations), and evidence of annual review/approval.
- The incident register for the period; for each incident, the full ticket trail: detection time, containment actions and times, remediation PRs, and closure.
- Postmortem documents for significant incidents (root cause, timeline, action items with owners).
- Customer/regulator notifications sent, if any, or the documented determination that none were required.
- Evidence of IR plan testing: tabletop exercise notes with date and participants (mandatory if no real incidents occurred).
- Evidence that postmortem action items were completed (closed Linear tickets with merged PRs).

## How a tiny AI-first startup satisfies it
- **`policies/incident-response.md`**: one document covering CC7.2–7.5 — severity matrix, IC assignment, containment playbooks written against real infrastructure (exact `gcloud iam service-accounts keys disable` style commands, GitHub token revocation steps, Workspace account suspension), a communication matrix (who tells customers, in what timeframe, template included), and a postmortem requirement for SEV1/SEV2.
- **Linear incident tickets** using an incident template: phases as checklist (Understand / Contain / Remediate / Communicate / Recover), timestamps per phase, IC named. The ticket is the incident record; Slack threads get linked, not relied on.
- **Remediation through the SDLC**: every remediation change is a normal CC8.1 flow (ticket → branch → PR → review → CI → merge → archive). For true emergencies, the documented **hotfix backport procedure** applies: push fix to main, then immediately open a backport PR to staging with the incident ticket attached — the compliance shadow's bypass detector flags the direct push and the backport PR closes the loop, converting an SDLC violation into documented emergency change evidence.
- **Postmortems** live in-repo (`postmortems/YYYY-MM-DD-slug.md`), blameless, with action items as linked Linear tickets. The shadow verifies every SEV1/SEV2 ticket has one.
- **Annual tabletop**: scheduled Linear ticket; scenario + notes committed to `evidence/ir-exercises/`. AI can generate the scenario; humans run it.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| IR plan exists, has required sections, reviewed <12mo | repo | file-existence + section grep + frontmatter review date |
| Every `incident` ticket uses the phase template | Linear | Linear API: template/checklist field presence |
| Containment timestamps within policy SLA per severity | Linear | detection→containment timestamp delta on incident tickets |
| SEV1/SEV2 incidents have a postmortem file | Linear+repo | incident tickets cross-referenced to `postmortems/*.md` |
| Postmortem action items exist and get closed | Linear | linked-ticket status query, staleness alert >30d |
| Remediation PRs traceable to incident tickets and archived | GitHub | PR body ticket-table parse + `compliance-archives` record lookup |
| Hotfix pushes to main have backport PRs + incident/hotfix ticket | GitHub | bypass detector: direct-push commits on main vs backport PR to staging referencing them |
| Annual tabletop exercise completed | Linear+repo | recurring ticket done + `evidence/ir-exercises/` file with date in period |
| Communication decision recorded per incident | Linear | "Communicate" checklist item completed with note |
| Adequacy of notifications vs contractual/legal duty | — | MANUAL — legal judgment, auditor reviews determination memos |

## Evidence artifacts
- `policies/incident-response.md` — the defined program, with version history in git.
- Linear: `incident` tickets with phase checklists and timestamps; action-item tickets.
- `postmortems/*.md` — blameless postmortems, git-tracked.
- `evidence/ir-exercises/YYYY-MM-DD-tabletop.md` — exercise scenario, participants, notes, lessons.
- `compliance-archives` branch — archive records for remediation and hotfix-backport PRs, plus bypass-detection reports showing emergency changes were reconciled.
- Sent notification copies (email exports) attached to incident tickets, when applicable.
