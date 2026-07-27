---
description: Offboarding execution with proof — before-picture, confirmed revocations, after-picture committed to archives (CC6.2)
---
# /shadow:offboard — Revoke Access, Prove It

Executes an offboarding (CC6.2) and produces the revocation proof in one pass:
a before-picture of every grant, human-confirmed revocations, and an
after-picture that must come back empty. **Args:** the leaver's login and/or
email (`/shadow:offboard alice` or `alice@example.com` — GitHub uses the
login, GCP uses the email; resolve both in Phase 1).

Three phases: **gather** (zero questions), **interview** (judgment only),
**file** (zero questions). Revocation is destructive — nothing is executed
without an explicit per-system "y" from the human. The agent never confirms
its own question; no answer means the system stays untouched and is recorded
**OPEN**.

## Phase 1: GATHER (no questions asked)

**Step 1 — the before-picture.** Run the per-person grant report:
```bash
USER_FILTER={user} .shadow/ci/target/release/shadow-ci access-review
```
(Local write is fine here — the archive commit happens in Phase 3 with both
pictures together.) Read it and enumerate every live grant: GitHub org
membership/role, per-repo collaborator grants, outside-collaborator entries,
GCP IAM bindings per project, user-managed service-account keys they own.
Note the manual systems the report can only list (Google Workspace, tracker
seat, anything without an API path).

**Step 2 — resolve identity.** Map login ↔ email where the report is
ambiguous (`gh api /users/{login}`, GCP bindings show the email). Save the
before-picture file path.

## Phase 2: INTERVIEW (judgment only)

**First, confirm identity and last day:**

> Offboarding **{name}** — GitHub `{login}`, GCP `{email}`. N live grants
> found. Confirm this is the right person, and their last day?

Then walk **per system**, showing the grants and the EXACT commands, and ask
**"execute now? [y/n]"** for each system:

> **GitHub org** — member (role: {role})
> ```bash
> gh api -X DELETE /orgs/{org}/members/{login}
> ```
> **GitHub repos** — direct collaborator on: {repos}
> ```bash
> gh api -X DELETE /repos/{owner}/{repo}/collaborators/{login}   # per repo
> ```
> **GCP** — {bindings}
> ```bash
> gcloud projects remove-iam-policy-binding {project} \
>   --member="user:{email}" --role="{role}"                      # per binding
> ```
> **Manual (no API here):** Google Workspace suspend + 2SV/session revoke,
> tracker seat removal — these need a human in the admin console; I will
> record them OPEN with a checklist unless you tell me they're done.

Default is **never yes**. "n" or silence → system recorded OPEN, commands left
in the ticket for later. Execute each confirmed system's commands immediately
and capture the output.

## Phase 3: FILE (no questions asked)

1. **The after-picture.** Re-run:
   ```bash
   USER_FILTER={user} .shadow/ci/target/release/shadow-ci access-review
   ```
   For every automated system that was confirmed and executed, the report
   **must be empty**. Any residue → surface it verbatim in the summary as a
   FAILED revocation (do not retry destructive commands unprompted; show the
   error and the leftover grant).
2. **Commit before + after to `compliance-archives`** under
   `evidence/{YYYY}/{QN}/offboarding-{user}-{date}/` (worktree:
   `git worktree add /tmp/archives compliance-archives`) — both reports plus a
   short index noting who confirmed each revocation and when. Message:
   `offboarding {user}: revocation proof (before+after)`. This pair IS the
   auditor's revocation evidence.
3. **Ticket.** Find the existing offboarding ticket (Linear MCP search or
   `gh issue list --search "{user}"`); if none, create one. Attach/link both
   pictures and the archive commit. If every system is revoked (no OPEN
   items), close it; otherwise leave it open with a checklist of OPEN manual
   systems, each with its console steps or exact command.
4. **Report a one-screen summary:** systems revoked (with confirmer's name),
   systems OPEN (manual or unconfirmed), after-picture verdict per system,
   ticket ID and state, archive commit SHA.

## STOP conditions
- The before-picture shows zero grants → report it, still file the empty
  before/after pair (proof of absence is still proof), skip the interview.
- The human does not confirm identity → stop entirely; revoke nothing.
- A revocation command errors → stop that system, record OPEN with the error
  verbatim, continue with the rest.
