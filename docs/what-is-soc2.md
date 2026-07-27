# What SOC 2 Actually Is

## The one-paragraph version

SOC 2 is an **attestation**, not a certification. A licensed CPA firm examines your organization against the AICPA **Trust Services Criteria** (TSP Section 100) and issues a report stating whether your controls are suitably designed (Type I) and operating effectively over a period (Type II). There is no "SOC 2 certificate" and no pass/fail body — there is an auditor's opinion, and the currency of that opinion is **evidence**.

## The governing documents (the only authoritative sources)

| Document | What it governs |
|---|---|
| **TSP Section 100** — 2017 Trust Services Criteria (with Revised Points of Focus — 2022) | The 61 criteria the auditor tests. This is [CHECKLIST.md](../CHECKLIST.md). The 2022 revision changed only the points of focus, not the criteria. |
| **DC Section 200** — Description Criteria | What your **system description** (Section III of the report) must contain. |
| **AT-C 105 / 205** (SSAE 18/21) | The attestation standard the CPA performs the engagement under. |

Everything else — GRC vendors' control lists, tasks, and seed datasets — is a vendor's *interpretation* mapped onto these. Interpretations drift (one popular open-source GRC dataset, for example, contains criteria that don't exist, like CC2.4 and A1.4, and omits CC7.5). The shadow tool only ever scores against TSP Section 100.

## The five categories

1. **Security (Common Criteria, CC1–CC9)** — mandatory in every SOC 2. 33 criteria. CC1–CC5 are the 17 COSO internal-control principles restated; CC6–CC9 are the security-specific supplement (access, operations, change management, risk mitigation).
2. **Availability (A1)** — 3 criteria. Optional.
3. **Confidentiality (C1)** — 2 criteria. Optional.
4. **Processing Integrity (PI1)** — 5 criteria. Optional.
5. **Privacy (P1–P8)** — 18 criteria. Optional, and rarely scoped by tiny startups (privacy obligations are usually handled as GDPR/CCPA programs instead).

You choose the scope. Typical tiny-SaaS scope: **Security + Availability + Confidentiality** (38 criteria).

## Type I vs Type II

- **Type I**: controls suitably designed **as of a date**. A snapshot. Cheaper, faster, weaker signal.
- **Type II**: controls operating effectively **over a period** (usually 3–12 months). The auditor samples evidence across the window: "show me the access review from each quarter", "show me the approval on these 25 randomly selected changes". This is what enterprise buyers ask for.

The consequence that shapes this whole product: **Type II is won or lost on continuously generated evidence.** You cannot backfill a review that never happened. Hence the shadow: gates that generate evidence as a side effect of working, and a clock that catches drift within a day.

## Anatomy of the report

1. **Section I** — Auditor's opinion (unqualified = clean; qualified = exceptions noted).
2. **Section II** — Management's assertion.
3. **Section III** — System description, per DC 200: DC1 types of services; DC2 principal service commitments and system requirements; DC3 components (infrastructure, software, people, procedures, data); DC4 system incidents; DC5 applicable TSC and related controls; DC6 complementary user entity controls (CUECs); DC7 complementary subservice organization controls (CSOCs — what you inherit from GCP/AWS, e.g. all of physical security under CC6.4).
4. **Section IV** — The control matrix: each criterion → your controls → the auditor's tests → results. Exceptions appear here.
5. **Section V** (optional) — Management's responses to exceptions.

## How the audit actually runs

1. **Scoping & readiness** — pick categories, define system boundary, gap assessment (this is what `agent/01-scan-platform.md` + the gauge replicate, continuously).
2. **PBC list** ("provided by client") — the auditor's evidence request list. Every criterion file in [criteria/](../criteria/) has a "What the auditor will ask for" section that mirrors this.
3. **Observation window** (Type II) — you operate; evidence accrues.
4. **Fieldwork** — auditor samples and tests. Population completeness matters: "give me the list of ALL changes in the window" (our `compliance-archives` branch *is* that population, pre-assembled).
5. **Report** — opinion issued. Renewed annually; the gauge never stops mattering.

## Key vocabulary (auditor lingo used across this repo)

- **Criterion** — one of the 61 TSC line items. What is tested.
- **Control** — *your* mechanism that satisfies a criterion (e.g., "branch protection on main requires PR + passing checks"). One criterion ↔ many controls.
- **Point of focus** — AICPA guidance under each criterion on what to consider. Not individually required; the 2022 revision modernized these.
- **Evidence / PBC** — artifacts proving a control operated (exports, tickets, archives, screenshots).
- **Exception** — a control that failed a test (a bypassed merge, a missed access review). Exceptions ≠ failure; unexplained exceptions ≈ failure.
- **CUEC / CSOC** — controls your customers, or your cloud providers, are responsible for. You inherit physical security from your cloud provider as a CSOC and must say so in the system description.
- **Subservice organization** — GCP/AWS/etc. You carve out their controls and rely on *their* SOC 2.
