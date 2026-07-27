---
id: CC4.2
family: CC4 — Monitoring Activities
category: Security (Common Criteria)
coso: COSO Principle 17
title: Deficiency Communication and Remediation
weight: 2
automatable: partial
nature: document
---

# CC4.2 — Deficiency Communication and Remediation

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 17: The entity evaluates and communicates internal control deficiencies in a timely manner to those parties responsible for taking corrective action, including senior management and the board of directors, as appropriate.

## What it means
Finding problems (CC4.1) is worthless if nothing happens next. CC4.2 requires that when monitoring, scans, self-assessments, pentests, or incidents reveal a control deficiency, it is (1) evaluated for severity, (2) communicated to whoever can fix it and to management, and (3) tracked to remediation on a timeline that matches its severity. The auditor's test is brutally simple: take the failures your own monitor recorded, the pentest findings, and the Dependabot criticals, and check each one reached a named owner and got fixed — or was formally risk-accepted — within your stated SLA.

At a 1–10 person startup, "communicate to senior management" is nearly automatic — the person who sees the alert usually *is* senior management. That doesn't make the criterion free: you still need the loop to be observable. An alert that fired into a Slack channel nobody reads, or a pentest finding that sat unowned for five months, is a classic Type II exception even when the founders "knew about it."

The honest implementation is: every deficiency becomes a Linear ticket with a severity label and due date derived from a written remediation SLA; the ticket's lifecycle (created → assigned → closed, with timestamps) is the communication-and-correction evidence. "Board" communication for a startup with a real board means a security/compliance line in the board update; without a board, founder review suffices and should be documented as such.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Assesses results of evaluations and determines whether findings represent deficiencies in internal control.
- Communicates deficiencies to parties responsible for corrective action and to management/board as appropriate.
- Management tracks whether deficiencies are remediated on a timely basis.

## What the auditor will ask for
- The deficiency-handling process (usually a section of the risk-management or incident policy) including severity levels and remediation SLAs.
- The full list of deficiencies identified during the period — from monitoring failures, scans, pentest, self-assessment, and incidents.
- For a sample of those: the ticket, owner, severity, dates opened/closed, and evidence of the fix.
- Aging analysis: any findings open past SLA, and documented risk acceptance for anything not fixed.
- Evidence of escalation to management (Slack/board-update excerpts, or founder sign-off on the monthly compliance summary).
- Post-incident corrective actions and their completion status (overlaps CC7.4/CC7.5).

## How a tiny AI-first startup satisfies it
- Write remediation SLAs into `policies/vulnerability-and-deficiency-management.md`: e.g., critical 7 days, high 30, medium 90, low best-effort; deficiencies past SLA require written founder risk-acceptance in the ticket.
- Single funnel: the shadow monitor, Dependabot, CodeQL, pentest findings, and self-assessment gaps all become Linear tickets with labels `compliance-failure` or `vuln` plus a severity label and due date. One queue, machine-checkable.
- The shadow tool auto-files tickets for its own check failures (it already knows severity from the criterion's weight) and auto-closes/links when the re-run passes — creating a perfect open→fix→verify chain.
- Monthly, the tool generates a one-page compliance summary (open deficiencies, aging, closed this month); a founder approves the PR that archives it — that approval is your "communicated to senior management" evidence. If a board exists, paste the summary into the board update.
- Never delete or silently ignore a finding. Won't-fix is fine; it just needs a sentence of rationale and a founder's name in the ticket.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Deficiency/vuln management policy with SLAs exists | GitHub | File existence + grep for SLA table in policies repo |
| Every archived monitor failure has a corresponding Linear ticket | GitHub + Linear | Cross-reference `compliance-archives` failure entries with issues labeled `compliance-failure` |
| Open Dependabot criticals/highs within SLA age | GitHub | `gh api repos/{org}/{repo}/dependabot/alerts --jq` filter by severity + `created_at` age |
| Open code-scanning alerts within SLA age | GitHub | `gh api repos/{org}/{repo}/code-scanning/alerts?state=open` age check |
| Linear deficiency tickets: closed within SLA or carry risk-acceptance comment | Linear | API query on labels + severity; compute open duration; grep comments for acceptance |
| No unassigned open deficiency tickets older than 7 days | Linear | API query — assignee null + age |
| Monthly compliance summary exists for each month in period | GitHub | File existence `evidence/monthly-summary/YYYY-MM.md` |
| Summary PRs approved by a founder | GitHub | `gh api` PR reviews on summary PRs — approving reviewer in founder list |
| Pentest findings mapped to tickets and resolved | Linear + files | Partially automatable if findings imported; otherwise MANUAL |
| Adequacy of severity ratings and acceptance rationale | — | MANUAL |

## Evidence artifacts
- `policies/vulnerability-and-deficiency-management.md` — severities, SLAs, escalation path.
- Linear export (JSON) of all `compliance-failure`/`vuln` tickets in the period with timestamps, labels, assignees, comments — archived to `compliance-archives`.
- `evidence/monthly-summary/YYYY-MM.md` — monthly deficiency/aging summaries with approving PR links.
- Dependabot and code-scanning alert exports (open + closed, with resolution dates) in `evidence/github/`.
- Pentest remediation tracker (epic export) in `evidence/pentest/`.
- Board-update excerpts or founder sign-off records showing management visibility, in `evidence/governance/`.
