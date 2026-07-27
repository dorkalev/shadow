# Grand Plan — SOC 2 Compliance Shadow

## 1. What we are building

A **compliance shadow**: a tool that plays the *auditor's* role continuously, not the GRC-platform role. Companies may already have a GRC tool — we don't replace them. We sit on the other side of the table: we hold the exact AICPA checklist (TSP Section 100, all 61 criteria), we look at the company's real systems the way an auditor would, and we keep a live 0–100% gauge of "would you pass today?".

Target user: **tiny startups (1–10 people) that develop with AI** (Claude Code or another LLM agent) and still want to be SOC 2 compliant. The product is delivered as:

1. **A markdown corpus** (this repo) — the criteria, the dictated SDLC, and agent instructions. The corpus *is* the product; any capable LLM can execute it.
2. **An agent skill** — the user runs Claude against their codebase/cloud with our instruction files; the agent scans, sets up the SDLC, and verifies compliance.
3. **A one-page website** — a fixed gauge (0–100%) plus the 61-item checklist, backed by SQLite. The least code possible.

Our unique privilege: because our users are tiny and AI-first, **we get to dictate the SDLC** rather than adapt to one. One simple, opinionated, SOC 2-compliant SDLC (see `sdlc/SDLC.md`) that an LLM agent can both follow and enforce.

## 2. Proven patterns this platform bakes in

| Pattern | What it gives us |
|---|---|
| Per-PR compliance agent | Deterministic ticket extraction (regex on PR title/body, crash-safe), ticket traceability, per-file change traceability, test coverage, review gate, confidence scoring (start 100, −10 invalid ticket, −10 unspecced file, −5 untested file, −5 missing reviewer), mandatory hard gates (no ticket ⇒ fail; PR body < 20 chars ⇒ fail; unresolved CRITICAL/MAJOR ⇒ fail), live in-place PR comment. |
| Two-phase gate + evidence archive | An audit check at `awaiting-review` and a review-gate at `post-review`; a **post-merge archive** (JSON + MD per merged PR on a dedicated `compliance-archives` branch); **bypass-merge detection** reading the live GitHub branch ruleset; Slack notifications; application-level audit-event logging for privileged actions. |
| Spec-first agent workflow | start → load (spec to the ticket) → finish (spec-alignment gate, blocks until diff matches spec) → fix-compliance → fix-pr → release (confirmation word, release ticket, fast-forward-only main, archive record) → hotfix (incident ticket + backport PR). |
| GRC data model | Controls with owners and categories, **monitors** (name, PASSING/FAILING, category, linked framework), policies with framework linkage, risk register (inherent/residual likelihood×impact, treatment), vendor register with reviews, evidence documents with OK/NEEDS_FILE status. |
| Policy canon | The standard ~14–27-policy pack and per-criterion standard mappings, adapted per company. |
| Vendor-dataset skepticism | Popular GRC vendors' SOC 2 datasets drift — some ship criteria that do not exist (e.g. CC2.4, CC6.9, A1.4) and omit real ones (CC7.5). **Never treat a vendor list as authoritative; only TSP Section 100 is.** That data-quality gap is literally our pitch. |

## 3. Repository layout (this repo)

```
soc2/
├── PLAN.md                  ← this file
├── README.md                ← orientation
├── CHECKLIST.md             ← the auditor's checklist: all 61 criteria, verbatim, checkboxes
├── criteria/                ← 61 files, one per criterion (CC1.1.md … P8.1.md)
│   └── <ID>.md              ← verbatim text, meaning, points of focus, PBC list,
│                              tiny-startup controls, automated shadow checks, evidence
├── docs/
│   └── what-is-soc2.md      ← SOC 2 meaning: report types, TSC, description criteria, audit process
├── sdlc/
│   └── SDLC.md              ← the one simple SOC 2-compliant SDLC we dictate
├── agent/                   ← instructions an LLM executes against a customer's systems
│   ├── 01-scan-platform.md  ← read-only discovery: repos, cloud, identity, current posture
│   ├── 02-setup-github-sdlc.md ← install the SDLC: rulesets, CI gates, archives, policies repo
│   ├── 03-verify-compliance.md ← the periodic shadow audit: run every check, score, write state
│   └── 04-proactive-sdlc.md ← make the SDLC self-enforcing: per-PR gates, hooks, escalation
├── website/
│   └── SPEC.md              ← one-pager app spec: gauge + checklist, SQLite, minimal code
└── policies/
    └── README.md            ← policy pack index and lifecycle rules
```

## 4. The scoring model (the gauge)

The gauge is **criterion-based, evidence-weighted** — like an auditor's readiness assessment, not a task tracker.

- Every criterion has a **weight** (frontmatter in its file): 3 = core security, 2 = important, 1 = optional-category or lighter.
- Every criterion has a **status**, computed from its automated shadow checks plus manual attestations:
  - `verified` (all automated checks pass, evidence present) → 100% credit
  - `implemented` (controls exist, evidence partial) → 60% credit
  - `in_progress` → 25% credit
  - `not_started` / `failing` → 0% credit
- Out-of-scope categories (A/C/PI/P when not selected) are excluded from the denominator.
- **Gauge = Σ(weight × credit) / Σ(weight) over in-scope criteria.**
- Hard-gate overlay: certain failures cap the gauge regardless of score — no MFA enforcement on the org (CC6.1/6.2), no branch protection on production branches (CC8.1), bypass merge in the last 30 days without an incident ticket (CC8.1) each cap the gauge at 79%. An auditor would call these exceptions no matter how pretty the rest looks.

Status lives in SQLite (`shadow.db`, table per §6) and is recomputed by `agent/03-verify-compliance.md` runs; manual attestations are rows a human (or the agent, with evidence links) inserts.

## 5. The enforced SDLC (summary — full spec in `sdlc/SDLC.md`)

One trunk-adjacent flow, small enough for a solo founder, strict enough for an auditor:

1. Every change starts as a **ticket** (Linear or GitHub Issues).
2. Branch `TICKET-ID-slug` off `staging`; PR to `staging` with ticket table + per-file Changes section + Test Plan.
3. **Gates on the PR** (required checks): CI tests, a review (an independent review bot is acceptable at this size; human where available), and the **compliance agent** (traceability + coverage + review-gate + score ≥ threshold).
4. Merge ⇒ **post-merge archive** (JSON+MD to `compliance-archives` branch) with bypass detection.
5. **Release** = fast-forward `staging` → `main` with a release record + release ticket. Direct pushes to `main` only as documented **hotfixes** with incident ticket + backport PR.
6. Everything the auditor will ever ask for about change management is thereby generated as a side effect of working.

## 6. The one-pager website (summary — full spec in `website/SPEC.md`)

- Single page: fixed gauge 0→100% on top, the 61-criterion checklist below, grouped by family, each row showing status, last-verified timestamp, and failing checks.
- Storage: one SQLite file (`shadow.db`): `criteria` (id, weight, scope, status, credit), `checks` (criterion_id, name, status, last_run, detail), `attestations`, `events`.
- Stack decision: **Rust, single static binary** (axum or tiny_http + rusqlite, server-rendered HTML, zero JS build). Rationale: you like Rust, the app is read-mostly with one JSON ingest endpoint, and "one binary + one .db file" matches the shadow-auditor ethos. Python/Flask is the fallback if iteration speed ever matters more; the spec is stack-agnostic and small enough that either lands under ~300 lines.
- The website never computes compliance. It renders state. The **agent** computes compliance and POSTs (or writes the .db directly). This keeps the site dumb, small, and honest.

## 7. Proactive + periodic verification (the missing link)

Two loops:

- **Event loop (proactive):** per-PR compliance agent + review gate as required status checks; post-merge archive on every merge; bypass detection against the live branch ruleset; hotfix procedure for emergencies. The SDLC blocks non-compliance *before* it happens.
- **Clock loop (periodic):** a scheduled run (GitHub Actions cron, launchd, or Claude Code `/schedule`) executes `agent/03-verify-compliance.md`: re-runs every automated shadow check across all criteria (org MFA, branch rulesets, dependabot/secret-scanning alerts, IAM key age, backup config, access-review freshness, policy-repo currency, archive completeness), recomputes the gauge, writes SQLite, and opens tickets for regressions. Drift becomes a ticket within a day, not a finding within a year.

## 8. Phases

- **Phase 0 — Corpus (this repo, now).** CHECKLIST.md, 61 criterion files, docs, SDLC spec, 4 agent instruction files, website spec, policy pack index. No code.
- **Phase 1 — Pilot on one real project.** Pick a project; run `agent/01` (scan) → present gap report → run `agent/02` (setup) → first `agent/03` (verify) produces the initial gauge.
- **Phase 2 — Website.** Build `website/SPEC.md` (Rust, ~1 day). Point `agent/03` output at it.
- **Phase 3 — Hardening.** Schedule the clock loop; wire Slack notifications; add the policy pack (adapt the canon in policies/README.md to the customer); first mock audit against CHECKLIST.md.
- **Phase 4 — Productize.** Package `agent/` as a Claude Code skill/plugin ("/shadow scan", "/shadow verify"); multi-project support in the website; onboarding doc for non-Claude LLMs.

## 9. Honesty clauses (what we tell users)

- The CC-series is mandatory; A/C/PI/P are scoped in only if you commit to them. We default new customers to **Security + Availability + Confidentiality**, the common tiny-SaaS scope.
- A 100% gauge is *readiness*, not certification. Only a CPA firm can issue a SOC 2 report. We make the audit boring, we don't replace it.
- Organizational criteria (board oversight, hiring, training) cannot be fully automated; the shadow tracks them as attestations with evidence links and ages them out (an attestation older than 12 months decays to `in_progress`).
