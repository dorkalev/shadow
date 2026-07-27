---
id: CC5.3
family: CC5 — Control Activities
category: Security (Common Criteria)
coso: COSO Principle 12
title: Policies and Procedures Deployed
weight: 2
automatable: partial
nature: document
---

# CC5.3 — Policies and Procedures Deployed

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 12: The entity deploys control activities through policies that establish what is expected and in procedures that put policies into action.

## What it means
Controls have to be *deployed* through written policies (what is expected) and procedures (how it's actually done), with named accountability, timely execution, competent performers, and periodic reassessment. This is the criterion behind the auditor's opening PBC request: "send us all your policies, with version history, approval, and evidence people have read them."

The trap for startups is in both directions. No policies is an automatic pile of exceptions across the whole report. But grabbing a 40-document enterprise policy pack you'll never follow is worse in a Type II: the auditor tests whether you *operate* the procedures you wrote, and every unmet promise ("quarterly DR tabletop exercises", "CAB approval for all changes") becomes an exception. The winning move for a tiny team is a small set of short policies that describe what you genuinely do, enforced by tooling wherever possible.

A git-native policies repo is the ideal implementation: markdown policies, PR review as documented approval, git history as version control, a required-review CODEOWNERS rule as the accountability mechanism, and per-hire acknowledgment recorded in-repo. The shadow tool can then verify freshness, approval, and acknowledgment coverage mechanically — which is most of what the auditor tests here.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Policies and procedures are established to support deployment of management's directives.
- Responsibility and accountability for executing policies and procedures is established with designated personnel/management.
- Personnel perform control activities in a timely manner as defined by the policies.
- Responsible personnel investigate and act on matters identified as a result of executing control activities (corrective action).
- Control activities are performed by competent personnel with sufficient authority.
- Management periodically reassesses policies and procedures for continued relevance, and refreshes them when necessary.

## What the auditor will ask for
- The complete policy set with owner, approval evidence, effective date, and last-review date for each.
- Version history demonstrating annual review (or documented "reviewed, no changes needed").
- Evidence of employee acknowledgment — every current team member, and new hires within onboarding SLA.
- Sample procedures in action: an access request handled per the access policy, an incident handled per the IR plan, a change merged per the change policy.
- The policy-exception process and any exceptions granted during the period.
- Mapping of policies to controls/criteria (often combined with the CC5.1 control matrix).

## How a tiny AI-first startup satisfies it
- A `policies/` GitHub repo containing the minimal viable set (~10 short docs): information security (umbrella), acceptable use / code of conduct, access control, change management, incident response, vulnerability & deficiency management, vendor management, data classification & retention, business continuity/DR, and risk management. Each has YAML front-matter: `owner`, `approved_by`, `effective_date`, `last_reviewed`.
- Approval = merged PR reviewed by the non-authoring founder; CODEOWNERS requires founder review on every policy change. Git history is the version log — no separate document-management system needed.
- Annual review: a scheduled Linear ticket per policy; even a no-change review lands a commit bumping `last_reviewed`. The shadow tool flags any policy > 12 months stale.
- Acknowledgments in-repo: `evidence/acknowledgments/<person>.md` listing policy versions acknowledged with date (or a PR approval by each member on an acknowledgment file). New hires acknowledge within 30 days — onboarding Linear template includes the step.
- Keep procedures inside the policies as short "How we do it here" sections naming actual tools ("access is requested via a Linear ticket; granted by a founder; reviewed quarterly by the shadow tool") — so operating evidence generates itself from systems you already use.
- Write only what you do. Before merging any policy, ask: can the shadow tool or an auditor verify this sentence? If not, cut or rewrite it.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Required policy set present (checklist of ~10 filenames) | GitHub | `gh api repos/{org}/policies/contents/` — diff against required list |
| Every policy has valid front-matter (owner, approved_by, dates) | GitHub | Fetch files, parse YAML front-matter, validate fields |
| No policy `last_reviewed` older than 12 months | GitHub | Parse front-matter dates; cross-check with commit history |
| All policy changes merged via reviewed PR (no direct pushes) | GitHub | Commit list on repo → each commit has associated PR with approving review |
| CODEOWNERS enforces founder review on policies repo | GitHub | File existence + branch protection `require_code_owner_reviews` |
| Acknowledgment file exists for every current org member | GitHub | Compare `evidence/acknowledgments/` contents vs. `gh api orgs/{org}/members` |
| New hires acknowledged within 30 days | GitHub + Linear | Member add date (org audit log) vs. acknowledgment file commit date |
| Annual policy-review tickets completed | Linear | Query recurring `policy-review` issues — state Done within period |
| Procedures actually followed (sampled walkthroughs) | Multiple | MANUAL — auditor samples; shadow tool assists by pre-collecting samples |
| Policy content quality and fit to actual operations | — | MANUAL |

## Evidence artifacts
- The `policies/` repo itself — full contents plus git history (the single most-requested SOC 2 artifact).
- Per-policy metadata report (generated by shadow tool): filename, owner, approved_by, effective date, last reviewed, approving PR link — archived to `compliance-archives`.
- `evidence/acknowledgments/` — per-person acknowledgment records covering the whole team.
- PR exports (JSON with reviews) for every policy change in the period.
- Linear export of completed annual policy-review tickets.
- Onboarding checklist template (Linear) showing the acknowledgment step, plus a completed example from an in-period hire.
