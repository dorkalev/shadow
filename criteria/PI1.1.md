---
id: PI1.1
family: PI1 — Processing Integrity
category: Processing Integrity
coso: null
title: Data definitions and processing specifications
weight: 1
automatable: partial
nature: document
---

# PI1.1 — Data definitions and processing specifications

## Criterion (AICPA TSP Section 100, verbatim)
> The entity obtains or generates, uses, and communicates relevant, quality information regarding the objectives related to processing, including definitions of data processed and product and service specifications, to support the use of products and services.

## What it means
This category is optional — it applies only if the company includes Processing Integrity in its SOC 2 scope. Startups add it when the product's core promise is correct processing — billing engines, data pipelines, document/AI extraction whose outputs customers rely on numerically.

Stripped of the language, PI1.1 asks: have you defined what your system is supposed to do with data, precisely enough that "correct" is checkable — and have you told users? That means definitions of the data you process (schemas, field meanings, units, accepted formats), specifications of the processing itself (what transformations happen, what the outputs mean), and communication of those specs to the people who rely on them (API docs, data dictionaries, product documentation).

For an AI-first startup this has a modern twist: if an LLM sits in the processing path, the "specification" includes what the model step is expected to produce (output schema, allowed values) and honest documentation of accuracy characteristics/limitations. You don't need formal requirements documents — an OpenAPI spec, typed schemas (Pydantic/Zod) in the repo, and public API docs cover most of it, because they *are* your data definitions and they're versioned in git.

## Points of focus (2022 revision, summarized)
These are guidance to consider, not requirements:
- **Identifies information specifications** — the specifications needed to support the use of products and services are identified (summary).
- **Defines data necessary to support a product or service** — data processed is defined: sources, formats, meanings, and how it flows (summary).
- Commonly assessed alongside: specifications and data definitions are communicated to internal and external users who need them (docs, contracts, API references), and are kept current as the product changes (summary).

## What the auditor will ask for
- Product/service specifications: API documentation, product docs describing inputs, processing behavior, and outputs.
- Data definitions: schema documentation, data dictionary, OpenAPI/JSON Schema files, DB schema.
- Evidence specifications are communicated to users (public docs site, docs shared with customers, in-app references).
- Evidence definitions are maintained — change history showing docs/schemas updated as the system changed.
- For AI processing steps: documentation of expected output structure and any published accuracy/limitation statements.
- Internal design docs or Linear specs for processing features shipped during the period (sampled).

## How a tiny AI-first startup satisfies it
- **OpenAPI spec in the repo, generated or hand-written**, published to a docs site. This single artifact is simultaneously the data definition, the service specification, and the communication mechanism — and every change to it is a git commit.
- **Typed schemas at every boundary**: Pydantic/Zod/TypeScript types for inputs, LLM outputs (structured output / function-calling schemas), and API responses. Point the auditor at the schema directory; CI type-checking proves they're enforced, not decorative.
- **A short data dictionary** (`docs/data-dictionary.md`) for the core domain tables/fields — meaning, units, source. One page is fine at this scale.
- **LLM step specs**: for each model-in-the-loop transformation, a note (in the prompt file's header or a `docs/ai-processing.md`) stating expected output schema, validation applied, and known limitations; customer-facing accuracy caveats in product docs.
- **Specs-as-tickets**: feature specs written in Linear serve as the "obtains/generates quality information" evidence — sample-able and timestamped.
- **Keep docs in the deploy path**: doc updates required in PRs touching API surface (checklist item or CI docs-diff check) so definitions can't silently drift.

## Automated shadow checks
| Check | Source | Method |
|---|---|---|
| OpenAPI/schema spec exists in repo | GitHub | `gh api repos/<org>/<repo>/contents/openapi.yaml` (or `api/schema` path) → exists |
| Spec updated alongside API changes | GitHub | `gh api .../commits?path=openapi.yaml` → commits within period; flag if API code changed but spec did not (path-diff heuristic) |
| Typed input/output schemas present | GitHub | `gh api` code search for schema dirs (`schemas/`, `*.zod.ts`, Pydantic models) → non-empty; CI runs type checks (`gh run list`) |
| Data dictionary exists, reviewed < 12 months | GitHub | `gh api .../contents/docs/data-dictionary.md` + last-commit date |
| Public docs site reachable | Web | HTTP GET on docs URL → 200; contains current API version string |
| LLM output schemas validated in code | GitHub | Search repo for structured-output/response-schema usage adjacent to LLM calls → present |
| Feature specs recorded for shipped work | Linear | Linear API: sample of completed issues in period → have spec/description content beyond title |
| Specs are accurate and sufficient | — | MANUAL — auditor reads docs vs. actual behavior |
| Accuracy/limitation disclosures adequate | — | MANUAL — judgment on AI output caveats |

## Evidence artifacts
- `openapi.yaml` (or equivalent) with git history — versioned service specification.
- Schema source directory (Pydantic/Zod models) permalinks, including LLM structured-output schemas.
- `docs/data-dictionary.md` — field-level definitions for core domain data.
- Public documentation site URL + dated capture (PDF/screenshot) for the audit file.
- `docs/ai-processing.md` or prompt-file headers documenting model-step specifications and limitations.
- Sample of Linear issues/specs for processing features shipped during the period.
