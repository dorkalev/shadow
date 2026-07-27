---
id: CC6.2
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: User Registration and Deprovisioning
weight: 3
automatable: partial
nature: technical
---

# CC6.2 — User Registration and Deprovisioning

## Criterion (AICPA TSP Section 100, verbatim)
> Prior to issuing system credentials and granting system access, the entity registers and authorizes new internal and external users whose access is administered by the entity. For those users whose access is administered by the entity, user system credentials are removed when user access is no longer authorized.

## What it means

CC6.2 is the identity lifecycle criterion: nobody gets an account before someone authorized it, and every account dies when the person leaves. The auditor tests it as two populations — everyone who joined during the period (show the authorization that preceded each account) and everyone who left (show the credentials were removed, and how fast). "External users" matters too: contractors, outside collaborators on GitHub repos, and vendor accounts you administer are all in scope.

At a tiny startup this criterion fails on informality, not malice. A founder adds a contractor to a repo from their phone; no ticket exists. Someone leaves and their Google account is suspended, but their GitHub personal account is still an org member, their PAT still works, and a service account they created keeps running with owner rights. The auditor will diff your HR reality (even if "HR" is a spreadsheet) against every system's member list, and every orphan account is an exception in the report.

The fix is a lightweight but real workflow: a Linear ticket (or issue template) for each onboarding and offboarding, listing every system touched, checked off with dates. With 3 hires a year that is minutes of work — but it must exist *before* access is granted, and offboarding must be same-day (or within a documented SLA, typically 24 hours) for it to test cleanly.

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Controls access credential creation** — new internal and external users are registered and credentials are issued only after authorization from the system owner or an authorized approver.
- **Removes access to protected assets when appropriate** — credentials are removed or disabled timely when the user is terminated or access is no longer required.
- **Reviews appropriateness of access credentials** — the population of credentials is periodically reviewed to identify unnecessary or inappropriate accounts (dovetails with CC6.3 access reviews).

## What the auditor will ask for

- Complete list of hires and terminations (including contractors) during the audit period, from payroll/HR records.
- Onboarding tickets/approvals for a sample (or all, at this size) of new users, showing authorization dated before account creation.
- Offboarding tickets for all leavers, showing each system deprovisioned with dates; auditor computes termination-date → deactivation-date lag.
- Current user lists exported from each in-scope system (Workspace, GitHub org incl. outside collaborators, GCP IAM, Linear, Slack) to reconcile against the HR roster.
- Evidence that shared/generic accounts either don't exist or are documented with an owner and MFA.
- The documented onboarding/offboarding procedure or checklist template.
- For a sampled leaver: proof their tokens/keys were revoked (GitHub PATs, SA keys they held) not just their SSO account suspended.

## How a tiny AI-first startup satisfies it

- **One ticket per lifecycle event.** A Linear template `Onboarding — <name>` and `Offboarding — <name>` with a checklist of every system: Workspace, GitHub org, GCP/AWS IAM, Linear, Slack, AI provider consoles (Anthropic/OpenAI), password manager, MDM. Founder approval recorded on the ticket before any account is created.
- **SSO-first provisioning.** Grant via Google Workspace wherever possible so offboarding is dominated by one suspend action. Track the non-SSO stragglers (GitHub personal accounts, cloud IAM `user:` bindings) explicitly on the checklist.
- **Same-day offboarding.** On departure: suspend Workspace account, remove from GitHub org, remove `user:` IAM bindings, deactivate Linear/Slack, rotate any shared secrets the person could have seen, revoke org-authorized PATs/OAuth grants, reclaim/wipe laptop. Target ≤24h, record actual timestamps on the ticket.
- **External users are users.** Contractors and GitHub outside collaborators get the same ticket, plus an access end date; the shadow tool alerts on expired end dates.
- **No shared accounts** — or, where unavoidable (a vendor portal without seats), the credential lives in the shared password manager vault, MFA enabled, and rotation on any departure is a checklist item.
- **Quarterly reconciliation** (shared with CC6.3): diff HR roster vs. every system's member export; any account without a matching active human becomes an offboarding ticket immediately.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| GitHub org members all map to active roster | GitHub + roster file | `gh api /orgs/{org}/members` diffed against `roster.yaml` (tool-maintained active-people file) |
| Outside collaborators are known and unexpired | GitHub | `gh api /orgs/{org}/outside_collaborators` diffed against roster entries with `type: contractor` and `end_date` |
| No pending org invitations older than 7 days | GitHub | `gh api /orgs/{org}/invitations` → `created_at` age |
| Suspended/removed Workspace users absent from GCP IAM | GCP + Workspace | `gcloud projects get-iam-policy` `user:` members vs. Admin SDK Directory API users (`suspended != true`) |
| Workspace has no suspended-but-not-deleted accounts > 30 days | Google Workspace | Admin SDK Directory API `users.list` → `suspended` + last-modified age |
| Offboarding ticket exists for each roster departure | Linear + roster | Linear API: search issues with label `offboarding` matching departed names; flag departures without one |
| Offboarding completed within SLA | Linear | Linear API: offboarding issue `completedAt` minus roster `end_date` ≤ 24h |
| Onboarding ticket predates account creation | GitHub/Linear | `gh api /orgs/{org}/members` join events (audit log, requires Enterprise) vs. Linear onboarding issue `createdAt` — MANUAL if no audit-log API |
| Leaver's PATs/OAuth grants revoked | GitHub | MANUAL — fine-grained PAT listing for other users is not exposed; verify via org credential authorizations (`gh api /orgs/{org}/credential-authorizations`, SAML orgs only) |
| Shared-account inventory reviewed | Password manager | MANUAL — export vault item list, confirm owners documented |

## Evidence artifacts

- `roster.yaml` (or HR export) — authoritative list of people, roles, start/end dates; version-controlled so history is auditable.
- Linear onboarding/offboarding issues — exported quarterly to `evidence/lifecycle/<quarter>/` as JSON/PDF with checklist states and timestamps.
- `evidence/github/members-<date>.json`, `outside-collaborators-<date>.json` — periodic member exports.
- `evidence/workspace/users-<date>.csv` — Directory export including suspended status.
- `evidence/gcp/iam-policy-<project>-<date>.json` — to prove leavers' bindings removed.
- `policies/onboarding-offboarding.md` — the checklist template and SLA, version-controlled.
- Reconciliation reports: `evidence/access-reviews/<quarter>/roster-diff.md` produced by the shadow tool.
