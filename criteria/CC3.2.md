---
id: CC3.2
family: CC3 — Risk Assessment
category: Security (Common Criteria)
coso: COSO Principle 7
title: Risk Identification and Analysis
weight: 3
automatable: partial
nature: document
---

# CC3.2 — Risk Identification and Analysis

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 7: The entity identifies risks to the achievement of its objectives across the entity and analyzes risks as a basis for determining how the risks should be managed.

## What it means
This is the core risk-assessment criterion and one of the most-cited exceptions in small-company SOC 2 reports. You must (1) actually identify the risks that could stop you meeting your objectives and commitments — external (attackers, vendor failure, cloud outage, regulation) and internal (key-person loss, bad deploy, leaked credential, AI-generated code introducing vulnerabilities), (2) analyze each for likelihood and impact, and (3) decide a response: mitigate, accept, transfer, or avoid.

The auditor is not grading your risk taste; they are checking the *process* exists, is documented, covers the system in scope, and was performed within the audit period. "The founders think about risk all the time" is true and worthless as evidence. A dated risk register plus minutes of a real discussion is what passes.

For an AI-first startup, expect the auditor to probe whether the register reflects your actual stack: LLM API key exposure, prompt-injection against agents with tool access, over-permissive CI tokens, reliance on a single model vendor, and AI-assisted code review gaps. A generic template register that never mentions these looks like it wasn't really performed.

Risk identification must include threats to and vulnerabilities of system components — so the register should be grounded in an asset list (repos, cloud projects, SaaS tools, data stores), not free-floating worries.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Risk identification covers entity, subsidiary, division, operating unit, and functional levels (for a tiny startup: the whole company plus each major system).
- Analyzes internal and external factors — economic, regulatory, technological, personnel, and vendor-related.
- Involves appropriate levels of management in risk assessment.
- Estimates the significance of identified risks (likelihood × impact).
- Determines how to respond to risks — accept, avoid, reduce, or share.
- (TSC-specific) Identifies threats to the system and vulnerabilities of system components, including those arising from vendors and business partners, and considers risks to data.

## What the auditor will ask for
- The current risk register (all fields: risk, objective affected, likelihood, impact, score, owner, response, status).
- Evidence the risk assessment was performed/refreshed during the audit period (minutes, PR history, tickets).
- The documented risk assessment methodology (how scores are assigned, what thresholds trigger treatment).
- The asset/system inventory used as the basis for threat identification.
- Vendor list with risk ratings (feeds CC9.2 but requested here too).
- Linkage from high risks to mitigating controls or remediation tickets (Linear).
- Who participated — names and roles, showing management involvement.

## How a tiny AI-first startup satisfies it
- Maintain `risks.md` in the policies repo: a markdown table per risk — ID, description, objective/commitment affected, likelihood (1–3), impact (1–3), score, response (mitigate/accept/transfer/avoid), owner (a named founder), linked control or Linear ticket, last-reviewed date. 15–30 rows is normal and credible for a 5-person company; 4 rows is not.
- Document the methodology in a short preamble in the same file (scoring scale, treatment threshold, review cadence). No separate 20-page framework needed.
- Hold an annual founder-led risk assessment meeting (plus a lightweight quarterly self-review); capture minutes with date, attendees, risks added/re-scored/retired, and decisions. Store as `evidence/risk-assessment-YYYY/minutes.md`, merged via reviewed PR.
- Keep `assets.md` (systems, data classes, vendors) adjacent so risks trace to real components. Cover AI-specific risks explicitly: model-vendor dependence, agent tool permissions, secrets in prompts/context, unreviewed AI-generated code.
- Every risk with response "mitigate" and score above threshold gets a Linear ticket (label `risk`) — this gives the shadow tool something to verify and the auditor a closed loop.
- Accepted risks get a one-line rationale and a founder's name. Auditors respect documented acceptance; they flag silence.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `risks.md` exists and parses (required columns present) | GitHub | Fetch file via `gh api`, validate table schema |
| Register reviewed within 12 months | GitHub | `gh api repos/{org}/policies/commits?path=risks.md` — latest commit < 365 days |
| Quarterly touch (any commit or dated review note per quarter) | GitHub | Commit history on `risks.md` bucketed by quarter |
| Every high-score mitigated risk links to a Linear ticket | GitHub + Linear | Parse ticket IDs from register; `linear` API confirms ticket exists and state |
| No risk row stale > 12 months (last-reviewed date column) | GitHub | Parse dates from table, flag stale rows |
| Asset inventory file exists and was updated in period | GitHub | File existence + last-commit check on `assets.md` |
| Annual minutes file for current period exists | GitHub | `evidence/risk-assessment-<year>/minutes.md` existence |
| Register content is genuinely entity-specific (mentions actual stack) | — | MANUAL — human judgment on quality |
| Management participation in assessment | — | MANUAL — attendee list vs. org roster |

## Evidence artifacts
- `risks.md` — the living risk register (policies repo), plus its full git history.
- `assets.md` — system/data/vendor inventory the register is grounded in.
- `evidence/risk-assessment-YYYY/minutes.md` — annual meeting minutes; quarterly review notes in the same directory.
- Export of Linear tickets labeled `risk` (JSON via API) showing remediation follow-through, archived to `compliance-archives` branch.
- PRs approving register changes (reviewer ≠ author where headcount allows).
- Point-in-time register snapshots at period start/end, archived to `compliance-archives` for the Type II window.
