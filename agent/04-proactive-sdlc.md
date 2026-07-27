# Agent Runbook 04 — Operate the Proactive SDLC (daily development)

This runbook is for the agent (or human+agent pair) doing *ordinary development work* inside the SDLC that Runbook 02 installed. It is the spec-first workflow adapted to the shadow: compliance is not a phase, it's how every change moves. If Runbooks 01–03 are the auditor, this one is the disciplined engineer the auditor never catches out.

## The command loop (`start → load → finish → fix-compliance → fix-pr → release`)

### start — open work
1. Take a ticket (or create one from the user's description — title, intent, acceptance criteria). No ticket, no branch.
2. `git checkout -b {TICKET-ID}-{slug} origin/staging`; open a **draft PR to staging** immediately with the four-section template.
3. Move ticket to In Progress.

### load — spec, then build
1. Read the ticket; research the codebase.
2. Post the implementation plan as a ticket comment **before** writing substantive code (interactive: get user approval; unattended: post and proceed).
3. Implement. New source files get tests. Stay inside the spec — if scope grows, update the ticket first, then the code.

### finish — the pre-push gate
1. Cleanup (temp files, debug artifacts, stray secrets — scan the diff).
2. **Spec alignment (blocking)**: compare the ticket + spec comment against `git diff origin/staging..HEAD`. Misaligned ⇒ fix the spec or drop the code; do not push misalignment.
3. Commit, merge `origin/staging` (never rebase pushed work), push, mark PR ready.
4. PR body final form: Summary / Tickets table / **Changes with every file listed under its ticket** / Test Plan. Ticket → In Review.

### fix-compliance — answer the shadow
When the compliance check fails, read its PR comment + run log first, then fix mechanically: unspecced file ⇒ add to Changes; invalid ticket ⇒ verify in tracker or remove; ghost ticket ⇒ remove or implement; thin description ⇒ rewrite; missing reviewer ⇒ trigger the review bot (push a commit, or its re-review command if it has one). Push; wait for re-run. Never argue with the gate by weakening the gate.

### fix-pr — answer the reviewer
Fetch review-bot findings; fix Critical/Major (or reply with a concrete false-positive justification and resolve); commit; wait for re-review. Loop until no Critical/Major remain. The review-gate check enforces this anyway — doing it promptly is just cheaper.

### release — ship
Only from green staging. Build the summary (commits, PRs, tickets since last release) → human types the confirmation word → archive record to `compliance-archives` → release ticket → `--ff-only` into main → comment the release on every included ticket. Fast-forward fails ⇒ stop and investigate, never force.

### hotfix — the documented emergency
Pushed to main directly? Immediately: incident ticket (impact, root cause, why bypassed), backport PR to staging through normal gates, link everything. The next Runbook 03 run checks that every bypass has this paper trail; make it true before it has to check.

## Standing rules for AI agents in this SDLC

1. Same doors as humans: no pushing with checks red, no self-approval (approving identity ≠ authoring identity), no editing rulesets/workflows to make a gate pass — a gate change is itself a ticketed, reviewed change.
2. Secrets never in code, diffs, tickets, or PR bodies. Seeing one in history ⇒ treat as incident (rotate, then hotfix procedure).
3. Every destructive or production-touching action needs a ticket trail *first* (that's CC8.1 authorization, and it's also just how you'd want an agent to behave).
4. When a gate and a deadline conflict, the gate wins; the human can consciously bypass — and the bypass detector will file the paperwork demand. The agent never bypasses on its own initiative.
5. Weekly hygiene (agent-runnable): triage new Dependabot/secret/CodeQL alerts into tickets (CC7.1); check PR queue for stale drafts; confirm the daily Runbook 03 cron actually ran (a dead monitor is itself a CC4.1 regression).

## Why this shape

Every audit question about change management has the same answer under this loop: *"here is the population (compliance-archives), pick any change, and you will find its ticket, spec, review, checks, approval, release record — generated at the time, by the process, not reconstructed for you."* That sentence is the product.
