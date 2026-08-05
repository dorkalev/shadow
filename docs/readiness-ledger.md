# Shadow readiness ledger

Shadow makes a repository **SOC 2 readiness-shaped from day zero**. It does not
issue a SOC 2 report and it cannot manufacture the operating history required
for a Type II examination. A licensed CPA still decides the examination result.

The daily path is deterministic and makes **zero model API calls**:

1. `shadow-ci verify` observes repository files, GitHub controls, configured
   Firebase/GCP controls, evidence packets, and expiring human attestations.
2. It writes a vendor-neutral schema-v2 JSON snapshot. This JSON is the source
   of truth; SQLite is only a disposable dashboard render cache.
3. `shadow import-verify` calculates the gauge and four non-interchangeable
   readiness measures.
4. GitHub Actions uploads the snapshot, dashboard, and cache. With
   `ARCHIVES_PUSH=1`, the verifier also appends the snapshot to the
   `compliance-archives` branch.

## The four measures

| Measure | What 100% means |
|---|---|
| Design readiness | Every in-scope criterion has a current, non-placeholder control design. |
| Technical health | Every applicable live technical check returned pass; unknowns count against it. |
| Evidence coverage | Every in-scope criterion has current proof from at least one source. |
| Operating maturity | Every in-scope criterion has current technical or human evidence that the control operated. This grows with time. |

The large gauge is a conservative roll-up. A failure wins over a pass, unknown
earns no credit, design-only proof earns partial credit, and technical/operating
proof earns full credit. “Not applicable” is excluded rather than silently
treated as pass.

## Agents and cost boundaries

| Agent | Trigger | Authority | Model cost |
|---|---|---|---|
| Change gate | Every pull request | Read repository/PR metadata; block merge | $0 |
| Evidence archivist | Every merge | Append the reviewed change record | $0 |
| Readiness verifier | Daily/manual | Read repository, GitHub, and configured GCP state | $0 |
| Ritual packet builder | Quarterly/manual | Prepare access/management/restore evidence packets | $0 |
| Human control owner | Before an attestation expires | Review evidence and sign a narrow statement | Human time only |
| Semantic review adviser | Manual opt-in only | Read the selected diff/criterion; no control ownership | Bounded model spend |

An LLM is never the source of truth, never scheduled by default, and never
allowed to turn `unknown` into `pass`. Adopting Drata, Vanta, or an auditor portal
later is an adapter problem: ingest `shadow/readiness-latest.json` or the
append-only snapshots using `readiness-snapshot.schema.json`.

## Path to 100

1. Declare honest scope in `shadow/scope.json`. Security is mandatory for SOC 2;
   Availability, Confidentiality, Processing Integrity, and Privacy are selected
   according to actual customer commitments.
2. Replace every draft/OPEN/TODO document with a reviewed policy or register.
3. Make all repository, GitHub, Firebase, and GCP technical checks pass.
4. Run the recurring rituals and restore/tabletop exercises; file their outputs
   under `evidence/`.
5. For a genuinely human control, record a narrow, expiring attestation with
   `shadow-ci control-attest`. The next daily run will fail it after expiry.
6. Accumulate clean snapshots and change archives throughout the audit window.

Reaching 100 means Shadow has current proof for its declared control model. It
does not mean the CPA has completed fieldwork or issued a report.

## Durable history (explicit privilege)

Appending to `compliance-archives` requires repository write permission. It is
deliberately not enabled by the read-only copy-paste workflow. After reviewing
that permission, set `ARCHIVES_PUSH=1`, `ARCHIVES_BRANCH=compliance-archives`,
and grant the workflow `contents: write`. GCP live checks likewise require an
explicit Workload Identity Federation configuration; absent credentials remain
visible as `unknown` and receive zero credit.
