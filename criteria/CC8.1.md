---
id: CC8.1
family: CC8 — Change Management
category: Security (Common Criteria)
coso: null
title: Authorized and Controlled Change Management
weight: 3
automatable: full
nature: technical
---

# CC8.1 — Authorized and Controlled Change Management

## Criterion (AICPA TSP Section 100, verbatim)
> The entity authorizes, designs, develops or acquires, configures, documents, tests, approves, and implements changes to infrastructure, data, software, and procedures to meet its objectives.

## What it means

CC8.1 is the densest criterion in the common criteria: eight verbs (authorizes, designs, develops or acquires, configures, documents, tests, approves, implements) applied to four targets (infrastructure, data, software, procedures). In practice it means every change to production must be traceable through a lifecycle: someone authorized it, its design/intent was recorded, it was built and tested, someone other than the author approved it, and its deployment is documented. The auditor's test is brutally sample-based: they pull 15–25 changes from the period and walk each one backward — ticket? review? tests? approval? who deployed? If any link is missing on any sample, that's an exception in the report.

This is where tiny AI-first startups traditionally fail, because velocity and AI-generated code tempt direct pushes, self-merges, and "docs later." It is also where they can be *better* than enterprises: with the whole SDLC in GitHub + Linear, every one of the eight verbs can be enforced mechanically and evidenced automatically, with zero reliance on human diligence at audit time. That is this tool's flagship claim: the change record is produced as a side effect of shipping, and gaps are detected the day they occur, not discovered in a PBC scramble.

"Changes to procedures" is easy to miss: policy documents are in-repo and edited through the same PR flow, so procedure changes ride the same rails. "Data" changes (migrations, backfills) run as code in the repo — a migration file in a PR is a documented, reviewed, tested data change.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Manages changes throughout the system life cycle using a defined SDLC process covering infrastructure, data, software, and procedures.
- Authorizes changes prior to development and tracks them (e.g., through a ticketing system); designs and develops changes in accordance with objectives.
- Documents changes — nature, configuration details, and rollback — to support ongoing operation and incident diagnosis.
- Tests changes prior to implementation, in an environment separate from production, with test data protected.
- Approves changes prior to implementation by personnel other than the developer; segregates access such that developers cannot unilaterally deploy.
- Identifies and evaluates changes to confidential/personal data handling as part of the change process.
- Provides for emergency changes with after-the-fact documentation and approval; detects and addresses unauthorized changes.
- Considers patches and infrastructure/baseline configuration changes within the same discipline.

## What the auditor will ask for
- SDLC/change-management policy describing the required flow, approval rules, emergency-change procedure, and segregation of duties.
- Population of all production changes in the period (merged PR list to release branches, plus release events) — auditors select their own sample from *your complete population*, so bypassed changes that appear in deploy logs but not the PR list are how you fail.
- For each sampled change: the ticket (authorization + design intent), the PR (documentation of what changed and why), review approval by a non-author, CI test results, and the deploy/release record with deployer identity.
- Branch protection / ruleset configuration for staging and main, with change history over the period (proving the gate was up the whole time, not just at audit).
- Emergency/hotfix changes with their after-the-fact documentation and approvals.
- Evidence of how unauthorized changes would be detected, and any detections with follow-up.
- For infrastructure: sample of Terraform changes with the same trail. For procedures: sample of policy-doc PRs.

## How a tiny AI-first startup satisfies it

The enforced SDLC, end to end — each stage maps to the criterion's verbs:

1. **Authorize** — every change starts as a **Linear ticket** (feature, bug, chore, or `hotfix`/`release` types). Ticket creation with priority/assignment is the authorization record; nothing merges without one.
2. **Design/document (intent)** — the ticket carries the problem statement and approach; substantive design lives on the ticket or a linked doc before the branch exists.
3. **Develop** — work happens on a branch named **`{TICKET-ID}-short-description`** (e.g. `ABC-142-rotate-sa-keys`). The naming convention is machine-checkable and makes ticket traceability structural, not conventional. AI agents (Claude/Codex via the spec-first workflow) develop on these branches like any engineer — the controls below apply identically to AI-authored changes.
4. **Document (change)** — the PR targets **staging** and must contain: a **ticket table** (ID, title, link) and a **changes-per-file section** explaining every touched file. This makes each PR a self-contained change record.
5. **Test** — CI runs the test suite on the PR; the environment split (staging branch → staging env, main → production) keeps testing out of production.
6. **Review/approve** — two independent gates:
   - The **review bot** posts findings (critical/major/minor).
   - The **SOC 2 compliance CI agent** verifies, per PR: (a) ticket traceability — branch name and ticket table resolve to real, open Linear tickets; (b) per-file change traceability — every file in the diff is explained in the changes-per-file section; (c) test coverage — changed logic has corresponding tests or a justified exemption; (d) **review gate** — fails if unresolved critical/major review findings remain; (e) emits a **confidence score 0–100** with a pass threshold. A human (or the ruleset-required non-author approval) plus green compliance check gates the merge.
7. **Implement (merge)** — branch protection/rulesets on staging and main require: PR-only merges, passing required checks (tests + compliance agent), non-author approval, no force pushes.
8. **Archive (post-merge)** — a post-merge workflow writes an **audit record — JSON + human-readable MD — to the `compliance-archives` branch**: PR metadata, ticket IDs, approver, check run results, compliance score, diff summary, timestamps. The archive is append-only and survives repo history games; it is the auditor-facing population.
9. **Release** — production deploys are **fast-forward merges staging → main**, each with a **release ticket** in Linear and its own archive record listing the included PRs/tickets. Fast-forward-only means main is always an exact, already-reviewed staging state — no new code enters at release time.
10. **Detect bypasses** — the shadow continuously diffs commits on staging/main against archive records. Any commit without a corresponding archived PR = **bypass merge**, flagged same-day as a security event (CC7.2/CC7.3). Legitimate emergencies use the **hotfix procedure**: push to main is allowed to happen in a true SEV, but requires an immediate backport PR to staging carrying a `hotfix` ticket, after-the-fact review, and an archive record — converting the bypass into a documented emergency change per the points of focus.
11. **Procedures and infrastructure** — Terraform and `policies/*.md` live in the same repos and ride the same pipeline, covering the "infrastructure" and "procedures" targets with zero extra process.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Rulesets enforce PR-only, required checks, non-author approval on staging+main | GitHub | `gh api /repos/{org}/{repo}/rulesets` + `/rules/branches/{branch}`; diff vs required baseline |
| Ruleset unchanged / change history reviewed | GitHub | `gh api /orgs/{org}/audit-log` actions `protected_branch.*`, `repository_ruleset.*` |
| Every merged PR has valid ticket table + resolvable Linear ticket | GitHub+Linear | PR body parse via `gh pr view --json body`; Linear API ticket lookup |
| Branch naming matches `{TICKET-ID}-desc` | GitHub | `gh pr list --json headRefName` regex + ticket resolution |
| Changes-per-file section covers full diff | GitHub | compliance agent check re-runnable: diff file list vs PR body section |
| Required checks (tests, compliance agent, review bot) passed on every merged PR | GitHub | `gh api /repos/{org}/{repo}/commits/{sha}/check-runs` |
| Non-author approval present on every merged PR | GitHub | `gh api /repos/{org}/{repo}/pulls/{n}/reviews` — approver ≠ author |
| Compliance confidence score ≥ threshold | archives | parse score from archive JSON on `compliance-archives` |
| Archive record exists for every merge to staging | GitHub | list merge commits on staging vs archive index — missing = pipeline fault |
| **Bypass merges: zero unarchived commits on staging/main** | GitHub | `gh api /repos/{org}/{repo}/commits?sha=main` vs archive records; flag orphans |
| Hotfix pushes have backport PR + hotfix ticket + retro approval | GitHub+Linear | orphan commits on main matched to backport PRs referencing them |
| Releases are fast-forward staging→main with release ticket | GitHub+Linear | merge-base check (main ancestor of staging at release); release archive record parse |
| Unresolved critical/major review findings block merge | GitHub | compliance agent check-run conclusion history |
| Policy/Terraform changes went through same flow | GitHub | path-filtered merged-PR query on `policies/`, `*.tf` |
| SDLC policy exists, reviewed <12mo | repo | file-existence `policies/change-management.md` + review date |
| Reviewer actually read the change (quality of review) | — | MANUAL — auditor judgment; comment depth on sampled PRs helps |

## Evidence artifacts
- **`compliance-archives` branch** — the crown jewel: one JSON + one MD record per merged PR and per release (metadata, tickets, approvals, check results, compliance score, diff summary). Append-only, timestamped, exportable as the complete change population.
- Bypass-detection reports (scheduled shadow runs) — including the all-clear runs, which prove continuous detection across the period.
- Ruleset/branch-protection exports: `evidence/github-rulesets/YYYY-MM-DD.json` plus org audit-log pulls for ruleset changes.
- Linear: full ticket set (auto-linked from archives), release tickets, hotfix tickets with backport links.
- `policies/change-management.md` — the SDLC policy matching what the pipeline enforces.
- CI workflow definitions in-repo (`.github/workflows/`) — the control's own configuration, git-versioned across the period.
- Postmortem/backport records for any emergency change, closing the loop on the documented exceptions.
