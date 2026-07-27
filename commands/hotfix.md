---
description: Document a direct push to main — incident ticket, backport PR through normal gates, everything linked
---
# /shadow:hotfix — Pay for the Emergency

A direct push to `main` is allowed only for genuine emergencies, and it costs
paperwork by design (SDLC §9). This command produces that paperwork: an
incident ticket, a backport PR to `staging` through the normal gates, and
links tying them together. The next daily shadow audit checks that **every**
bypass has exactly this trail — make it true before it checks.

Run this immediately after the hotfix lands (or when the bypass detector /
daily audit flags an undocumented direct push).

## Phase 1: Identify the hotfix commits

```bash
git fetch origin main staging
git log origin/staging..origin/main --oneline --no-merges
```

These are the commits on main that never went through staging. For each, note
SHA, author, timestamp, and files touched (`git show --stat {sha}`). If this
list is empty and you were pointed at a bypass, check the archive record that
flagged it (`is_bypass: true` in `compliance-archives`) for the PR/commit.

## Phase 2: Gather incident context

Ask the user (or extract from the situation if unattended and evident):
1. **What broke** — user-visible symptom and affected surface.
2. **Impact** — who/what was affected, duration, severity.
3. **Root cause** — the actual defect, not the symptom.
4. **Why the process was bypassed** — why the fix couldn't wait for the normal
   staging → release path. "It was faster" is not an emergency; record honestly.
5. **The fix** — what the hotfix commits change.

Do not proceed with placeholders. An incident ticket with "TBD" root cause is
worse than a late one — STOP and get the answers.

## Phase 3: Create the incident ticket (priority: urgent)

- Linear MCP: `save_issue(title: "INCIDENT: {symptom}", priority: 1 (urgent), description: ...)`.
- GitHub Issues: `gh issue create --title "INCIDENT: {symptom}" --label incident,urgent --body-file /tmp/incident.md`.

Body structure:

```markdown
## Incident
**What broke:** ...
**Impact:** ...
**Detected:** {when/how}

## Root cause
...

## Hotfix
Commits pushed directly to main: {sha list with links}
**Why the process was bypassed:** ...

## Remediation
- [ ] Backport PR to staging: {filled in Phase 4}
- [ ] Follow-up actions: {rotation, monitoring, test to prevent recurrence}
```

Extract the `IDENTIFIER` from the API response — never fabricate.

## Phase 4: Backport PR through the normal gates

The fix must exist on `staging` too, or the next release's fast-forward will
fail and staging silently lacks the fix.

```bash
git fetch origin staging
git checkout -b "${IDENTIFIER}-backport-hotfix" origin/staging
git cherry-pick {sha...}          # each hotfix commit, in order
# Conflicts: resolve minimally to reproduce the fix on staging; unsure → STOP, ask.
git push -u origin "${IDENTIFIER}-backport-hotfix"
```

Open a normal (non-draft — it's already tested in production) PR **to staging**
with the four required sections; `shadow-ci` gates it like any change:

```bash
gh pr create --base staging --head "${IDENTIFIER}-backport-hotfix" \
  --title "${IDENTIFIER}: backport hotfix — {symptom}" \
  --body "## Summary
Backport of emergency hotfix pushed directly to main ({sha list}). See incident {IDENTIFIER}.

## Tickets
| Ticket | Title | Status |
|--------|-------|--------|
| [${IDENTIFIER}]({URL}) | INCIDENT: {symptom} | In Progress |

## Changes
### ${IDENTIFIER}
- `{each file}` — {what the hotfix changed}

## Test Plan
- [ ] Verified in production by the hotfix itself: {evidence}
- [ ] CI green on this backport"
```

If the hotfix commits touched source files with no tests, add the missing test
in this backport PR — the gate will demand it anyway, and the incident earns a
regression test.

## Phase 5: Link everything

- Comment the backport PR URL and the hotfix SHAs on the incident ticket
  (Linear `save_comment` / `gh issue comment`), and check the remediation box.
- If the bypass was already archived with `is_bypass: true`, comment the
  incident ticket URL on the archived PR (or note it in the incident ticket so
  the audit can join them).
- Report to the user: incident ticket URL, backport PR URL, remaining
  remediation items. The backport now proceeds through
  `/shadow:fix-compliance` / `/shadow:fix-pr` like any PR.

## STOP conditions
- No actual divergence between main and staging and no flagged bypass → nothing to document; report and exit.
- Incident context incomplete → STOP (Phase 2).
- Cherry-pick conflicts you can't resolve confidently → STOP, ask the user.
- Never "fix" the divergence by force-pushing or resetting main. Ever.
