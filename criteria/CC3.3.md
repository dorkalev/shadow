---
id: CC3.3
family: CC3 — Risk Assessment
category: Security (Common Criteria)
coso: COSO Principle 8
title: Fraud Risk Consideration
weight: 2
automatable: partial
nature: document
---

# CC3.3 — Fraud Risk Consideration

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 8: The entity considers the potential for fraud in assessing risks to the achievement of objectives.

## What it means
Your risk assessment must explicitly consider fraud — not just outside attackers, but the possibility that people inside or connected to the company act dishonestly: an employee exfiltrating customer data, a founder overriding controls, a contractor abusing production access, fraudulent transactions against the product itself, or misappropriation of assets (including data — in SOC 2 terms, data theft is asset misappropriation).

At a 3-person startup this feels awkward — "we'd notice if the other founder went rogue" — but the criterion doesn't require distrust, it requires *consideration*. The classic COSO fraud triangle framing (incentives/pressures, opportunities, attitudes/rationalizations) applied honestly to a tiny company yields real findings: everyone has admin everywhere (opportunity), there's no independent review of the person who controls billing and infrastructure (opportunity + management override), and financial pressure on an early-stage company is structural (incentive).

The practical output is a fraud section in the risk register plus the compensating controls tiny teams actually have: audit logs that no single person can silently erase, PR review on all changes, alerts on unusual admin actions, and vendor-side immutability (GitHub audit log, GCP Cloud Audit Logs). The auditor also expects fraud risk to include external fraud vectors relevant to you: account takeover of your customers, payment fraud, and social engineering of your team.

## Points of focus (2022 revision, summarized)
Guidance, not requirements:
- Considers various types of fraud — fraudulent reporting, possible loss of assets (including data), and corruption.
- Assesses incentives and pressures that could motivate fraud.
- Assesses opportunities — weak controls, broad access, ability to act unilaterally.
- Assesses attitudes and rationalizations that could justify fraudulent actions.
- Considers how management might override controls, and threats from vendors and business partners.
- (TSC-specific) Considers risks related to unauthorized use, alteration, or theft of data and IT resources by personnel with access.

## What the auditor will ask for
- The fraud-risk section of the risk register (or a standalone fraud risk assessment) with the triangle factors considered.
- Evidence it was discussed during the period (risk assessment minutes with a fraud agenda item).
- Controls mapped against management-override risk: PR review requirements, audit log immutability, alerting on admin actions.
- The code of conduct / acceptable use policy employees acknowledged (ties to CC1.1 but requested here).
- Access review evidence showing least privilege — the main anti-opportunity control at small scale.
- Any actual fraud incidents or suspicions during the period and how they were handled.

## How a tiny AI-first startup satisfies it
- Add a `## Fraud risks` section to `risks.md` with 4–8 entries: insider data exfiltration, founder/management override of controls, contractor abuse of prod access, payment/transaction fraud against the product, social engineering (phishing, fake-CEO), and misuse of AI agents with production credentials. Score and treat like any other risk.
- Make the fraud discussion an explicit agenda line in the annual founder-led risk assessment; one paragraph of minutes ("considered fraud triangle; key exposure is unilateral admin access; mitigations: X, Y") is sufficient and honest.
- Compensating controls that work at n<10 and are verifiable: GitHub branch protection with required review (no one merges their own unreviewed code to main), org-level audit logs retained by the vendor (GitHub audit log, Google Workspace audit, GCP Cloud Audit Logs — a rogue admin can't rewrite them), billing/spend alerts in GCP/AWS, and a second founder on the bank/payment-processor account.
- Code of conduct in the policies repo (`policies/code-of-conduct.md`) acknowledged by each person via PR approval or a signed acknowledgment file per hire.
- Where two-person segregation is impossible, document it as an accepted risk with compensating detective controls (log review, alerts) — auditors accept this when it's written down.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| `risks.md` contains a fraud-risk section with entries | GitHub | Fetch file, grep for fraud section heading and count rows |
| Branch protection requires PR review on default branch (anti-override) | GitHub | `gh api repos/{org}/{repo}/branches/main/protection` — `required_pull_request_reviews` present |
| No admin bypass of branch protection (`enforce_admins`) | GitHub | Same API — `enforce_admins.enabled == true` |
| GCP Cloud Audit Logs (Admin Activity) enabled/unmodified | GCP | `gcloud logging sinks list` / audit config via `gcloud projects get-iam-policy`; Admin Activity logs are always-on — verify no exclusion filters |
| Workspace admin audit events retrievable | Google Workspace | Admin SDK Reports API `activities.list(applicationName=admin)` returns events |
| Billing/budget alerts configured | GCP | `gcloud billing budgets list` non-empty |
| Code of conduct exists and each member has acknowledgment | GitHub | File existence + acknowledgment files/PR approvals vs. org member list |
| Fraud considered in annual assessment minutes | GitHub | Grep minutes file for fraud agenda item — flag if absent (else MANUAL) |
| Quality of fraud analysis (triangle actually applied) | — | MANUAL |

## Evidence artifacts
- Fraud section of `risks.md` with git history showing in-period review.
- `evidence/risk-assessment-YYYY/minutes.md` containing the fraud discussion paragraph.
- Branch protection settings export (`gh api ... /protection` JSON) in `evidence/github/`, archived to `compliance-archives`.
- GitHub org audit log export (JSON) and Workspace admin audit export for the period, in `evidence/audit-logs/`.
- `policies/code-of-conduct.md` plus per-person acknowledgments (`evidence/acknowledgments/<name>.md` or PR approvals).
- GCP budget-alert configuration export in `evidence/gcp/`.
