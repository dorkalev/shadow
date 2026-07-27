---
description: Assemble the PBC binder when the real audit starts — verify every expected artifact exists, pull the sampling populations, interview only for the missing, hand the auditor one index file
---
# /shadow:audit-binder — Assemble the PBC Binder

Builds the binder you hand the auditor on day one: every in-scope criterion
mapped to its evidence artifacts, each artifact **verified to exist** with
location and date, plus the standard sampling populations pre-pulled. Three
phases, strictly ordered: **gather** (zero questions), **interview** (only
for MISSING artifacts), **file** (zero questions).

**Rule:** an artifact is FOUND only if the file/commit/ticket is actually
readable at a concrete path. Never mark FOUND from memory, from a criteria
file's claim, or because "it should be there". Unverifiable = MISSING until
the interview resolves it.

## Phase 1: GATHER (no questions asked)

**Step 1 — determine the audit window and scope.** Window: from the
engagement dates if known, else the trailing 12 months (state the assumption
in the binder header). Scope: the shadow's in-scope criteria list.

**Step 2 — expected artifacts per criterion.** For every in-scope criterion
read `criteria/{ID}.md` → its **"Evidence artifacts"** section (cross-check
"What the auditor will ask for" for anything the artifacts list omits). Build
the expected-artifact table: criterion → artifact → where it should live.

**Step 3 — verify each artifact actually exists.** Check the real locations:
```bash
git fetch origin compliance-archives
git ls-tree -r --name-only origin/compliance-archives -- evidence/ releases/
```
plus the policies repo files (policies pack, registers, system description —
read frontmatter dates), `evidence/{YYYY}/{QN}/` packets (access reviews,
mgmt packets per quarter of the window — a Type II needs one per quarter, so
a missing quarter is a MISSING artifact, not a shrug), attestations,
tabletop and restore-test records, and tracker tickets where the artifact is
a ticket. Record per artifact: **location** (exact path/URL/commit) and
**date** (commit date or frontmatter date). Status: FOUND / MISSING.

**Step 4 — pull the standard sampling populations** (completeness is what the
auditor tests first):
- **All merged PRs in the window** = the archives listing (`git ls-tree` of
  the per-PR records; the `compliance-archives` branch IS the population).
  Note any records flagged `is_bypass: true` — list them, never hide them.
- **All releases:** `releases/release-*.json` in archives.
- **All incidents:** tickets labeled `incident` in the window (Linear MCP or
  `gh issue list`), plus hotfix records in archives.
- **All joiners/leavers:** diff the roster across the window's access-review
  packets.

Save each population as a list with counts.

## Phase 2: INTERVIEW (missing artifacts only)

One batched walk of the MISSING list, expected location shown:

> **CC1.4 — background-check evidence: not found** (expected in
> `evidence/{YYYY}/` or an HR ticket).
> (a) exists elsewhere — where exactly? (I will verify the path before
> marking FOUND)
> (b) doesn't exist — I open a remediation ticket.

Every "exists elsewhere" answer gets verified (Phase 1 rules) before the
status changes; an unverifiable location stays MISSING with the claim noted.
Unanswered items stay MISSING. Also collect: who is the audit point of
contact (name goes in the binder header).

The agent never downgrades an expectation ("the auditor probably won't ask")
— the criteria files define expected; only the auditor narrows it later.

## Phase 3: FILE (no questions asked)

1. **`binder-index.md` committed to archives** under `evidence/{YYYY}/` on
   `compliance-archives`: header (window, scope, point of contact, assembly
   date), then the table — **criterion → evidence artifact → location → date
   → status (FOUND / MISSING / OPEN-claimed-unverified)** — followed by the
   four populations with counts and paths, bypass merges listed with their
   incident-ticket links. Commit message:
   `audit-binder {YYYY}: {found}/{expected} artifacts, {readiness}%`.
   This is the file you hand the auditor on day one.
2. **One remediation ticket per genuinely-missing artifact** — Linear MCP
   (`save_issue`) when available, otherwise `gh issue create`: criterion,
   what artifact is expected, why it's absent (never existed vs lapsed), and
   whether it can still be generated inside the window (a missed quarterly
   review cannot be backfilled — the ticket must say so honestly).
3. **One-screen summary:** **audit-readiness % = artifacts found / expected**,
   per-category breakdown (CC families, A1, C1…), MISSING list with ticket
   IDs, population counts (PRs/releases/incidents/joiners-leavers), bypass
   count, binder path + commit SHA.

## STOP conditions
- `compliance-archives` branch missing or empty → there is no evidence spine;
  run the shadow setup and rituals first. A binder of nothing helps no one.
- Scope list unavailable → ask only "which criteria are in scope?" (or offer
  the standard Security + Availability + Confidentiality 38) — the sole
  scope question permitted.
