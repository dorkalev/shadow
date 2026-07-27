---
id: CC9.2
family: CC9 — Risk Mitigation
category: Security (Common Criteria)
coso: null
title: Vendor and Business Partner Risk Management
weight: 3
automatable: partial
nature: document
---

# CC9.2 — Vendor and Business Partner Risk Management

## Criterion (AICPA TSP Section 100, verbatim)
> The entity assesses and manages risks associated with vendors and business partners.

## What it means

Your security posture is mostly other people's computers: GCP/AWS runs your infrastructure, GitHub holds your source, Google Workspace holds identity and email, Anthropic/OpenAI process your prompts (possibly containing customer data), plus Linear, Sentry, Stripe, and a tail of SaaS. CC9.2 requires you to know who these vendors are, what data and access each has, how risky each is, and to manage that risk over the relationship lifecycle — onboarding assessment, contractual terms (DPAs, security addenda), periodic review of their SOC 2 / ISO / trust-center attestations, monitoring for their incidents, and offboarding (data deletion, access revocation) when you drop them.

For an AI-first startup this criterion has real teeth, because auditors increasingly probe LLM vendors specifically: is customer data sent to model APIs? Under what retention terms — is a zero-data-retention or no-training agreement in place? Is there a DPA? Are subprocessors disclosed to *your* customers? Your vendor register should treat model providers as high-criticality data processors, not dev tools.

The good news: a tiny startup's vendor list is 10–25 entries, and the entire control can be a YAML register in the repo with an annual review cycle. What auditors reject is the absence of any tiering or review — "we trust GCP because everyone does" needs to be written down as "GCP: critical tier, SOC 2 Type II reviewed on <date>, report on file."

## Points of focus (2022 revision, summarized)
*Points of focus are illustrative guidance, not requirements. Summarized:*
- Establishes requirements for vendor and business partner engagements, including security and confidentiality commitments.
- Assesses vendor and business partner risks — criticality, data access, and the vendor's own control environment — before and during the relationship.
- Assigns responsibility and accountability for vendor risk management and specifies terms in written agreements (e.g., DPAs, security terms, notification obligations).
- Obtains and reviews assurance over vendors' controls (e.g., SOC reports, certifications) commensurate with risk; implements compensating procedures where assurance is unavailable.
- Assesses vendor performance and compliance periodically; addresses issues identified.
- Implements procedures for terminating relationships, including data return/destruction and access removal.

## What the auditor will ask for
- The vendor register: all vendors with service description, data shared/access granted, criticality tier, owner, and review status.
- Vendor risk assessment procedure — how new vendors are evaluated and tiered before adoption.
- For critical vendors: the reviewed SOC 2 report or equivalent (auditors check the report date falls in/near your period and that someone documented reading it, including subservice-organization and CUEC sections).
- Signed agreements for data-handling vendors: DPAs, and for LLM providers, the data-retention/no-training terms relied upon.
- Evidence of periodic (usually annual) vendor reviews performed during the period.
- Offboarding evidence for any vendor dropped in the period: access revoked, data deleted/exported.
- Evidence of monitoring for vendor security incidents affecting you, and any responses.

## How a tiny AI-first startup satisfies it
- **Vendor register in-repo** (`vendors/register.yaml`): one entry per vendor with fields — name, purpose, data categories shared (source code, customer PII, prompts, payment data, none), access direction, criticality (critical/high/standard), attestation type + last-reviewed date + link, DPA status, owner, next review date. Changes to the register go through the CC8.1 PR flow, so vendor onboarding/offboarding decisions carry review + archive evidence automatically.
- **Tiering rule of thumb**: critical = outage or breach directly breaches customer commitments (GCP, GitHub, Google Workspace, Anthropic/OpenAI, Stripe); high = holds sensitive data or broad tokens (error trackers, the tracker, review-bot apps — note they read your source); standard = the rest.
- **LLM vendors get first-class treatment**: register entries record which API tier/agreement applies (e.g., zero-data-retention or enterprise no-training terms), whether customer data can appear in prompts, and the customer-facing subprocessor disclosure. This preempts the audit's sharpest questions.
- **Annual review ritual**: recurring Linear ticket per tier — for critical vendors, pull the current SOC 2 report from the vendor trust center, note report period, opinion, relevant CUECs you must operate (e.g., "enable MFA" — cross-link to your own control), and any exceptions; commit a one-paragraph review memo per vendor to `vendors/reviews/`.
- **Onboarding gate**: a lightweight checklist (data classes? DPA needed? SSO/MFA available? attestation exists?) as the Linear ticket template for "add vendor"; no production credentials issued before the register entry merges.
- **Offboarding**: ticket template covering token/OAuth revocation (checkable in Workspace and GitHub app lists), data deletion request, and register status flip to `offboarded`.
- **Incident monitoring**: vendor status pages/security advisories feed the CC7.2 anomaly channel; a vendor breach affecting you becomes a security event (CC7.3).

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Vendor register exists and parses against schema | repo | file-existence + YAML schema validation of `vendors/register.yaml` |
| Every entry has criticality, data classes, owner, review date | repo | field-completeness validation |
| Critical/high vendors reviewed within 12 months | repo | `last_reviewed` field vs today; review memo file exists in `vendors/reviews/` |
| DPA flagged vendors have DPA status = signed | repo | field cross-check (data classes include PII → DPA required) |
| Register covers actually-connected services (no shadow vendors) | GitHub+GCP+Workspace | `gh api /orgs/{org}/installations` (GitHub apps), `gcloud services list`, Workspace token report — diff against register entries |
| Third-party GitHub app permissions match register access notes | GitHub | `gh api /orgs/{org}/installations` permission scopes |
| Offboarded vendors have no live access | GitHub+Workspace | app/OAuth-grant absence check for `offboarded` entries |
| Register changes went through PR flow with archive record | GitHub | path-filtered merged PRs on `vendors/` + `compliance-archives` lookup |
| Annual review tickets completed | Linear | recurring ticket completion query per tier |
| SOC 2 report content actually reviewed (CUECs, exceptions) | evidence | MANUAL — human reads the report; shadow only verifies memo exists |
| Contract/DPA terms adequacy | — | MANUAL — legal judgment |

## Evidence artifacts
- `vendors/register.yaml` — the register, with git history showing lifecycle changes through reviewed PRs.
- `vendors/reviews/YYYY-<vendor>.md` — per-vendor annual review memos (report period, opinion, CUECs, decision).
- `evidence/vendor-attestations/` — vendor SOC 2 reports/certificates on file (or trust-center access notes where reports are portal-only).
- DPAs and LLM data-retention agreements (signed copies or order-form references) in `evidence/vendor-contracts/`.
- Linear: vendor onboarding/offboarding/review tickets with checklists.
- `evidence/vendor-access/YYYY-MM-DD.json` — shadow export of connected GitHub apps, enabled GCP services, and Workspace OAuth grants used for the shadow-vendor diff.
- `compliance-archives` branch — archive records for register-change PRs (vendor decisions as controlled changes).
