---
id: CC7.3
family: CC7 — System Operations
category: Security (Common Criteria)
coso: null
title: Security Event Evaluation and Incident Determination
weight: 3
automatable: partial
nature: technical
---

# CC7.3 — Security Event Evaluation and Incident Determination

## Criterion (AICPA TSP Section 100, verbatim)
> The entity evaluates security events to determine whether they could or have resulted in a failure of the entity to meet its objectives (security incidents) and, if so, takes actions to prevent or address such failures.

## What it means

CC7.2 hands you a stream of anomalies; some get classified as *security events*. CC7.3 is the judgment step: for each security event, decide whether it did — or could — cause you to fail your objectives (confidentiality breach, availability loss, data integrity damage, contractual/legal exposure). If yes, it's a *security incident* and you must act. The criterion is deliberately about the evaluation itself: who does it, against what severity rubric, and how the decision is recorded.

At a tiny startup the evaluator is a named founder or engineer, and the rubric is a half-page severity matrix in your incident response plan (SEV1: confirmed data exposure or full outage; SEV2: likely exposure or partial outage; SEV3: contained event, no impact). The critical discipline is writing the decision down *even when the answer is "not an incident."* A Linear ticket that says "leaked key was a scoped read-only token for a sandbox project, revoked within 20 minutes, no data access in audit logs — classified: security event, not incident" is exactly the artifact an auditor wants.

Auditors test this criterion by sampling: they take your security-event list and check each was evaluated, timestamped, severity-rated, and — where it became an incident — that response actions followed (bridging into CC7.4). An empty event list across a whole period invites the question of whether CC7.2 works at all, so classify honestly; low-severity events with clean write-ups strengthen your report.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Assigns roles and responsibilities for the evaluation of security events, including responsibility for determining whether an event is a security incident.
- Communicates and reviews detected security events with designated personnel.
- Develops and implements procedures to analyze security events, including determining root cause and impact on objectives.
- Uses defined criteria (severity/impact thresholds) to determine whether an event constitutes an incident requiring response.
- Initiates prevention or remediation actions when the evaluation indicates objectives could be or were affected.

## What the auditor will ask for
- Incident response plan section defining security-event evaluation: roles, severity matrix, decision criteria, and required documentation.
- The complete register of security events for the audit period (Linear query export).
- For a sample of events: the evaluation record — who assessed, when, severity assigned, incident yes/no rationale, and root-cause notes.
- For events classified as incidents: linkage to the incident response records (CC7.4 evidence).
- Evidence that evaluation happened timely (event detection timestamp vs evaluation timestamp).
- Evidence of preventive follow-ups for near-miss events (e.g., "not an incident, but we added push protection" — a Linear ticket and merged PR).

## How a tiny AI-first startup satisfies it
- **Incident response plan** (`policies/incident-response.md`) contains a severity matrix (SEV1–SEV3), a named incident commander default (founder/CTO) with deputy, and a rule: every `security-event` ticket must reach a disposition (`incident` / `no-incident` + rationale) within 24 hours of detection.
- **Linear as the system of record**: security events are tickets with label `security-event`; a required template captures detection source, timeline, evaluator, severity, incident determination, and rationale. Events promoted to incidents get label `incident` and a linked incident ticket.
- **Evaluation inputs are queryable**: cloud audit logs (`gcloud logging read`) to check whether a leaked credential was used; GitHub audit log for repo events; Sentry for blast-radius of an error. The evaluation write-up cites the queries run — this makes the analysis reproducible for the auditor.
- **AI-assisted, human-decided**: the compliance shadow can pre-draft an evaluation (pull related logs, propose severity), but the determination is committed by a named human on the ticket — auditors need an accountable person, not a bot.
- **Preventive actions** land as normal changes through the CC8.1 SDLC, so each "action taken to prevent failure" has a ticket → PR → review → archive chain automatically.
- Quarterly: review the event register for patterns (recurring source = weak control) and record the review as a Linear ticket.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Every `security-event` ticket has a disposition within 24h | Linear | Linear API: label query, template-field presence, timestamps |
| Disposition includes evaluator and rationale | Linear | field/comment presence check on sampled tickets |
| Events promoted to `incident` have a linked incident ticket | Linear | relation check between `security-event` and `incident` tickets |
| Secret-scanning alerts each map to a security-event ticket | GitHub+Linear | `gh api .../secret-scanning/alerts` cross-referenced with Linear tickets by alert URL |
| Push-protection bypasses generated events | GitHub | `gh api /orgs/{org}/audit-log` action `secret_scanning_push_protection.bypass` vs Linear |
| Bypass-merge detections generated events | archives | shadow's CC8.1 bypass reports cross-referenced to Linear `security-event` tickets |
| Preventive-action tickets closed with merged PRs | Linear+GitHub | ticket→PR link resolution, PR merged + archived |
| Quarterly event-register review completed | Linear | recurring ticket completion history |
| Severity matrix exists in IR plan | repo | file-existence + section grep in `policies/incident-response.md` |
| Quality of evaluation rationale | Linear | MANUAL — auditor/human judgment on sampled write-ups |

## Evidence artifacts
- Linear: `security-event` ticket register (exportable query), each with evaluation template filled.
- `policies/incident-response.md` — severity matrix and evaluation procedure with review date.
- `evidence/security-events/YYYY-QN-register.csv` — quarterly export snapshot committed by the shadow.
- Log-query excerpts attached to event tickets (audit-log reads used in evaluations).
- `compliance-archives` branch — archive records of preventive-action PRs referenced from event tickets.
