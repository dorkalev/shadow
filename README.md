# Shadow — a SOC 2 assistive platform

**The goal: make SOC 2 approachable for the indie hacker and the solopreneur** — reliable, honest, yet dirt-cheap governance and controls for the builder who needs real assurance without an enterprise GRC budget or a full-time compliance hire.

**Shadow is a SOC 2 *assistive* platform, not a compliance platform of its own.** It installs the **gateways** that make a codebase's change management enforceable, and runs a continuous **self-check against the principles of SOC 2** (the AICPA Trust Services Criteria — Security, Availability, Processing Integrity, Confidentiality, Privacy), rendering one honest gauge from 0 to 100%. It also offers a **template for SOC 2-compliant company infrastructure** you can apply in one step (annotated Terraform + GitHub workflows).

What that means precisely:
- **We install gateways, not verdicts.** Branch rulesets, a deterministic per-PR change-management gate, evidence archives with bypass detection, and scanners — the machinery that generates audit evidence as a side effect of working.
- **We self-check the principles.** For each of the 61 Trust Services Criteria, Shadow runs the checks it *can* automate against your real systems and reports status; the criteria docs make plain what only a human can attest.
- **We ship an infrastructure template.** `provision/` is a one-command SOC 2-shaped runtime (keyless deploys, backups, audit logs, least-privilege identity) you adopt as-is or adapt.

**What Shadow is *not*:** it is not a certification, not an audit, and not a GRC platform of record. It does not issue SOC 2 reports — only a licensed CPA firm can, after evidence accrued over an examination period. Shadow makes you *ready* and keeps you honest between audits; a green gauge is self-assessed readiness, not a report. See the full disclaimer below.

The deterministic verifier and vendor-neutral evidence ledger are the product core. The corpus supplies the human/auditor context; an LLM is an optional adviser for bounded semantic work, never a scheduled source of truth. MIT-licensed — see [LICENSE](LICENSE).

## Map

| Path | What it is |
|---|---|
| [PLAN.md](PLAN.md) | The grand plan — vision, architecture, scoring model, phases. Start here. |
| [COVERAGE.md](COVERAGE.md) | Measured Rust coverage baseline, enforced ratchet, and the path to 100%. |
| [CHECKLIST.md](CHECKLIST.md) | The auditor's checklist: all **61 criteria** of AICPA TSP Section 100 (2017 TSC, 2022 revised points of focus), verbatim, as checkboxes. |
| [criteria/](criteria/) | One in-depth file per criterion (CC1.1 … P8.1): verbatim text, meaning, points of focus, what the auditor asks for, tiny-startup controls, automated shadow checks, evidence artifacts. |
| [docs/what-is-soc2.md](docs/what-is-soc2.md) | SOC 2 meaning: attestation vs certification, Type I/II, report anatomy, description criteria, auditor lingo. |
| [sdlc/SDLC.md](sdlc/SDLC.md) | The one simple SOC 2-compliant SDLC we dictate — ticket → branch → gated PR → archive → release, plus the clock rituals. |
| [agent/](agent/) | Optional judgment runbooks: scan/setup, deep semantic review, and developer assistance. The daily readiness reading comes from deterministic `shadow-ci verify`. |
| [provision/](provision/) | The compliant runtime as annotated Terraform (GCP: Cloud Run + Firestore, always-free-tier eligible, zero keys and zero database secrets, keyless WIF deploys) — `/shadow:provision` applies it once, `deploy.yml` ships keylessly forever after. |
| [actions/](actions/) | What adopting projects vendor: **shadow-ci** (Rust binary — deterministic per-PR compliance gates + post-merge evidence archive with bypass detection) and the three GitHub workflow templates. No Python, no LLM in the merge path. |
| [docs/readiness-ledger.md](docs/readiness-ledger.md) | The zero-model daily evidence contract, four readiness measures, expiring human attestations, vendor-neutral export, and the honest path to 100. |
| [commands/](commands/) | The developer workflow as Claude commands (capture → start → load → verify → finish → fix-compliance / fix-pr → release, hotfix as the emergency path) **plus the ritual interviews** — every traditionally-manual SOC 2 task (access reviews, management reviews, risk/vendor/policy refreshes, tabletop, on/offboarding, postmortems, system description, audit binder) reduced to gather → judgment-only dialogue → auto-filed evidence. |
| [website/SPEC.md](website/SPEC.md) | The one-pager app: fixed 0–100% gauge + checklist, SQLite-backed, Rust single binary, ≤300 lines. |
| [policies/README.md](policies/README.md) | The policy pack canon and lifecycle rules. |

## Showtime

```
./showtime.sh                    # the spectacle: board opens, 61 boxes cascade green (~30s, simulated, $0)
./showtime.sh --real             # the truth: the verifier runs every criterion's actual checks;
                                 # failures open gh issues automatically (label: shadow)
./showtime.sh --firebase PROJ    # additionally publish the static gauge to Firebase Hosting
```

CI-style unified log on stdout; the micro board self-refreshes so the greens land in front of your eyes.

```
./genesis.sh                     # from NOTHING: creates a brand-new repo, installs the shadow,
                                 # then a REAL ticket → commit → PR → gate REJECTS it → fix →
                                 # gate PASSES → merge → archive record appears. ~10 min, all inspectable.
```

Genesis now ends with **judgment**: after proving the pipeline it builds a deterministic, zero-model readiness snapshot against the new repo (and, with `--gcp-project P --alert-email E`, terraform-applies the Firestore runtime first so cloud controls verify against real infrastructure). It also installs `judgment.sh` INTO the new repo:

```
./judgment.sh                    # from inside any shadow-installed repo, any time:
                                 # re-prove the route (ticket → commit → PR → gate rejects →
                                 # fix → passes → merge → archive), then rebuild the
                                 # evidence snapshot and dashboard with zero model calls
./judgment.sh --skip-pipeline    # snapshot only
./judgment.sh --skip-pipeline --deep-llm  # optional paid semantic review
```

```
./testimony.sh --since 2026-01-01    # the CPA's fieldwork, self-run: population from three
                                     # sources, reconciled; 100% attribute testing; exceptions
                                     # listed or "NO EXCEPTIONS" — the report you hand the auditor
```

Showtime demonstrates the board; genesis installs a compliant change pipeline + scaffold (readiness, not a finished audit); **atonement** fixes what genesis couldn't (org 2FA, Workspace 2SV, the policy pack, human approver) — auto-running what has an API, deep-linking + Playwright-guiding the console-only rest; judgment tests it; testimony proves the history. A SOC 2 report still requires a CPA and evidence accrued over months.

```
./atonement.sh --guided          # walk genesis's "NOT established" list: auto-apply org
                                 # settings, autogenerate the policy pack as a gated PR, and
                                 # open console-only settings in a guided browser (step overlay)
```

## Fidelity guarantee

Criterion IDs and texts come verbatim from **AICPA TSP Section 100** — verified against the official PDF, not from any GRC vendor's dataset (those drift; one popular open-source one ships criteria that don't exist). Points-of-focus sections are labeled summaries. If it isn't in TSP 100, it isn't in the checklist.

## Quick start (pilot on a project)

1. Read [PLAN.md](PLAN.md).
2. In the target project, run an agent on [agent/01-scan-platform.md](agent/01-scan-platform.md) → review `shadow/scan-report.md`.
3. Approve and run [agent/02-setup-github-sdlc.md](agent/02-setup-github-sdlc.md).
4. Run [agent/03-verify-compliance.md](agent/03-verify-compliance.md) → your baseline gauge. Schedule the deterministic verifier daily; run the LLM deep review only when explicitly approved.
5. Develop through [agent/04-proactive-sdlc.md](agent/04-proactive-sdlc.md). Watch the gauge climb.

## Positioning & disclaimer

Shadow is an **assistive** tool. It helps a team install and operate the technical controls and evidence trails that a SOC 2 examination looks for, and it self-assesses progress against the Trust Services Criteria. It does **not**:

- issue, grant, or represent a SOC 2 report, certification, or attestation;
- replace a licensed CPA firm, which alone can perform a SOC 2 examination and issue an opinion;
- replace a GRC program of record — Shadow can run alongside an existing one, or stand in before you adopt one.

A SOC 2 Type II report additionally requires controls to **operate over an examination period** (typically 3–12 months); no tool can fast-forward that elapsed evidence. Shadow's gauge is **self-assessed readiness**, not an audit result. "SOC 2", "AICPA", and "Trust Services Criteria" refer to the standards of the American Institute of Certified Public Accountants; Shadow is not affiliated with, endorsed by, or certified by the AICPA. Nothing here is legal or audit advice. Use is governed by the MIT [LICENSE](LICENSE), **without warranty of any kind** — including any warranty that use results in SOC 2 compliance.

## License

[MIT](LICENSE).
