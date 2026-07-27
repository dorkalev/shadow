---
description: Pick or create a ticket, branch off staging, open a draft PR, ticket to In Progress
---
# /shadow:start — Open Work

No ticket, no branch. This command turns intent into the audit artifacts the
rest of the SDLC hangs off: a ticket, a `{TICKET-ID}-{slug}` branch off
`staging`, and a draft PR to `staging` from minute one.

## Phase 1: Select or create the ticket

**If the argument looks like a ticket ID** (`[A-Z]{2,6}-[0-9]+` or `#N` / bare number):
- Linear MCP available: fetch with `get_issue`. Extract `IDENTIFIER`, `TITLE`, `URL` from the response.
- Otherwise GitHub Issues: `gh issue view {N} --json number,title,url,state`. `IDENTIFIER` is `#{N}`.
- STOP if the ticket does not exist — never invent one.

**If the argument is a description (text, not an ID):**
- Briefly scan the codebase (1–2 searches) to sharpen it, then create the ticket with title, intent, and acceptance criteria:
  - Linear MCP: `save_issue(title, description, teamId)`.
  - GitHub Issues: `gh issue create --title "..." --body "..."` then `gh issue view` to get the number.
- **CRITICAL**: take `IDENTIFIER`/`TITLE`/`URL` from the API response. Never fabricate or guess an ID.

**If no argument:** list open tickets assigned to the user (Linear MCP `list_issues`, or `gh issue list --assignee @me --state open`), show a `| ID | Title | Priority | State |` table, and ask which to take (or take a description for a new one).

## Phase 2: Check for existing branch/PR

```bash
git fetch origin
BRANCH_NAME="${IDENTIFIER}-$(echo "${TITLE}" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9' '-' | sed 's/^-*//; s/-*$//' | cut -c1-50)"
# For GitHub Issues, strip the '#': BRANCH_NAME="${N}-${slug}" won't match the ticket regex in titles — keep '#N' in the PR title/body, use the bare number in the branch.
git branch -a | grep -i "${IDENTIFIER#\#}" || true
gh pr list --state open --search "${IDENTIFIER}" --json number,headRefName,url
```

- Branch already exists → check it out (`git checkout ${BRANCH_NAME}` or track the remote), skip to Phase 4.
- Open PR already exists for it → report the URL, skip creation. Do NOT open a duplicate.

## Phase 3: Branch + draft PR

```bash
git fetch origin staging
git checkout -b "${BRANCH_NAME}" origin/staging
git push -u origin "${BRANCH_NAME}"
```

Open the draft PR **to staging** (never to main) with all four required sections
— the `shadow-ci` compliance checker parses this structure:

```bash
gh pr create --draft --base staging --head "${BRANCH_NAME}" \
  --title "${IDENTIFIER}: ${TITLE}" \
  --body "## Summary
${one_or_two_sentences_of_intent}

## Tickets
| Ticket | Title | Status |
|--------|-------|--------|
| [${IDENTIFIER}](${URL}) | ${TITLE} | In Progress |

## Changes
_(populated as work lands — every changed file will be listed here under its ticket)_

## Test Plan
_(populated before the PR is marked ready)_"
```

## Phase 4: Ticket to In Progress

- Linear MCP: `save_issue(id, state: "In Progress")`.
- GitHub Issues: `gh issue edit {N} --add-label "in-progress"` (and assign: `--add-assignee @me`).

Report: ticket ID, title, branch name, PR URL. Then continue **in this
session** by running `/shadow:load ${IDENTIFIER}` — spec first, then code.

## STOP conditions
- Ticket ID given but not found in the tracker → STOP, ask the user.
- Working tree dirty on a different branch → STOP, ask before switching (never stash/discard on the user's behalf).
- `gh pr create` fails (permissions, no remote) → STOP and report; do not commit work without the draft PR.
