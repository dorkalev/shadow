---
description: Draft SOC 2 Section III (the system description) per DC 200 — discover DC1–DC7 from live systems and repos, interview only for commitments and boundaries, file as a DRAFT PR
---
# /shadow:system-description — Draft Section III (DC 200)

Drafts the system description — Section III of the SOC 2 report — structured
per DC Section 200 (DC1–DC7). Three phases, strictly ordered: **gather** (zero
questions — nearly everything here is discoverable), **interview** (judgment
only — commitments and boundaries), **file** (zero questions).

**Honest note, stated to the user up front:** a CPA will review and reshape
this during the engagement. The draft's job is to save the blank-page week,
with every judgment call either answered by a named human or marked OPEN.

**Segregation of duties:** the agent never invents a service commitment, an
SLA number, or a CUEC decision — those come from the human or are OPEN.

## Phase 1: GATHER (no questions asked)

Discover each DC section from the sources that already exist:
- **DC1 — types of services:** repo READMEs, the product's marketing/security
  page, the scan report's service inventory.
- **DC2 — commitments & requirements (partial):** grep ToS/MSA/DPA/security
  page for stated uptime, support, security promises; grep `objectives.md` or
  CC3.1 evidence. Collect candidates — confirmation is interview material.
- **DC3 — components, five kinds:**
  *infrastructure* (scan report / `gcloud` inventory: projects, Cloud SQL,
  networking, regions); *software* (repos, languages, key dependencies, CI/CD,
  the shadow tooling itself); *people* (access register / latest access-review
  packet: roles, not just names); *procedures* (`sdlc/SDLC.md`, runbooks,
  the commands in `.claude/commands/shadow/`); *data* (data classification
  policy: classes, stores, flows, encryption at rest/in transit).
- **DC4 — system incidents:** incident tickets and hotfix/incident records in
  the archives for the period; "none" is a valid, stated answer.
- **DC5 — applicable TSC and related controls:** the shadow scope (which of
  the 61 criteria are in scope) and, per criterion, the control one-liners
  from `criteria/{ID}.md`.
- **DC6 — CUECs:** prepare the standard candidate set to offer: customer
  credential management, customer-side access control/authorization of their
  own users, timely reporting of suspected compromise.
- **DC7 — subservice organizations:** the vendor register's cloud/platform
  providers → carved-out CSOCs, explicitly including all of physical and
  environmental security (CC6.4) inherited from the cloud provider; note
  reliance on each subservice org's own SOC 2 report (date from the vendor
  register).

## Phase 2: INTERVIEW (judgment only, batched)

Only the commitments and boundary calls — evidence inline:

1. **Principal service commitments (DC2):** "Found these candidate
   commitments: {uptime claim from security page, support terms from MSA…}.
   Confirm each as a *principal* commitment, correct the numbers, or drop.
   Anything promised to customers that I didn't find?" Never invent an SLA.
2. **System requirements (DC2):** "Given the commitments, these system
   requirements follow: {encryption at rest, RPO/RTO, SSO…}. Confirm/amend."
3. **CUECs (DC6):** offer the standard set (customer credential management,
   customer-side access control) — "declare these? any others your contracts
   assume?"
4. **Boundary judgment calls:** anything ambiguous found in gather — "is the
   internal analytics pipeline inside the system boundary?", "is the marketing
   site in scope?" One batched question list.

Collect the name of the human who confirmed the commitments — it goes in the
draft's header. Unanswered items → **OPEN** in the draft, listed prominently.

## Phase 3: FILE (no questions asked)

1. **`system-description.md` as a PR to the policies repo** through the
   normal gates. Structure: title block (system name, period placeholder,
   **DRAFT** banner), then sections DC1–DC7 in order, each populated from
   gather + interview, sources cited inline (file paths, register entries,
   archive paths) so the CPA can trace every sentence. An **OPEN items** list
   sits at the top of the draft — every unconfirmed commitment, boundary, or
   CUEC. Frontmatter per the policy lifecycle (owner, version 0.x while
   DRAFT, approved_by the confirming human for the commitments only).
2. **One-screen summary:** sections drafted, word count, commitments confirmed
   (by whom) vs OPEN, CUECs declared, subservice orgs carved out (and whose
   SOC 2 reports are on file vs missing — cross-reference the vendor
   register), PR URL, and the honest note that CPA review comes next.

## STOP conditions
- No scan report and no `gcloud`/`gh` access → DC3 infrastructure would be
  fiction. Run `agent/01-scan-platform.md` first.
- No human confirms any commitment → file the draft with DC2 entirely OPEN
  and say so; a system description with invented SLAs is worse than none.
