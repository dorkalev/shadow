---
id: CC5.1
family: CC5 — Control Activities
category: Security (Common Criteria)
coso: COSO Principle 10
title: Risk-Mitigating Control Selection
weight: 2
automatable: partial
nature: document
---

# CC5.1 — Risk-Mitigating Control Selection

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 10: The entity selects and develops control activities that contribute to the mitigation of risks to the achievement of objectives to acceptable levels.

## What it means
CC5.1 is the bridge between the risk register (CC3.2) and everything else in your report: for each significant risk, you must have deliberately chosen controls that bring it down to an acceptable level. The auditor reads this as a mapping exercise — risk → control(s) → evidence — and the failure mode is a register and a control list that don't reference each other, which suggests the controls were copied from a template rather than selected against your actual risks.

"Selects and develops" also implies fit: controls should match your environment and size. A 4-person startup claiming a Change Advisory Board is less credible (and less passable in testing) than one claiming "all changes to main require one PR approval and passing CI, enforced by branch protection." Auditors explicitly accept that entity-specific factors — size, nature of operations — shape the control set.

The hard point for tiny teams is segregation of duties: with three people, the person writing code often also deploys and administers the cloud. COSO anticipates this — where segregation isn't practicable, management selects alternative (compensating) controls: mandatory peer review, immutable audit logs, alerts on sensitive actions, and periodic access review. You must say this out loud in your control documentation, not hope the auditor doesn't notice.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Control activities integrate with and respond to the risk assessment — they exist to mitigate identified risks.
- Considers entity-specific factors: environment, complexity, nature, and scope of operations.
- Relevant business processes determine which control activities are needed.
- Evaluates a mix of control activity types — preventive and detective, manual and automated.
- Considers at what level (entity-wide vs. process/transaction) activities are applied.
- Addresses segregation of duties; where segregation is not practical, management selects and develops alternative control activities.

## What the auditor will ask for
- The control matrix: list of controls mapped to risks and to TSC criteria (auditors often bring their own matrix and ask you to populate it).
- The risk register with a "mitigating control" column populated for each treated risk.
- Documentation of the segregation-of-duties analysis and the compensating controls chosen where SoD is absent.
- Descriptions of key automated controls and where they're configured (branch protection JSON, IAM policies, CI required checks).
- Evidence a management decision process exists for adding/changing controls (PRs to the policies repo, risk-assessment minutes with control decisions).
- Examples of preventive vs. detective coverage for the top risks.

## How a tiny AI-first startup satisfies it
- Maintain `controls.md` in the policies repo: one table — control ID, description, type (preventive/detective), automated/manual, risk IDs mitigated, TSC criteria mapped, owner, evidence source. 25–50 rows is a realistic full set for a tiny SaaS.
- Enforce the referential integrity mechanically: every `risks.md` row with response "mitigate" must cite control IDs that exist in `controls.md`, and vice versa. The shadow tool lints this on every run — this single check kills the template-copy failure mode.
- Prefer automated preventive controls that platforms enforce for you: GitHub branch protection + required status checks, Workspace MFA/2SV enforcement, GCP IAM least-privilege with no user-managed service-account keys, forced SSO. Each is simultaneously a control and its own evidence.
- Write an explicit `## Segregation of duties` section in `controls.md`: acknowledge that engineers hold multiple duties; list compensating controls (required PR review by a different human — enforced even for admins; vendor-immutable audit logs; alerting on IAM changes; quarterly access review).
- Control changes go through PRs on the policies repo, reviewed by the other founder — that's your "management selects and develops" trail.
- Revisit the mapping in the annual risk assessment: new/re-scored risks get controls assigned in the same meeting; minutes record the decisions.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `controls.md` exists with required columns | GitHub | Fetch and validate table schema |
| Referential integrity: mitigated risks ↔ existing control IDs | GitHub | Parse `risks.md` and `controls.md`, cross-check ID sets both directions |
| Every control row names an evidence source | GitHub | Parse table, flag empty evidence cells |
| Key automated controls actually configured as described (branch protection, MFA, IAM) | GitHub/GCP/Workspace | Run the corresponding technical checks and diff against `controls.md` claims |
| SoD section present with named compensating controls | GitHub | Grep `controls.md` for segregation section |
| Required PR review enforced incl. admins (core compensating control) | GitHub | `gh api .../branches/main/protection` — reviews required, `enforce_admins` true |
| Control matrix reviewed in last 12 months | GitHub | Last commit date on `controls.md` |
| Mix of preventive/detective per top risk | GitHub | Parse types column grouped by risk ID — flag top-weight risks with only one type |
| Whether chosen controls actually reduce risk to acceptable levels | — | MANUAL — auditor/human judgment |

## Evidence artifacts
- `controls.md` — the risk-to-control matrix, versioned in the policies repo (the central CC5.1 artifact).
- `risks.md` with populated mitigating-control column and cross-references.
- Shadow tool lint report showing risk↔control integrity passing across the period (`compliance-archives`).
- Configuration exports proving key automated controls: branch protection JSON, Workspace 2SV report, `gcloud projects get-iam-policy` output — in `evidence/github/`, `evidence/workspace/`, `evidence/gcp/`.
- PRs (with reviews) that added or modified controls during the period.
- Risk-assessment minutes recording control-selection decisions (`evidence/risk-assessment-YYYY/minutes.md`).
