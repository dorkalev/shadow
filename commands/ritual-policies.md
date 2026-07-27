---
description: Annual policy re-approval, staff attestation, and micro-training in one session — walk the diffs, bump frontmatter, collect signed attestation comments (CC5.3, CC1.1, CC1.4, CC2.2)
---
# /shadow:ritual-policies — Re-Approval, Attestation & Micro-Training

Runs the annual policy cycle (CC5.3 re-approval, CC1.1/CC2.2 communication,
CC1.4 training) as one session. Three phases, strictly ordered: **gather**
(zero questions), **interview** (judgment only), **file** (zero questions).
Never ask the user anything a tool can discover.

**Segregation of duties — absolute:** the agent NEVER approves a policy,
NEVER defaults a re-approval to yes, and the approving human's name goes into
the frontmatter. Attestations are each person's own GitHub comment — the agent
never attests for anyone. Unapproved policies and absent attesters are
recorded as **OPEN**.

## Phase 1: GATHER (no questions asked)

**Step 1 — find stale policies.** In the policies repo, parse every policy's
frontmatter (`owner`, `version`, `approved_by`, `approved_at`, `review_by`).
Flag any with `review_by` in the past or within 30 days. None flagged → report
"pack current, next review_by {date}" and stop.

**Step 2 — diff each stale policy against its last approved version:**
```bash
git log --follow -p -- {policy}.md   # find the commit at/before approved_at
git diff {approved_commit} HEAD -- {policy}.md
```
Summarize material changes per policy (substantive rule changes, not typo
churn). No diff → mark "no changes since last approval".

**Step 3 — list staff** from the access register / latest access-review packet
in archives (GitHub org members ∪ tracker seats). This is the attestation
roster. Note new joiners since the last attestation cycle
(`evidence/{YYYY-1}/attestations.md` in archives, if present).

**Step 4 — draft the micro-training** (do not ask; derive): a 10-minute md
tailored to THIS company's stack, sourced from the scan report and registers —
phishing (the actual email/SSO setup), secrets handling (the actual secret
stores and the "names only, never values" rule), incident reporting (the
actual runbook's first step and channel), AI-tool data rules (the actual LLM
vendors from the vendor register and the AI Development & Agent Use Policy).
End with 3 quiz questions answerable from the training text.

## Phase 2: INTERVIEW (judgment only, batched)

Walk each stale policy, diff summary inline:
> **Incident Response Plan** (v1.2, review_by 2026-07-01 — overdue). Material
> changes since approval: {2-line summary or "none"}.
> Re-approve as-is, re-approve with these changes, or hold for edits?

Policies with no changes get the explicit question anyway: "no changes —
reapprove as-is?" (never assume yes).

Then once: **"Who is the approver of record for this re-approval?"** — collect
the human's name (the owner may differ per policy; ask per-policy only if the
user says approvers differ).

Held-for-edits or unanswered policies → **OPEN** (frontmatter untouched;
Runbook 03 will keep decaying CC5.3 until resolved — that is correct).

## Phase 3: FILE (no questions asked)

1. **Frontmatter bump via PR to the policies repo** through the normal gates:
   for each re-approved policy set `version` +1 (minor for as-is, as
   appropriate for changed), `approved_by` = the named human, `approved_at` =
   today, `review_by` = +12 months. OPEN policies are not touched.
2. **One attestation issue per person** on the roster — Linear MCP
   (`save_issue`) when available, otherwise `gh issue create`. Body: link to
   the policy pack at the re-approved version ("policy pack v{X}"), the full
   micro-training md, the 3 quiz questions, and the instruction:
   *"Comment `I attest` plus your three quiz answers to sign. Your comment is
   the training and attestation record."* The attestation comments ARE the
   training + attestation evidence — no separate form exists.
3. **Track completion in evidence:** write/update
   `evidence/{YYYY}/attestations.md` on `compliance-archives`: one row per
   person — issue link, attested-at timestamp and quiz answers present
   (verbatim check, not paraphrase), or **OPEN** for absentees. Also record
   the re-approval walk (per-policy decision, approver, diff summary). Commit
   message: `policy-reapproval {YYYY}: approved by {name}`. Re-run this
   command later to sweep attestation comments and close OPEN rows.
4. **One-screen summary:** policies re-approved / OPEN, new pack version,
   approver, attestation issues opened (n) and completed (n), PR URL, archive
   commit SHA, and the date the next review_by lands.

## STOP conditions
- Frontmatter missing/unparseable on any policy → that is itself a CC5.3
  finding; open a ticket to fix the frontmatter, mark the policy OPEN.
- No human responds → nothing gets a new approval date. File the walk record
  with everything OPEN and say so. A stale-but-honest pack beats a
  self-approved one.
