---
id: CC2.2
family: CC2 — Communication and Information
category: Security (Common Criteria)
coso: COSO Principle 14
title: Internal Communication of Objectives
weight: 2
automatable: partial
nature: document
---

# CC2.2 — Internal Communication of Objectives

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 14: The entity internally communicates information, including objectives and responsibilities for internal control, necessary to support the functioning of internal control.

## What it means
Everyone inside the company must know what the security objectives are, what their own responsibilities are, and how to raise problems. In practice the auditor tests three things: (1) policies and responsibilities are communicated — people acknowledged them and can find them; (2) operational security information flows — incidents, changes, and alerts are communicated to the people who need to act; (3) there is a channel to report concerns, including one that bypasses management (a whistleblower path — for a startup, naming an external advisor or counsel as the alternate contact is the accepted equivalent).

At 1–10 people, "communication" is mostly ambient — everyone is in the same Slack. Auditors accept that, but they still want artifacts: the policies repo is discoverable and linked from onboarding, security topics appear in written form (Slack posts, Linear tickets, changelog), incident response communication paths are written in the IR plan, and new joiners demonstrably received the material. The trap for tiny teams is having everything in founders' heads; this criterion is where that gets flagged.

Include your AI-tooling ground rules in what's communicated: if the acceptable-use policy has AI clauses, show they were pushed to the team (announcement + acknowledgment), not just merged silently.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Communicates internal control information** — a process communicates required information to enable personnel to understand and carry out their internal control responsibilities, including objectives and changes thereto.
- **Communicates with the board of directors** — information relevant to oversight flows between management and the board (ties to CC1.2 review packets).
- **Provides separate communication lines** — whistleblower or equivalent channels exist for anonymous/confidential communication when normal channels are inoperative.
- **Selects relevant method of communication** — timing, audience, and medium are appropriate to the information.
- **Communicates responsibilities and system information** (TSC supplemental) — personnel are informed of their system responsibilities, system objectives/commitments, system changes, and how to report security failures, incidents, and concerns.

## What the auditor will ask for
- Evidence that policies were distributed and acknowledged (overlaps CC1.1 — acknowledgment records for new versions).
- Onboarding materials showing where policies live and what new joiners must read.
- The incident response plan's internal communication/escalation section, and evidence it was followed for any incident (or tabletop-exercise notes if none).
- Evidence of security communications during the period: announcement of policy changes, security reminders, post-incident notes (Slack exports/screenshots, Linear tickets).
- Description of the confidential/whistleblower reporting channel and where it is documented.
- Evidence that engineers are informed of system changes affecting security (release notes, change tickets, PR review process description).

## How a tiny AI-first startup satisfies it
- Single source of truth: the `policies/` repo, linked from the onboarding checklist and pinned in the team Slack channel. Policy README lists every policy, its owner, and last-review date.
- Announce changes: every merged policy PR triggers a Slack post to `#general`/`#security` (GitHub → Slack integration) plus re-acknowledgment tickets in Linear for material changes. The Slack integration message history is your distribution evidence.
- Responsibilities: communicated via `roles-and-responsibilities.md` (CC1.3) and each person's onboarding ticket, which explicitly lists their security duties.
- Incident/escalation comms: `runbooks/incident-response.md` names the internal notification path (who is told, in what channel, within what time). If no incidents occurred, run one annual tabletop and keep the notes — auditors ask.
- Whistleblower-equivalent: a sentence in the Code of Conduct naming the external advisor/counsel email for concerns about founders. That satisfies "separate communication lines" at this scale.
- Change communication: Linear + PR descriptions + a lightweight release log serve as change communication; state this in the SDLC policy so the auditor can map artifact to claim.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Policies README exists with owner + last-review table | GitHub | `gh api repos/{org}/policies/contents/README.md`; parse table rows |
| Policy-change announcement automation configured | GitHub | `gh api repos/{org}/policies/hooks` — Slack (or equivalent) webhook active on push/PR-merge events |
| Re-acknowledgment tickets created after material policy changes | Linear API + GitHub | For each policy PR labeled `material`, matching `policy-ack` issues exist per person |
| IR plan contains internal communication/escalation section | GitHub | Fetch `runbooks/incident-response.md`, grep for escalation/notification headings |
| Annual tabletop or real-incident comms record exists | Linear API | Issue labeled `tabletop` or `incident` with Done state within last 12 months |
| Onboarding template references policies repo and role responsibilities | Linear API | Fetch onboarding issue template, grep for policies-repo link |
| Whistleblower/alternate channel documented | GitHub | Grep Code of Conduct for alternate-contact section |
| Whether people actually understand their responsibilities | MANUAL | Auditor interviews staff |
| Quality/timeliness of real communications | MANUAL | Auditor reviews Slack samples with management |

## Evidence artifacts
- `policies/README.md` — index with owners and review dates (the communication anchor).
- Slack screenshots/exports of policy-change announcements and security posts, saved to `evidence/communications/<quarter>/`.
- Linear exports: `policy-ack` re-acknowledgment tickets, onboarding tickets, `tabletop`/`incident` tickets with comms trail.
- `runbooks/incident-response.md` — escalation and notification section (version-controlled).
- Code of Conduct alternate-reporting-channel excerpt (part of CC1.1 evidence, cross-referenced).
- Quarterly oversight packet (CC1.2) demonstrating management↔board communication flow.
