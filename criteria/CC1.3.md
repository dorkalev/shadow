---
id: CC1.3
family: CC1 — Control Environment
category: Security (Common Criteria)
coso: COSO Principle 3
title: Structures, Reporting Lines, Authorities
weight: 2
automatable: partial
nature: document
---

# CC1.3 — Structures, Reporting Lines, Authorities

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 3: Management establishes, with board oversight, structures, reporting lines, and appropriate authorities and responsibilities in the pursuit of objectives.

## What it means
The auditor needs to see that it is defined *who is responsible for what* — especially for security. In a big company this is org charts and RACI matrices; in a 4-person startup it is a half-page document naming the security owner (usually a founder wearing the "Security Lead" hat), who owns infrastructure, who approves access, and who handles incidents. The point is that responsibilities are assigned deliberately, not assumed, and that authority to grant access or change production is limited to named people.

This criterion also covers how authority is *limited*: least-privilege assignment of admin roles in GitHub, GCP/AWS, and Google Workspace is the technical expression of "defines and limits authorities." If everyone is an org owner everywhere, you have not established structures and limits, no matter what the document says. Expect the auditor to compare the documented roles against actual admin-role membership in your systems.

Contractors and outsourced providers count too: if a contractor has production access, the document should say under what authority and who supervises them.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Considers all structures of the entity** — management designs structures considering the entity's objectives, size, and how it operates (including use of outsourced providers).
- **Establishes reporting lines** — reporting lines are designed so authority and responsibility are exercised and information flows to the right people.
- **Defines, assigns, and limits authorities and responsibilities** — authority is delegated and limited as appropriate at all levels; segregation of duties is considered where feasible.
- **Addresses specific requirements when defining authorities and responsibilities** (TSC supplemental) — includes responsibility for the design, implementation, and operation of controls over the system.
- **Considers interactions with external parties** (TSC supplemental) — structures account for vendors, contractors, and business partners interacting with the system.

## What the auditor will ask for
- Org chart or roles document (for a tiny company, a roles-and-responsibilities page is expected instead of a chart).
- Documented assignment of security responsibility (who is the Security Lead / security officer equivalent).
- Job descriptions or role definitions for in-scope roles (can be brief).
- Admin-role membership exports: GitHub org owners, GCP project IAM / AWS account admins, Google Workspace super admins — to compare against the documented structure.
- Access-authorization policy: who may approve access grants and production changes.
- Contractor agreements or a list of contractors with system access and their assigned responsibilities.
- Evidence that the structure was reviewed with board/oversight (often a line item in the quarterly oversight review — ties to CC1.2).

## How a tiny AI-first startup satisfies it
- `policies/roles-and-responsibilities.md`: lists each person, their role, and their security responsibilities. Explicitly names the Security Lead, the Infrastructure Owner, and the access approver. For a 2-founder company, the same person holds several hats — that's fine if written down, with a note on compensating controls where segregation of duties is impossible (e.g., PR review by the other founder, monitoring alerts visible to both).
- Keep admin roles minimal and mapped to the doc: exactly the named people are GitHub org owners, GCP `roles/owner` holders, and Workspace super admins. Everyone else gets member/editor-scoped roles.
- Access approval flows through Linear: an `access-request` ticket approved (comment) by the named approver before the grant. This gives you the "authorities exercised as documented" trail.
- Contractors: a short section in the roles doc plus a signed agreement; their access is time-boxed and reviewed in the quarterly access review.
- Reporting lines in a flat company: state that all personnel report to the founders and that security escalations go to the Security Lead; that sentence satisfies the reporting-lines expectation at this size.
- Review the roles doc annually and whenever someone joins/leaves (tie the update to the onboarding/offboarding checklist).

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `roles-and-responsibilities.md` exists and names a Security Lead | GitHub | `gh api repos/{org}/policies/contents/roles-and-responsibilities.md`; grep for `Security Lead` |
| GitHub org owners match documented admin list | GitHub | `gh api orgs/{org}/members?role=admin` vs. parsed allowlist in roles doc |
| GCP owner/editor bindings match documented list | GCP | `gcloud projects get-iam-policy <project> --format=json`, diff `roles/owner`+`roles/editor` members against allowlist |
| Workspace super admins match documented list | Workspace Admin API | `GET admin/directory/v1/users?query=isAdmin=true`, diff against allowlist |
| Roles doc updated within 12 months or since last team change | GitHub | Latest commit date on file; compare to most recent onboarding/offboarding ticket date in Linear |
| Access grants preceded by approved Linear ticket | Linear API + audit logs | Sample recent IAM changes (GCP audit log via `gcloud logging read`) and match to `access-request` issues with approver comment |
| Contractors with access are listed in roles doc | GitHub + doc | Diff GitHub outside collaborators (`gh api orgs/{org}/outside_collaborators`) against contractor section |
| Appropriateness of structure and reporting lines | MANUAL | Auditor inquiry; judgment-based |

## Evidence artifacts
- `policies/roles-and-responsibilities.md` — the roles document with git history.
- `evidence/admin-membership/` — quarterly JSON exports of GitHub org owners, GCP IAM policy, and Workspace admin list (generated by the shadow tool, committed to `compliance-archives`).
- Linear export of `access-request` tickets for the audit period.
- Contractor agreements / list, in `evidence/contractors/`.
- Quarterly oversight-review minutes showing the structure was reviewed (shared with CC1.2 evidence).
