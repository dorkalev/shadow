# Policy Pack Index

Policies are the written half of CC5.3 ("policies that establish what is expected and procedures that put policies into action") and feed CC1.1, CC2.2, and a dozen others. This directory defines the canon the shadow expects to find in a customer's private `policies` repo, and where to source each template.

Template sourcing: adapt any reputable open-source policy pack (several 20–30-policy canons exist) or a GRC tool's markdown exports; the table below is the canon this platform expects, regardless of source.

## The minimum set (tiny startup, Security + Availability + Confidentiality scope)

| # | Policy | Primary criteria | Template source |
|---|---|---|---|
| 1 | Information Security Policy (umbrella) | CC1.1, CC5.3 | template pack |
| 2 | Access Control Policy (incl. onboarding/termination) | CC6.1–CC6.3 | template pack |
| 3 | Secure Development / SDLC Policy — points at [`../sdlc/SDLC.md`](../sdlc/SDLC.md) | CC8.1 | template pack |
| 4 | Incident Response Plan | CC7.3–CC7.5 | template pack |
| 5 | Business Continuity & Disaster Recovery Policy | CC9.1, A1.2, A1.3 | template pack |
| 6 | Risk Assessment / Management Policy | CC3.1–CC3.4, CC9.1 | template pack |
| 7 | Vendor Management Policy | CC9.2 | template pack |
| 8 | Data Classification & Handling Policy | C1.1, CC6.7 | template pack |
| 9 | Data Retention & Disposal Policy | C1.2, CC6.5 | template pack |
| 10 | Encryption & Password/Credential Policy | CC6.1, CC6.7 | template pack |
| 11 | Acceptable Use & Code of Conduct | CC1.1, CC1.5 | template pack |
| 12 | Change Management Policy — thin wrapper over the SDLC | CC8.1 | write fresh |
| 13 | Vulnerability Management Policy (with remediation SLAs — the SLAs Runbook 03 enforces) | CC7.1 | write fresh |
| 14 | **AI Development & Agent Use Policy** — agent identities, credential scoping, no self-approval, prompt/spec-on-ticket rule, LLM-vendor data handling | CC8.1, CC6.3, CC9.2 | write fresh — our differentiator |

Add if Privacy is scoped: Privacy Management Policy (P1–P8), plus the public privacy notice.

## Lifecycle rules (what the shadow verifies)

- Policies live in a private git repo and change **only through the SDLC's PR flow** — the merged PR is the approval record.
- Frontmatter on every policy: `owner`, `version`, `approved_by`, `approved_at`, `review_by` (≤12 months out).
- Annual re-approval: `review_by` in the past ⇒ Runbook 03 decays CC5.3 and opens a ticket.
- Staff attestation: each person acknowledges the pack yearly (a signed row in `evidence/{YYYY}/attestations.md` or the GRC tool's policy-acceptance feature); new joiners within 2 weeks (onboarding checklist item).
