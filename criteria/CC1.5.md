---
id: CC1.5
family: CC1 — Control Environment
category: Security (Common Criteria)
coso: COSO Principle 5
title: Accountability for Internal Control
weight: 2
automatable: partial
nature: document
---

# CC1.5 — Accountability for Internal Control

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 5: The entity holds individuals accountable for their internal control responsibilities in the pursuit of objectives.

## What it means
CC1.3 assigns responsibilities; CC1.5 asks whether anything happens when people do or don't meet them. The auditor wants to see that security responsibilities are part of how performance is assessed, that there are consequences (or at least documented follow-up) for ignoring them, and that incentives don't push people to cut security corners — e.g., shipping pressure that makes engineers bypass code review.

In a tiny startup, accountability is mostly structural: branch protection means you *cannot* merge without review; the shadow-tool monitors mean a lapsed control (stale access review, failed check) becomes a visible, assigned ticket with an owner and a due date. That machinery — control failure → ticket → named owner → closure — is the accountability evidence. Complement it with the human layer: security responsibilities named in each role description, mentioned in the annual review note, and a sanctions clause in the Code of Conduct ("violations may result in disciplinary action up to termination").

Auditors will also probe "excessive pressures": be ready to explain (and ideally write down) that deadlines don't override review requirements, and that emergency changes have a defined hotfix path with retroactive review rather than silent bypass.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Enforces accountability through structures, authorities, and responsibilities** — mechanisms hold individuals accountable for internal control performance, cascading from the board and management down.
- **Establishes performance measures, incentives, and rewards** — appropriate to responsibilities at all levels, reflecting both short- and long-term objectives.
- **Evaluates performance measures, incentives, and rewards for ongoing relevance** — alignment with internal control responsibilities is maintained.
- **Considers excessive pressures** — management evaluates and adjusts pressures (targets, deadlines) that could motivate control circumvention.
- **Evaluates performance and rewards or disciplines individuals** — internal control performance is part of evaluation, with rewards or discipline as appropriate.

## What the auditor will ask for
- Sanctions/disciplinary language in the Code of Conduct or handbook.
- Role descriptions or policy text tying security responsibilities to individuals (overlaps CC1.3).
- Evidence that control failures are tracked to a named owner and remediated (tickets, monitor alerts with assignees).
- Performance-review evidence showing internal-control responsibilities are considered (attestation or redacted sample).
- Records of any disciplinary actions related to policy violations during the period, or attestation that none occurred.
- The exception path for emergency changes (hotfix policy) and evidence that exceptions were documented and reviewed, not silent.

## How a tiny AI-first startup satisfies it
- Sanctions clause in `policies/code-of-conduct.md`: policy violations are addressed by founders and may result in discipline up to termination or contract cancellation. One paragraph.
- Structural enforcement: GitHub branch protection (required reviews, status checks, no force-push) makes review non-optional; Workspace and cloud policies enforce MFA. Controls people can't skip are the strongest accountability mechanism a small team has.
- Failure-to-ticket pipeline: every failed shadow check (stale policy, unreviewed access, missing training) auto-creates a Linear issue labeled `control-failure` with an assignee and SLA. Closure history over the audit period demonstrates accountability in operation.
- Hotfix path: a documented exception process — direct-to-main pushes are allowed only for emergencies, must be recorded (e.g., a `hotfix` Linear ticket with reason) and get retroactive PR review/backport. The record of exceptions, each reviewed, is exactly the "no silent circumvention" evidence.
- Annual review notes (CC1.4) include one line on security-responsibility performance per person; founders self-attest for themselves as part of the quarterly oversight review (CC1.2).
- If a violation happens, document the response in a (private) ticket and provide a summarized attestation to the auditor.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Sanctions clause present in Code of Conduct | GitHub | Fetch `code-of-conduct.md`, grep for disciplinary/sanctions section |
| Branch protection enforced on default branch of production repos | GitHub | `gh api repos/{org}/{repo}/branches/main/protection` — required reviews ≥1, enforce_admins true |
| Direct pushes to main are absent or matched to hotfix tickets | GitHub + Linear | `gh api repos/{org}/{repo}/commits` — commits without associated PR cross-referenced to Linear `hotfix` issues |
| Hotfixes received retroactive review | GitHub + Linear | Each `hotfix` ticket links a backport/review PR in merged state |
| Control-failure tickets have assignee and were closed within SLA | Linear API | Query `control-failure` issues; verify assignee set and Done within N days |
| No long-lived unassigned control failures | Linear API | Open `control-failure` issues older than SLA → flag |
| Annual performance-review attestation file exists for the year | GitHub | File existence `evidence/people/annual-review-<year>.md` on `compliance-archives` |
| Incentive design and pressure evaluation | MANUAL | Auditor inquiry with founders |
| Actual disciplinary handling of violations | MANUAL | Review of records/attestation |

## Evidence artifacts
- `policies/code-of-conduct.md` sanctions section (version-controlled).
- Linear export of `control-failure` tickets with assignees, timestamps, and closure — the operating-effectiveness core for this criterion, archived quarterly to `compliance-archives`.
- Linear export of `hotfix` tickets plus linked backport PRs (exception log with retroactive review).
- Branch-protection settings JSON export in `evidence/github/branch-protection-<date>.json`.
- `evidence/people/annual-review-<year>.md` — attestation that reviews occurred and covered control responsibilities.
- Attestation letter (or ticket summary) regarding disciplinary actions during the period, in the evidence store.
