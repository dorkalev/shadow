---
id: CC1.4
family: CC1 — Control Environment
category: Security (Common Criteria)
coso: COSO Principle 4
title: Attract, Develop, Retain Competence
weight: 2
automatable: partial
nature: document
---

# CC1.4 — Attract, Develop, Retain Competence

## Criterion (AICPA TSP Section 100, verbatim)
> COSO Principle 4: The entity demonstrates a commitment to attract, develop, and retain competent individuals in alignment with objectives.

## What it means
The auditor wants evidence that the people running the system are competent and are kept competent: you hire deliberately (job descriptions, some screening), you check backgrounds proportionate to risk, and you train people — at minimum, security awareness training annually and at onboarding. For a tiny startup the hiring side is light-touch, but the two things auditors reliably sample are **background checks** and **security awareness training completion** for everyone hired or active during the period.

"Develop" does not mean an L&D department. It means new joiners get an onboarding that covers the security policies and the stack, everyone completes annual security training (a free/cheap course or a well-built internal one is fine), and engineers stay current on the tools they operate. For an AI-first team, include training on secure AI-assisted development — prompt-injection risks, not committing secrets suggested by a model, reviewing AI-generated code — since your system description will say AI tools are part of the SDLC.

"Retain" and succession planning are judged proportionally: at 5 people, a bus-factor note (documented runbooks, shared access to critical systems via break-glass procedure) is the realistic equivalent of a succession plan.

## Points of focus (2022 revision, summarized)
Guidance from COSO as mapped in the 2022 TSC — illustrative, not required:
- **Establishes policies and practices** — expectations of competence are defined to support achieving objectives.
- **Evaluates competence and addresses shortcomings** — management evaluates competence across the entity and in outsourced providers, and acts on gaps.
- **Attracts, develops, and retains individuals** — the entity provides the mentoring and training needed to attract, develop, and retain sufficient, competent personnel and providers.
- **Plans and prepares for succession** — contingency plans exist for responsibilities important to internal control.
- **Considers the background of individuals** (TSC supplemental) — background screening of employees and contractors proportionate to role risk.
- **Considers the technical competency of individuals** (TSC supplemental) — technical skills needed to operate and secure the system are considered in hiring and training.

## What the auditor will ask for
- Hiring/HR policy or handbook section covering recruiting, screening, and training expectations.
- Background check reports (or screening evidence) for a sample of people hired during the period, including contractors if in scope.
- Job description for a sampled hire.
- Security awareness training records: completion date per person, at onboarding and annually (certificates, LMS export, or ticket trail).
- Onboarding checklist for a sampled new hire.
- Evidence of performance evaluation or competence review (at tiny scale: documented 1:1 notes or annual founder review is acceptable).
- Succession/contingency notes for key roles (runbooks, break-glass access documentation).

## How a tiny AI-first startup satisfies it
- `policies/people-security.md` (or a section in the handbook): screening approach (e.g., third-party background check for employees; reference check + identity verification for short-term contractors), training requirements, and competence expectations per role.
- Background checks: use a low-cost provider (Certn, Checkr) at offer stage; store the completion summary (not the full report) in `evidence/people/<person>/`.
- Security training: assign at onboarding and annually. Practical options: a free course + quiz tracked as a Linear ticket per person per year, or the GRC tool's built-in training module. Add a short internal module on AI-assisted development hygiene (secrets, code review of AI output, approved tools).
- Onboarding/offboarding: templated Linear checklists (`onboarding` label) covering policy acknowledgment, training, and access grants — the closed ticket is the evidence.
- Competence review: a yearly founder-led review note per person (two paragraphs is enough), stored privately; attest to its existence rather than exposing contents.
- Succession/bus-factor: runbooks in the repo, credentials in a shared vault (1Password) with founder-level recovery, and a documented break-glass procedure. Reference these in the policy as the contingency plan.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| People-security/training policy exists and reviewed in last 12 months | GitHub | File existence + latest commit date via `gh api repos/{org}/policies/...` |
| Every active person has a completed security-training ticket in the last 12 months | Linear API | Query issues labeled `security-training`, state Done, one per person per year; roster from `gh api orgs/{org}/members` + Workspace user list |
| Training covers AI-usage module | GitHub/Linear | Grep training ticket template or training doc for AI section heading |
| Onboarding ticket completed for each person who joined during period | Linear API + Workspace | Compare Workspace user `creationTime` to closed `onboarding` tickets |
| Background-check evidence file exists per hire | GitHub | File existence `evidence/people/<person>/background-check.md` on `compliance-archives` |
| Runbooks exist for critical operations | GitHub | File existence checks for `runbooks/` directory and named runbooks (deploy, incident, break-glass) |
| Break-glass / vault recovery configured | MANUAL | 1Password recovery settings screenshot; verified by human |
| Actual competence and quality of training content | MANUAL | Auditor judgment via inquiry and content review |

## Evidence artifacts
- `policies/people-security.md` — screening, training, and competence policy.
- `evidence/people/<person>/` — background-check completion summary, onboarding checklist export, training certificate/ticket link.
- Linear exports: `onboarding`, `offboarding`, and `security-training` tickets for the period, archived quarterly to `compliance-archives`.
- `runbooks/` directory in the main repo (deploy, incident response, break-glass) — bus-factor/contingency evidence.
- GRC-tool training-completion report, if one is in use.
- Annual competence-review attestation (one line per person, signed by founder) in `evidence/people/annual-review-<year>.md`.
