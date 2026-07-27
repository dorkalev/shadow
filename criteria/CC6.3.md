---
id: CC6.3
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: Role-Based Access and Least Privilege
weight: 3
automatable: partial
nature: technical
---

# CC6.3 — Role-Based Access and Least Privilege

## Criterion (AICPA TSP Section 100, verbatim)
> The entity authorizes, modifies, or removes access to data, software, functions, and other protected information assets based on roles, responsibilities, or the system design and changes, giving consideration to the concepts of least privilege and segregation of duties, to meet the entity's objectives.

## What it means

Where CC6.2 covers whether an account should exist, CC6.3 covers what that account can do. Access must be granted according to role, changed when the role changes, and periodically re-verified — with least privilege (no more access than the job needs) and segregation of duties (no single person able to both perpetrate and conceal a problem) considered explicitly.

For a 1–10 person startup, "considered" is the operative word. True segregation of duties is often impossible when two founders run everything, and auditors know it. What they expect is: (a) a written role-to-access matrix, even if it has three roles; (b) compensating controls where SoD breaks down — chiefly branch protection so no one merges their own unreviewed code to production, and audit logging so admin actions are attributable; and (c) a recurring access review that actually removes stale privilege, with evidence.

The classic tiny-startup failures: everyone is a GCP Owner "because it's easier," everyone is a GitHub org owner, the first engineer still has billing admin from an emergency two years ago, and no access review has ever been documented. Quarterly reviews with a diffable IAM export are cheap and close all of these.

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Creates or modifies access based on roles and responsibilities** — access changes follow role definitions and are authorized.
- **Uses role-based access controls** — RBAC (groups, predefined cloud roles, GitHub teams) is used to administer access rather than ad-hoc individual grants.
- **Considers least privilege** — access is limited to what the role requires, including for service accounts and automation.
- **Considers segregation of duties** — incompatible duties are separated, or compensating controls (independent review, logging/monitoring) are applied where headcount makes separation impractical.
- **Reviews access** — the appropriateness of access rights is reviewed on a periodic basis and inappropriate access is removed.

## What the auditor will ask for

- The role/access matrix: which roles exist, and what standing access each role has per system.
- Completed access review evidence for each quarter in the period: who reviewed, when, what was found, and tickets showing removals actioned.
- Current IAM/permission exports (GCP IAM policy, GitHub org roles and team/repo permissions, Workspace admin roles) to test against the matrix.
- Access-change requests during the period (role changes, elevated grants) with authorization.
- Description of segregation-of-duties conflicts and the compensating controls (e.g., branch protection settings, admin audit logs).
- Evidence privileged/admin access is limited (count of GCP owners/editors, GitHub org owners, Workspace super admins — auditor wants each ≤2 and justified).
- Service-account role assignments with justification for any broad roles.

## How a tiny AI-first startup satisfies it

- **A one-page role matrix.** Typically three roles: `founder/admin`, `engineer`, `contractor`. For each: Workspace (user vs. super admin), GitHub (member + team vs. org owner), GCP (scoped predefined roles vs. break-glass owner), Linear, production data access. Version-controlled in the policy repo.
- **Grant via groups, not individuals.** GCP IAM bindings go to Google Groups (`eng@`, `admins@`); GitHub access goes via teams. New engineer = add to group, offboarding = remove from group — and the matrix stays testable.
- **Least privilege in practice.** Engineers get `roles/editor`-narrower predefined roles on the dev project and read-mostly on prod; exactly two humans can assume break-glass admin, and usage is logged (Cloud Audit Logs admin activity is on by default — export it). CI service accounts get only the deploy roles they need; the AI agent's token is a fine-grained PAT scoped to specific repos with no admin rights.
- **SoD by compensating control.** Written statement: with N<10 people, code SoD is enforced technically — branch protection requires one non-author review before merge to `main`; direct pushes and force pushes are blocked even for admins (`enforce_admins: true`); production deploys run only from CI off `main`. Admin actions in GCP/Workspace are covered by immutable audit logs.
- **Quarterly access review with teeth.** The shadow tool generates the review packet (all exports below); a founder reviews line-by-line, marks keep/remove, removals become Linear tickets closed within 7 days. The signed-off packet is the evidence.
- **Role changes get a ticket.** Same Linear template family as CC6.2: "Access change — <name>" with approver, before/after access, and date.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| GitHub org owner count ≤ 2 (or documented) | GitHub | `gh api /orgs/{org}/members?role=admin` → count vs. allowed list |
| Repo permissions flow through teams, not direct collaborators | GitHub | `gh api /repos/{org}/{repo}/collaborators?affiliation=direct` → flag unexpected direct grants |
| Branch protection enforces review incl. admins | GitHub | `gh api /repos/{org}/{repo}/branches/{default}/protection` → `required_pull_request_reviews.required_approving_review_count ≥ 1`, `enforce_admins.enabled` |
| GCP owners/editors limited to allowed list | GCP | `gcloud projects get-iam-policy --format=json` → members of `roles/owner`, `roles/editor` vs. matrix |
| IAM bindings use groups, minimal `user:` grants | GCP | Same policy export → count `user:` vs `group:`/`serviceAccount:` members |
| No service account with `roles/owner` or `roles/editor` on prod | GCP | Policy export filter `serviceAccount:` members on broad roles |
| IAM policy drift since last review | GCP | Diff current `get-iam-policy` output vs. `evidence/access-reviews/<last>/iam-policy.json` |
| Workspace super admin count ≤ 2 | Google Workspace | Admin SDK Directory API `users.list` → `isAdmin` flag count |
| Quarterly access review completed on time | Linear + files | Linear issue labeled `access-review` completed in quarter; file-existence check `evidence/access-reviews/<quarter>/signoff.md` |
| Review removals actioned within 7 days | Linear | Linear API: issues spawned from review closed ≤ 7 days after review date |
| Role matrix document exists and recently updated | Repo | File-existence + git log age check on `policies/role-matrix.md` |
| Admin audit logs being exported/retained | GCP | `gcloud logging sinks list` → sink for admin activity exists |
| Elevated-access grants had approval tickets | Linear | MANUAL — sample IAM diff entries against access-change tickets |

## Evidence artifacts

- `policies/role-matrix.md` — role-to-access matrix, version-controlled (git history shows maintenance).
- `evidence/access-reviews/<quarter>/` — the full review packet: `iam-policy.json`, `github-members.json`, `github-teams.json`, `workspace-admins.csv`, `linear-members.json`, plus `signoff.md` with reviewer name/date and keep/remove decisions.
- Linear issues labeled `access-review` and `access-change` — exported per quarter alongside the packet.
- `evidence/gcp/audit-log-sink-<date>.json` — proof admin activity logs are retained.
- Git history of `policies/role-matrix.md` and branch-protection exports demonstrating the SoD compensating controls were in force all period.
