---
id: CC9.1
family: CC9 — Risk Mitigation
category: Security (Common Criteria)
coso: null
title: Business Disruption Risk Mitigation
weight: 2
automatable: partial
nature: document
---

# CC9.1 — Business Disruption Risk Mitigation

## Criterion (AICPA TSP Section 100, verbatim)
> The entity identifies, selects, and develops risk mitigation activities for risks arising from potential business disruptions.

## What it means

CC9.1 asks: what could knock your business over — cloud region outage, key-person loss, critical vendor failure, ransomware, funding gap affecting security spend — and what have you deliberately decided to do about each? The verbs matter: *identify* the disruption risks, *select* a treatment (mitigate, accept, transfer via insurance, avoid), and *develop* the actual mitigation activities. Documented risk *acceptance* is a perfectly valid outcome; undocumented ignorance is not.

For a 1–10 person startup the honest disruption register is short: single cloud region, single production database, one or two people who hold all context, dependence on GitHub/GCP/Anthropic APIs, and a laptop-borne credential compromise. Mitigations are correspondingly simple — backups and IaC (CC7.5), documented runbooks so knowledge isn't only in heads, cyber insurance if customers require it, and a one-page BC/DR policy stating recovery approach and accepted risks like "single-region: accepted until Series A, revisit at 10 engineers."

Auditors test this criterion mostly through documentation and coherence: a risk assessment that includes business-disruption scenarios, a BC/DR policy consistent with reality, and evidence the analysis is refreshed annually. This overlaps heavily with CC3.x (risk assessment) and A1.x (availability); in a security-only report the bar is a considered, current, founder-approved analysis — not a 40-page BCP.

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Considers risk mitigation for business disruptions as part of the entity's risk assessment and treatment activities.
- Develops, implements, and maintains business continuity and disaster recovery plans proportionate to the risks identified.
- Considers the use of insurance to offset the financial impact of loss events, where appropriate.
- Periodically evaluates and updates mitigation activities as the business and its dependencies change.

## What the auditor will ask for
- The risk register including business-disruption scenarios, each with likelihood/impact rating and selected treatment (mitigate/accept/transfer/avoid).
- BC/DR policy and any continuity runbooks, with approval and last-review dates.
- Evidence of the annual risk review: who participated, when, what changed.
- Insurance policy documentation (cyber/E&O) if transfer is a claimed treatment, or the documented decision not to carry it.
- Mapping from identified risks to implemented mitigations (e.g., "region outage → backups + IaC redeploy runbook, restore test on <date>").
- Evidence of mitigation activities operating: backup/restore evidence (shared with CC7.5), runbook existence, key-person cross-training notes.

## How a tiny AI-first startup satisfies it
- **Risk register in-repo** (`risk-register/register.yaml` or `.md`): 8–15 entries, a disruption section covering at minimum: cloud region/provider outage, data loss/corruption, key-person unavailability, critical vendor failure (GitHub, GCP, Anthropic/OpenAI, Stripe), credential compromise/ransomware, and office-independence (trivial for remote teams — say so). Each entry: likelihood, impact, treatment, owner, linked mitigation.
- **BC/DR policy** (`policies/bcdr.md`, shared with CC7.5): recovery strategy is "restore from backups + redeploy from repo via CI," honest RTO/RPO, and explicit accepted risks with revisit triggers.
- **Key-person mitigation**: runbooks in-repo for deploy, restore, incident response, and vendor account access; break-glass access documented (org owner recovery, password-manager emergency access). For a 2-person company, this is your most credible real mitigation — auditors know it.
- **Treatment by SDLC**: mitigation work items are Linear tickets shipped through the CC8.1 pipeline, so "developed" mitigations have merge + archive evidence rather than assertions.
- **Annual review ritual**: a recurring Linear ticket "annual risk & BC/DR review" — founders walk the register, update ratings, re-approve the policy (PR to the policy file = the approval record via review + archive).
- **Insurance**: if carried, policy PDF in the evidence store with renewal date tracked; if not, an accepted-risk register entry with rationale.
- The compliance shadow enforces freshness: register and policy review dates, restore-test recency, and open mitigation tickets going stale all trigger findings.

## Automated shadow checks

> Datastore commands are per-stack: Cloud SQL shown. On Firestore stacks (the blessed `provision/gcp`), the equivalents are `gcloud firestore databases describe` (PITR, delete protection, state) and `gcloud firestore backups list` / `backups schedules list` (schedule present, recent snapshots).

| Check | Source | Method |
|---|---|---|
| Risk register exists with disruption-scenario entries | repo | file-existence + schema/section parse of `risk-register/` |
| Every register entry has treatment, owner, and date | repo | YAML/frontmatter field validation |
| Register reviewed within 12 months | repo | git log last-meaningful-change date on register file |
| BC/DR policy exists, approved, reviewed <12mo | repo | file-existence + frontmatter + merged-PR approval record |
| Annual risk review ticket completed in period | Linear | recurring ticket completion query |
| Mitigation tickets linked from register are closed or in-progress (not stale >90d) | Linear | Linear API status query on linked ticket IDs |
| Runbooks exist for deploy/restore/incident/access | repo | file-existence in `runbooks/` |
| Backup + restore-test evidence current (shared with CC7.5) | GCP+evidence | `gcloud sql instances describe`; latest `evidence/restore-tests/*` date |
| Register/policy changes went through PR flow | GitHub | path-filtered merged-PR + archive record check |
| Insurance policy current | evidence | file-existence + expiry field in `evidence/insurance/` — MANUAL to verify coverage adequacy |
| Risk ratings and treatment choices are sensible | — | MANUAL — founder/auditor judgment |

## Evidence artifacts
- `risk-register/register.yaml` — the register, git history showing annual updates.
- `policies/bcdr.md` — BC/DR policy with approval trail (PR review + `compliance-archives` record).
- `runbooks/*.md` — deploy, restore, incident, break-glass access runbooks.
- Linear: annual review ticket, mitigation tickets linked from register entries.
- `evidence/insurance/` — policy documents and renewal tracking, if applicable.
- `evidence/restore-tests/` — shared with CC7.5, demonstrating the flagship disruption mitigation actually works.
