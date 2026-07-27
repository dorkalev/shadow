---
id: CC3.4
family: CC3 — Risk Assessment
category: Security (Common Criteria)
coso: COSO Principle 9
title: Assessing Significant Change
weight: 2
automatable: partial
nature: document
---

# CC3.4 — Assessing Significant Change

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 9: The entity identifies and assesses changes that could significantly impact the system of internal control.

## What it means
Risk assessment can't be a once-a-year snapshot if the company changes weekly — and startups change weekly. CC3.4 requires a mechanism to notice when something significant changes and to re-assess risk when it does. "Significant" here means changes that could break your control environment: new cloud provider or region, a major new vendor or model provider, first customer with regulated data, a new hire or departure (especially anyone with admin access), a pivot in the product, adopting a new AI agent framework with production access, new regulation touching you, or leadership change.

The distinction from CC3.2 is trigger versus cadence: CC3.2 is the periodic assessment; CC3.4 is the standing question "did anything just happen that invalidates our last assessment?" For a tiny startup the honest implementation is a change-triggered addendum process: certain event types automatically require a risk-register touch, and the quarterly self-review includes a "what changed?" section.

Auditors test this by picking real changes from the period (they can see your commit history, vendor list diffs, and team changes) and asking "show me where you assessed this." If you onboarded a new sub-processor in March and the risk register was untouched until December, that's an exception.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Assesses changes in the external environment — regulatory, economic, and physical/threat environment shifts.
- Assesses changes in the business model — new lines of business, rapid growth, new technologies, new geographies.
- Assesses changes in leadership and the attitudes/philosophy they bring to internal control.
- (TSC-specific) Considers changes in systems, technology, and vendor/business-partner relationships, and changes in threats and vulnerabilities affecting the system.

## What the auditor will ask for
- The documented trigger list: which change types require a risk re-assessment.
- Examples from the period: significant changes that occurred and the corresponding risk-register updates or minutes (they will pick their own examples too).
- Quarterly review notes containing a "changes since last review" section.
- Vendor onboarding records showing risk consideration before adoption (links to CC9.2).
- Change history of the risk register correlated with major events (new infra, new hires, incidents).
- Post-incident risk updates, if any incidents occurred.

## How a tiny AI-first startup satisfies it
- Add a short `## Change triggers` section to `risks.md` or `policies/risk-management.md` listing events that force a register update: new vendor/sub-processor, new cloud service or project, personnel join/leave, new data category processed, security incident, new AI model/agent with credential access, material regulation change.
- Wire triggers into workflows you already have: a Linear template checklist item "risk register updated?" on vendor-onboarding, new-hire, and incident tickets. The ticket becomes the evidence.
- Quarterly founder self-review (30 minutes) with three fixed questions — what changed in the stack, the team, and the vendor list — recorded as `evidence/risk-reviews/YYYY-QN.md` via PR. Copy team/vendor diffs from the shadow tool's own reports.
- When a significant change lands, commit a dated addendum row to `risks.md` in the same week; git history makes timeliness provable.
- For AI-first teams specifically: treat "gave an agent a new tool/credential" and "switched primary model provider" as first-class significant changes — auditors increasingly ask.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Change-trigger list documented | GitHub | Fetch `risks.md`/`risk-management.md`, grep for change-triggers section |
| Quarterly review file exists for each quarter in period | GitHub | File existence `evidence/risk-reviews/YYYY-QN.md` per quarter |
| New GitHub org member in period → register commit within 14 days | GitHub | `gh api orgs/{org}/members` + audit log `org.add_member` events; correlate with `risks.md` commit dates |
| Departure (member removed) → register/access-review touch within 14 days | GitHub | Audit log `org.remove_member` correlated with commits |
| New GCP service/project enabled → register touch within 30 days | GCP | `gcloud services list --enabled` / `gcloud projects list` diffed against prior snapshot; correlate with commits |
| Vendor list change → risk row added | GitHub | Diff `assets.md`/`vendors.md` history vs. `risks.md` history |
| Incident tickets include risk-update checklist item completed | Linear | Query issues labeled `incident`; verify checklist/linked register commit |
| Judgment that a change was "significant" and adequately assessed | — | MANUAL |

## Evidence artifacts
- Change-triggers section of `risks.md` (or `policies/risk-management.md`) with git history.
- `evidence/risk-reviews/YYYY-QN.md` — quarterly what-changed reviews, one per quarter, PR-merged.
- Correlated timeline export (shadow tool output): significant events vs. register commits, archived to `compliance-archives`.
- Linear ticket exports for vendor-onboarding/new-hire/incident tickets showing the risk checklist item, in `evidence/linear/`.
- Snapshots of vendor list and enabled-services list at each quarter end (`evidence/gcp/services-YYYY-QN.json`), archived to `compliance-archives`.
