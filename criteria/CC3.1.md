---
id: CC3.1
family: CC3 — Risk Assessment
category: Security (Common Criteria)
coso: COSO Principle 6
title: Objectives Specified with Clarity
weight: 2
automatable: partial
nature: document
---

# CC3.1 — Objectives Specified with Clarity

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 6: The entity specifies objectives with sufficient clarity to enable the identification and assessment of risks relating to objectives.

## What it means
You cannot assess risk against nothing. Before you can say "X threatens us," you have to be able to say what "us succeeding" means. CC3.1 requires the company to write down its objectives — operational (ship the product, keep it up), reporting (accurate metrics and financials), and compliance (contractual security commitments, applicable laws like GDPR/CCPA) — clearly enough that a risk assessment can be anchored to them.

For a SOC 2 audit specifically, the objectives that matter most are your **service commitments and system requirements**: what you promised customers (in your ToS, MSA, security page, DPA) about security, availability, and confidentiality, and what the system must therefore do. The auditor will trace your risk register back to these objectives; if the objectives were never articulated, the risk assessment fails at the root.

For a 1–10 person startup this is not a strategy offsite. It is a one-to-two page document: what the product does, what you commit to customers, what regulations apply, and what your risk tolerance is (e.g., "we accept single-region deployment risk; we do not accept unencrypted customer data at rest"). Founders already know this — the criterion just requires it to exist on paper and be the stated basis for the risk register.

## Points of focus (2022 revision, summarized)
These are AICPA/COSO guidance to inform judgment, not requirements in themselves:
- Operations objectives reflect management's choices about structure, industry, and performance, and include tolerances for risk.
- Objectives form a basis for committing resources and include operational and financial performance goals.
- External reporting objectives comply with applicable standards and reflect entity activities at an appropriate level of precision.
- Compliance objectives reflect the laws and regulations applicable to the entity.
- (TSC-specific) The entity establishes sub-objectives tied to its service commitments and system requirements — the security, availability, and confidentiality promises made to customers.

## What the auditor will ask for
- The document(s) where entity objectives and service commitments are defined (business plan excerpt, objectives.md, security page, ToS/MSA security clauses).
- The system description (Section 3 of the report) showing principal service commitments and system requirements.
- Evidence the risk assessment references these objectives (risk register with an "objective affected" column or intro section).
- Customer-facing commitments: standard contract/DPA template, trust/security page URL and change history.
- A statement of risk appetite/tolerance, even if brief, approved by management.
- Evidence of periodic review — a commit, meeting minutes, or Linear ticket showing objectives were reaffirmed within the audit period.

## How a tiny AI-first startup satisfies it
- Keep `policies/objectives-and-commitments.md` in a versioned `policies/` GitHub repo: 1–2 pages covering (a) business objectives, (b) service commitments to customers (uptime target, encryption, breach notification window, data deletion), (c) applicable laws/regs, (d) explicit risk tolerance statements. Git history is your approval and review trail.
- Make `risks.md` (the risk register) open with a line: "Risks are assessed against the objectives in objectives-and-commitments.md" — this is the traceability the auditor wants.
- Reaffirm annually in the founder-led risk assessment meeting; record minutes as `evidence/risk-assessment-YYYY/minutes.md` with attendees and date, merged via PR so approval is provable.
- Keep the public security page and contract security exhibit consistent with the internal doc — the auditor will diff your promises against your controls.
- Acceptable tiny-startup equivalent of "board-approved objectives": a PR approving the doc, reviewed by the other founder (or sole-founder self-attestation with a dated commit plus advisor acknowledgment if available).

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| Objectives/commitments doc exists in policies repo | GitHub | `gh api repos/{org}/policies/contents/objectives-and-commitments.md` returns 200 |
| Doc reviewed within last 12 months | GitHub | `gh api repos/{org}/policies/commits?path=objectives-and-commitments.md` — latest commit date < 365 days |
| Risk register references objectives doc | GitHub | Fetch `risks.md` content, grep for objectives doc filename/link |
| Approval via PR review (not direct push) | GitHub | `gh api` commit → associated PR → reviews non-empty |
| Annual risk-assessment minutes exist for current period | GitHub | File existence check `evidence/risk-assessment-<year>/minutes.md` |
| Public security page matches internal commitments | Web + repo | MANUAL — human diff of security page vs. objectives doc |
| Contract/DPA templates align with stated commitments | Legal docs | MANUAL |

## Evidence artifacts
- `policies/objectives-and-commitments.md` — the objectives and service-commitments document (versioned in policies repo).
- `risks.md` header section linking risks to objectives.
- `evidence/risk-assessment-YYYY/minutes.md` — annual meeting minutes naming objectives review as an agenda item.
- PR link showing review/approval of the objectives doc within the period (archived to `compliance-archives` branch as JSON export of PR + reviews).
- Snapshot (PDF or HTML capture) of the public security/trust page, stored in `evidence/commitments/`.
- Signed customer contract template excerpt showing security commitments (redacted, in `evidence/commitments/`).
