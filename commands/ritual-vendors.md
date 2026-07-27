---
description: Annual vendor re-review as a dialogue — auto-detect vendor drift, walk each vendor's risk/DPA/SOC 2 status with a human, file the updated register (CC9.2, P6.4/P6.5)
---
# /shadow:ritual-vendors — The Annual Vendor Re-Review

Turns the annual vendor review (CC9.2; P6.4/P6.5 if Privacy is scoped) into an
interview over real data. Three phases, strictly ordered: **gather** (zero
questions), **interview** (judgment only), **file** (zero questions). Never
ask the user anything a tool can discover.

**Segregation of duties — absolute:** the agent NEVER rates a vendor itself,
NEVER defaults "still used / risk unchanged / DPA current" to yes, and the
responding human's name goes on the record as reviewer. Any vendor left
undiscussed is recorded as **OPEN — not reviewed**.

## Phase 1: GATHER (no questions asked)

**Step 1 — read the current register.** Fetch `vendor-register.md` from the
policies repo. Per entry note: vendor, purpose, data access, inherent risk
rating, DPA/terms status, SOC 2 report on file (date), breach-notification
commitment, last-reviewed date.

**Step 2 — detect vendor DRIFT automatically.** Build the "detected in use"
list from live systems:
- **GitHub Apps installed:**
  `gh api /orgs/{org}/installations --jq '.installations[].app_slug'`
  (org) and per-repo `gh api /repos/{o}/{r}/installations` where applicable.
- **Actions secrets hinting at services** (names only — never values):
  `gh api /repos/{o}/{r}/actions/secrets --jq '.secrets[].name'` across the
  org's repos (and `/orgs/{org}/actions/secrets`). `ANTHROPIC_API_KEY`,
  `STRIPE_*`, `SENTRY_*`, `SLACK_WEBHOOK_URL` etc. each imply a vendor.
- **Workspace OAuth scopes:** no API available → list as a manual note; ask
  the user to paste the Workspace admin's third-party-app list verbatim during
  the interview (filed as a manual attachment, absent = OPEN).
- **LLM providers explicitly:** any Anthropic/OpenAI/Google AI key, SDK
  dependency, or MCP server in the stack is a vendor with data access —
  include them even if someone considers them "just an API".

Diff detected-in-use against the register: **UNREGISTERED** (detected, not in
register), **STALE** (registered, no trace found), **MATCHED**.

## Phase 2: INTERVIEW (judgment only, batched)

**Per existing vendor** (batch MATCHED ones by risk tier, evidence inline):
> **Anthropic** (inherent risk 4, DPA signed 2025-06, SOC 2 on file dated
> 2025-03, breach notification: 72h in DPA). Detected in use:
> `ANTHROPIC_API_KEY` in 3 repos.
> 1. Still used? 2. Inherent risk still 4? 3. DPA/terms current?
> 4. Current SOC 2 report on file — what's its date? 5. Breach-notification
> commitment confirmed?

STALE vendors: "no trace found in GitHub/secrets — still used (where?), or
retire from the register?"

**Per detected-but-unregistered vendor**, one at a time with the detection
evidence:
> **Sentry** — detected via `SENTRY_DSN` secret in `api` repo; not in the
> register. Register it (rate inherent risk 1–5, state what data it can
> access) or flag for removal (opens a removal ticket)?

Finally: **"Who is signing this review as reviewer?"** — collect the human's
name. Unanswered vendors → **OPEN**.

## Phase 3: FILE (no questions asked)

1. **Updated `vendor-register.md` as a PR to the policies repo** through the
   normal gates: revised ratings, newly registered vendors (with data-access
   notes and detection evidence), retirements struck with reason, SOC 2 report
   dates, last-reviewed = today per reviewed vendor, frontmatter bumped
   (version / approved_by = the named human / approved_at / review_by +12mo).
   OPEN vendors stay marked OPEN.
2. **Review-evidence md to archives** under `evidence/{YYYY}/` on
   `compliance-archives`: the detection diff (unregistered/stale/matched),
   per-vendor answers and who gave them, pasted Workspace OAuth list verbatim
   (or OPEN), reviewer name, date, link to the register PR. Commit message:
   `vendor-review {YYYY}: signed by {reviewer}`.
3. **Tickets for gaps** — Linear MCP (`save_issue`) when available, otherwise
   `gh issue create`: one per missing/expired DPA, one per missing or stale
   (>12mo) SOC 2 report ("request current report from {vendor}"), one per
   flagged-for-removal vendor (with the exact secret/app to revoke). Each
   ticket links the archived review.
4. **One-screen summary:** vendors reviewed / added / retired / OPEN, risk
   changes, missing DPAs and SOC 2 reports (ticket IDs), reviewer, register
   PR URL, archive commit SHA.

## STOP conditions
- `gh api` calls all fail (no org access) → fix credentials first; a review
  that can't see live usage can't detect drift. Register-only review is a
  degraded mode — say so in the evidence if the user insists.
- No human responds → file the review record with ALL vendors OPEN, open no
  register PR, and say so in the summary.
