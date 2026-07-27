---
description: Prove the ticket's acceptance criteria in a real browser — screenshot evidence per criterion, posted to the ticket (CC8.1 "tested")
---
# /shadow:verify — Prove It Works in the Browser

Drive the running app with Playwright MCP as the user would, walk every
acceptance criterion, and post the verified user story + screenshots to the
ticket. This is the CC8.1 "tested" evidence for UI-facing changes — a green
unit suite says the code runs; this says the feature works.

Run during or after `/shadow:load`, before `/shadow:finish`.

## Phase 1: Resolve the ticket & criteria

Argument is a ticket ID (`[A-Z]{2,6}-[0-9]+` or `#N`); if absent, derive it
from the current branch name (`{TICKET-ID}-{slug}`). Fetch the ticket and its
comments (the spec comment often refines the criteria):
- Linear MCP: `get_issue(id)` + `list_comments(issueId)`.
- GitHub Issues: `gh issue view {N} --json title,body,url --comments`.

Each acceptance criterion becomes one verification case. A criterion that is
not browser-observable (pure backend, config-only) is marked "not
browser-verifiable" and skipped — never fake evidence for it. Zero
browser-observable criteria → report that and stop; nothing to prove here.

## Phase 2: Start or locate the app

1. Detect the run command (`package.json` scripts, `Makefile`, README).
2. If it's already serving, use it; otherwise start the dev server **in the
   background**, logging to a file so browser failures can be correlated with
   server stack traces.
3. Wait until the URL actually responds (poll / `browser_wait_for`) — a live
   process is not a live app.

App won't start → **STOP**. Capture the startup error, report it, do not open
a browser against nothing.

## Phase 3: Plan the walk

For each criterion write a concrete user-story script: entry route, the
interactions (clicks, form fills, key presses), and the **observable success
condition** (text appears, element state, URL change, request returns 2xx).
Add at least one adversarial check per story where sensible — empty input,
invalid value, unauthorized access — so the evidence covers more than the
happy path.

## Phase 4: Execute & collect evidence

Per story, using the Playwright MCP tools:
1. `browser_navigate` → `browser_wait_for` load → `browser_snapshot` to find elements.
2. Interact: `browser_click`, `browser_fill_form`, `browser_type`, `browser_select_option`.
3. Capture at each meaningful step:
   - `browser_take_screenshot` → save to `.shadow-evidence/{TICKET-ID}/{case}-{step}.png`
   - `browser_console_messages` → record errors/warnings
   - `browser_network_requests` → record 4xx/5xx or stuck-pending requests
4. Judge pass/fail against the criterion's observable success condition.
   Record criterion, steps taken, result, screenshot paths, anomalies.

`.shadow-evidence/` is local scratch: ensure it is in the repo's `.gitignore`
(append it if missing). Screenshots are posted to the ticket, **never
committed to the code repo** — `/shadow:finish` must find a clean diff.

## Phase 5: Auto-fix loop (small fixes only, capped)

A case fails → diagnose from the evidence (browser state + server log), and if
the fix is small and **within the posted spec** (wrong selector, missing null
check, off-by-one in existing code), fix it, restart if needed, and re-run only
the failed cases. **Max 3 cycles.** Anything bigger — new behavior, schema
change, scope beyond the spec — goes back through `/shadow:load` (revise the
spec on the ticket first). Never mark a failing case as passed; after the cap,
report remaining failures honestly with their evidence.

## Phase 6: Post the verified story to the ticket

When all browser-verifiable criteria pass:
- **Linear MCP:** upload screenshots one at a time — `prepare_attachment_upload`
  (exact byte size via `wc -c`), immediately `curl -X PUT --data-binary` with
  every returned header verbatim (signed URLs expire in ~60s; never batch),
  then `create_attachment_from_upload`. Finish with `save_comment` embedding
  each shot inline as `![{case}]({assetUrl})`.
- **GitHub Issues:** `gh issue comment {N} --body-file /tmp/verify.md`; attach
  images via the upload API or link them from the comment.

Comment structure:

```markdown
## Verified in browser — {TICKET-ID}

Driven against `{URL}` on {date}.

### Criteria verified
1. {criterion} → {steps} → {observed result}  {screenshot}

### Adversarial checks
- {invalid/empty/unauthorized case} → {handled as expected}

### Evidence
- Console: {clean / listed errors}
- Network: {all 2xx / listed failures}
- Skipped (not browser-verifiable): {list or none}
```

Close the browser (`browser_close`) and stop any server you started.

## STOP conditions
- App won't start after diagnosis → STOP, report the startup error.
- Criteria unmeetable as written (contradict the app's actual behavior/design)
  → STOP, raise it on the ticket; do not reinterpret silently.
- Playwright MCP unavailable → STOP, report; do not fabricate evidence.
- Tracker unreachable → keep local evidence, report, retry the post later.

Report: ticket ID, criteria verified/skipped/failed, auto-fix cycles used, and
the ticket comment URL. Then continue to `/shadow:finish`.
