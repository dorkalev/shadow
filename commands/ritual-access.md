---
description: Quarterly access review as a 10-minute dialogue — gather live grants, walk only the diff, file the signed packet (CC6.2/CC6.3)
---
# /shadow:ritual-access — The Quarterly Access Review

Turns the quarterly access review (CC6.2/CC6.3) into a 10-minute interview.
Three phases, strictly ordered: **gather** (zero questions), **interview**
(judgment only), **file** (zero questions). Never ask the user anything a tool
can discover.

**Segregation of duties — absolute:** the agent NEVER answers its own review
questions, NEVER defaults an approval to "yes", and the responding human's name
goes on the record as reviewer. Any question left unanswered is recorded as
**OPEN**, never silently approved.

## Phase 1: GATHER (no questions asked)

**Step 1 — generate this quarter's packet and push it to archives:**
```bash
ARCHIVES_PUSH=1 .shadow/ci/target/release/shadow-ci access-review
```
The packet lands under `evidence/{YYYY}/{QN}/` on the `compliance-archives`
branch. Read the generated packet in full.

**Step 2 — fetch last quarter's packet and compute the diff.** Determine the
previous quarter from today's date, then:
```bash
git fetch origin compliance-archives
git show origin/compliance-archives:evidence/{YYYY}/{QN-1}/access-review-*.md
```
(List the directory first with `git ls-tree` if the filename timestamp is
unknown; if no prior packet exists, this is the baseline quarter — every grant
is "new".)

Compute the grant DIFF per system (GitHub org roles, outside collaborators,
per-repo direct grants, deploy keys, GCP IAM bindings, service-account keys):
- **NEW** — grants in this quarter's packet not in last quarter's
- **REMOVED** — grants present last quarter, gone now (note them; no approval needed)
- **UNCHANGED** — everything else

**Step 3 — device posture.** For each teammate, request (via the user, who
relays) that they run the one-liner and paste the raw output:
```bash
# macOS
fdesetup status && sysadminctl -screenLock status
```
Whatever is pasted gets filed verbatim into the packet's manual-attachments
section. Anyone who hasn't pasted by the end of the interview is marked
**OPEN — device posture not attested**. Do not fabricate or paraphrase output.

## Phase 2: INTERVIEW (judgment only, batched)

Walk **only** the new/changed grants — one batch per system, evidence inline:

> **GCP — 3 new grants this quarter:**
> 1. `alice@…` → `roles/cloudsql.admin` on `proj-x` (appeared 2026-05-14)
> 2. …
>
> Approve all, or flag which? (flagged grants get a revocation ticket)

Then one blanket confirmation for everything unchanged:

> **N unchanged grants across all systems** (unchanged since last quarter's
> signed review). Confirm they all remain appropriate? [confirm / name exceptions]

Finally: **"Who is signing this review as reviewer?"** — collect the human's
name. The agent must not be the reviewer. If the reviewer granted any of the
new access themselves, note it in the packet (small-team reality; the record
must say so).

Unanswered batches → every grant in the batch is recorded **OPEN**.

## Phase 3: FILE (no questions asked)

1. **Fill the sign-off block** in this quarter's packet: per-grant decisions
   (APPROVED / FLAGGED / OPEN), the computed diff summary, pasted device-posture
   outputs (absentees marked OPEN), reviewer name, and today's date.
2. **Commit to `compliance-archives`** — checkout the branch (worktree
   preferred: `git worktree add /tmp/archives compliance-archives`), amend the
   packet file in place under `evidence/{YYYY}/{QN}/`, commit with message
   `access-review {YYYY}-{QN}: signed by {reviewer}`, push. The commit is the
   signature.
3. **Open one revocation ticket per flagged grant** — Linear MCP
   (`save_issue`) when available in the session, otherwise `gh issue create`.
   Ticket body must contain the EXACT revocation command, e.g.:
   ```bash
   gcloud projects remove-iam-policy-binding {project} \
     --member="user:{email}" --role="{role}"
   # or
   gh api -X DELETE /orgs/{org}/members/{user}
   gh api -X DELETE /repos/{owner}/{repo}/collaborators/{user}
   ```
   plus the evidence line from the packet and a link to the archived review.
4. **Report a one-screen summary:** grants reviewed (new/removed/unchanged
   counts), approvals, flags (with ticket IDs), OPEN items (unanswered
   batches, missing device attestations), reviewer, archive commit SHA.

## STOP conditions
- `shadow-ci access-review` fails or every system section is "unavailable" →
  fix credentials first; a review of an empty packet proves nothing.
- No human responds to the interview → file the packet with ALL decisions OPEN
  and say so in the summary. An OPEN review is honest; a self-approved one is
  a control failure.
