---
description: Annual risk refresh as a dialogue — derive candidate risks from the year's real data, re-rate every entry with a human, file the updated register (CC3.1–CC3.4, CC9.1)
---
# /shadow:ritual-risks — The Annual Risk Refresh

Turns the annual risk assessment (CC3.1–CC3.4, CC9.1) into a structured
interview. Three phases, strictly ordered: **gather** (zero questions),
**interview** (judgment only), **file** (zero questions). Never ask the user
anything a tool can discover.

**Segregation of duties — absolute:** the agent NEVER rates a risk itself,
NEVER defaults an existing rating or treatment to "still fine", and the
responding human's name goes on the record as the assessor. Any risk left
undiscussed is recorded as **OPEN — not reassessed**, never silently carried
forward.

## Phase 1: GATHER (no questions asked)

**Step 1 — read the current register.** Fetch `risk-register.md` from the
policies repo (frontmatter: owner/version/approved_by/approved_at/review_by).
Note its last-approved date — that date bounds "since last refresh" for every
query below. No register exists → this run creates the baseline; every risk
below is a candidate.

**Step 2 — derive CANDIDATE risks from the year's actual data.** Collect all
of it before asking anything:
- **Incidents:** tickets labeled `incident` since last refresh — Linear MCP
  `list_issues` (label filter) when available, else
  `gh issue list --label incident --state all --search "created:>{date}"`.
- **Bypass merges:** grep the archives —
  `git fetch origin compliance-archives && git grep -l '"is_bypass": true' origin/compliance-archives -- evidence/` (each one is realized change-control risk).
- **Criteria that spent time failing:** gauge history in `shadow.db` and open
  or closed shadow-regression tickets — sustained red on a criterion is an
  observed weakness, not a hypothetical.
- **New vendors:** diff `vendor-register.md` entries dated after the last
  refresh (each new vendor with data access is a candidate third-party risk).
- **Org changes:** joiners/leavers from the access register / access-review
  packets in archives (key-person and offboarding risk).

Draft one candidate per finding, with the evidence line attached (ticket ID,
archive path, gauge dates). Deduplicate against existing register entries.

**Step 3 — pre-compute the walk list:** every existing risk (current
likelihood/impact/treatment/owner) + every candidate (evidence inline). Do not
pre-fill proposed ratings — ratings are the human's job.

## Phase 2: INTERVIEW (judgment only, batched)

Walk the register in three passes, evidence inline with every question:

**Pass 1 — existing risks**, batched by category:
> **R-04 — Cloud region outage** (likelihood 3, impact 4, treatment ACCEPT,
> owner: dor). No related incidents this year. Likelihood still 3? Impact
> still 4? Treatment still ACCEPT — say why in one line.

Where gathered data contradicts a rating, show it: "R-07 credential compromise
is rated likelihood 2, but there was 1 leaked-key incident (ENG-231) this
year — revise?"

**Pass 2 — candidates**, one at a time with evidence:
> **CANDIDATE — dependency on {new LLM vendor}** (added to vendor register
> 2026-03). Add to the register? If yes: likelihood, impact, treatment
> (ACCEPT / MITIGATE / TRANSFER / AVOID), owner.

**Pass 3 — fraud consideration (CC3.3, explicit and on the record):** one
question set covering fraud vectors — insider misuse of production access,
payment/billing manipulation, management override of the SDLC gates,
agent-credential misuse. Ask whether each is covered by an existing entry or
needs one; record the answers verbatim as the CC3.3 fraud-consideration note.

Finally: **"Who is signing this assessment?"** — collect the human's name.
Unanswered items → **OPEN**.

## Phase 3: FILE (no questions asked)

1. **Updated `risk-register.md` as a PR to the policies repo** through the
   normal gates (ticket, branch, the four PR-body sections): revised ratings
   with one-line rationale each, new entries, retired entries struck with
   reason, the CC3.3 fraud note, frontmatter bumped (version, approved_by =
   the named human, approved_at = today, review_by = +12 months). OPEN items
   stay in the register marked OPEN.
2. **Refresh-evidence md to archives** under `evidence/{YYYY}/` on
   `compliance-archives`: the walk record — per-risk decisions and who made
   them, candidates raised (with source evidence) and their disposition, the
   fraud Q&A, assessor name, date, link to the register PR. Commit message:
   `risk-refresh {YYYY}: assessed by {name}`.
3. **One treatment ticket per MITIGATE decision** — Linear MCP (`save_issue`)
   when available, otherwise `gh issue create`. Body: the risk entry, the
   agreed mitigation, owner, and a link to the archived refresh record.
4. **One-screen summary:** risks reassessed / added / retired / OPEN, rating
   changes, MITIGATE tickets opened (IDs), fraud vectors covered, assessor,
   register PR URL, archive commit SHA.

## STOP conditions
- Policies repo unreachable or `risk-register.md` location unknown → ask the
  user for the repo path only, nothing else.
- No human responds → file the refresh record with ALL entries OPEN, open no
  PR (an unassessed register must not get a fresh approval date), and say so.
