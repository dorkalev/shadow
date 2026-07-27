---
id: CC1.1
family: CC1 — Control Environment
category: Security (Common Criteria)
coso: COSO Principle 1
title: Commitment to Integrity and Ethics
weight: 2
automatable: partial
nature: document
---

# CC1.1 — Commitment to Integrity and Ethics

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 1: The entity demonstrates a commitment to integrity and ethical values.

## What it means
The auditor wants to see that the company has explicitly stated what "behaving properly" means — usually a Code of Conduct — and that this isn't a dead document. People must know the standards exist, acknowledge them, and there must be some mechanism (however lightweight) for spotting and dealing with violations. Tone at the top matters most: in a 5-person company, "the top" is the founders, so the founders' own written expectations and behavior are the control environment.

For a tiny AI-first startup this criterion is almost entirely about documentation and acknowledgment hygiene. You do not need an ethics hotline or an HR department. You need: a versioned Code of Conduct, proof that every person (including contractors with system access) read and accepted it, and a documented path for reporting concerns — even if that path is "email either founder, or the external advisor if the concern involves a founder."

Auditors also expect standards of conduct to extend to how the company treats customer data and, increasingly, to AI usage — e.g., an acceptable-use rule that production customer data is never pasted into unapproved AI tools. Putting that clause in the Code of Conduct or an Acceptable Use Policy is a cheap, credible signal.

## Points of focus (2022 revision, summarized)
These are guidance from the COSO framework as mapped in the 2022 TSC — illustrative, not required:
- **Sets the tone at the top** — leadership's directives, actions, and behavior demonstrate the importance of integrity and ethical values.
- **Establishes standards of conduct** — expectations are defined and understood at all levels and by outsourced service providers and business partners.
- **Evaluates adherence to standards of conduct** — processes exist to evaluate performance of individuals and teams against the standards.
- **Addresses deviations in a timely manner** — deviations are identified and remedied promptly and consistently.
- **Considers contractors and vendor employees** (TSC supplemental) — standards of conduct and adherence evaluation cover non-employees with system access.

## What the auditor will ask for
- Code of Conduct / employee handbook, with version history and approval date.
- Acceptable Use Policy (including AI-tool usage rules, if referenced).
- Evidence of acknowledgment by all current personnel and in-scope contractors (signed forms, GRC-tool policy-acceptance records, or PR-based sign-off).
- Onboarding checklist for a sample of hires during the audit period showing conduct acknowledgment.
- Description of the process for reporting and handling ethics/conduct concerns, plus records of any incidents (or attestation that none occurred).
- Evidence that policies were reviewed/re-approved within the audit period (annual review).
- Background check evidence or documented policy stating the screening approach (often sampled here or under CC1.4).

## How a tiny AI-first startup satisfies it
- Keep a `policies/` GitHub repo with `code-of-conduct.md` and `acceptable-use-policy.md` as versioned markdown. Approval = a PR merged by a founder; the merge commit and PR review are the approval evidence.
- Include an AI acceptable-use clause: approved AI tools, prohibition on sending customer data/secrets to unapproved models, requirement that AI-generated code goes through normal PR review.
- Acknowledgment mechanism: each person (employees and contractors with repo/cloud access) opens or is assigned a Linear ticket "Acknowledge policies vX" and closes it with a comment "Read and agree," or signs via the GRC tool policy acceptance. Do this at onboarding and after each annual policy revision.
- Reporting channel: a paragraph in the Code of Conduct naming who to contact (founder; external advisor for founder-related concerns). For a 3-person company this is acceptable — auditors expect proportionality.
- Annual review: a recurring Linear ticket "Annual policy review" owned by a founder; the resulting policy-repo PR (even a no-change re-approval) is the evidence.
- Actually enforce it once if needed and document it: a single Linear ticket recording how a deviation was handled is stronger evidence than any policy text.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `code-of-conduct.md` and `acceptable-use-policy.md` exist in policies repo | GitHub | `gh api repos/{org}/policies/contents/code-of-conduct.md` (and AUP) returns 200 |
| Policies reviewed within last 12 months | GitHub | `gh api repos/{org}/policies/commits?path=code-of-conduct.md` — latest commit date < 365 days, or frontmatter `last_reviewed` field parsed from file |
| Policy approval went through PR review | GitHub | `gh pr list --repo {org}/policies --state merged` — merged PRs touching policy files have ≥1 approving review |
| Every active member has a closed policy-acknowledgment ticket for current policy version | Linear API + GitHub | Cross-reference `gh api orgs/{org}/members` against Linear issues labeled `policy-ack` with matching version label, state = Done |
| AI acceptable-use clause present | GitHub | Fetch AUP content, grep for required section heading (e.g., `## AI and LLM Usage`) |
| Conduct-concern reporting channel documented | GitHub | Grep Code of Conduct for a `## Reporting Concerns` section |
| Tone-at-the-top / actual adherence in practice | MANUAL | Auditor inquiry with founders; cannot be automated |
| Handling of actual deviations | MANUAL | Review of incident/HR records if any occurred |

## Evidence artifacts
- `policies/code-of-conduct.md`, `policies/acceptable-use-policy.md` — the policies themselves, with git history as the version/approval trail.
- `evidence/policy-acknowledgments/` — export of Linear `policy-ack` tickets (CSV/JSON) or GRC-tool acceptance report, snapshotted quarterly to the `compliance-archives` branch.
- `evidence/annual-review/` — link/export of the annual policy-review PR and the Linear review ticket.
- `evidence/onboarding/<person>.md` — per-hire onboarding checklist showing conduct acknowledgment and background-screening note.
- Shadow monitor output: "policy freshness" and "acknowledgment coverage" check results, archived with timestamps.
