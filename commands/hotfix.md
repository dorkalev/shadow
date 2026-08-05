---
description: Document an emergency bypass of protected main and file the remediation PR
---
# /shadow:hotfix — Pay for the Emergency

Normal changes reach production only through a gated PR to protected `main`.
If the founder uses an authorized emergency bypass, the post-merge archive
must flag it and this command creates the exception trail. The one-person
company does not pretend a second approver exists.

## Phase 1: Identify the bypass

Inspect the flagged archive record and the affected `main` commit. Capture the
SHA, authenticated actor, time, changed files, failed or missing checks, and
the exact bypass mechanism. If no bypass record or direct change exists, stop
rather than fabricate an incident.

## Phase 2: Record the incident

Ask the founder for what broke, impact, root cause, why waiting for the normal
PR path would have increased harm, and what was changed. Create an urgent
incident ticket with those answers and links to the commit and archive record.
Placeholders such as `TBD` remain explicit open exceptions.

## Phase 3: Open a remediation PR to main

Branch from the current `main` and add the missing regression test, monitoring,
documentation, or other corrective control. The emergency commit itself is
already in history and must not be replayed or rewritten.

```bash
git fetch origin main
git checkout -b "${IDENTIFIER}-hotfix-remediation" origin/main
# add the smallest corrective changes and regression test
git push -u origin "${IDENTIFIER}-hotfix-remediation"
gh pr create --base main --head "${IDENTIFIER}-hotfix-remediation" \
  --title "${IDENTIFIER}: hotfix remediation" --body-file /tmp/pr_body.md
```

The remediation PR uses the standard four-section template, links the incident
and bypass record, and passes every normal required check. It does not erase
the original exception.

## Phase 4: Close the loop

Link the PR, completed checks, and final disposition from the incident. When
the PR merges, confirm that a new non-bypass archive record exists. Record any
remaining follow-up owner and due date.

## STOP conditions

- Incident facts are incomplete.
- The proposed action rewrites `main` or archive history.
- The remediation PR targets anything except `main`.
- A required gate is being disabled instead of satisfied.
