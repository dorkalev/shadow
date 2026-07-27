---
id: CC6.7
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: Data Transmission and Movement Protection
weight: 3
automatable: partial
nature: technical
---

# CC6.7 — Data Transmission and Movement Protection

## Criterion (AICPA TSP Section 100, verbatim)
> The entity restricts the transmission, movement, and removal of information to authorized internal and external users and processes, and protects it during transmission, movement, or removal to meet the entity's objectives.

## What it means

CC6.7 covers data in motion and data leaving the system: who is allowed to move protected information, and is it protected while moving? "Transmission" is the network path (TLS on every hop), "movement" is data flowing between systems or environments (prod → laptop, prod → third-party API), and "removal" is data leaving your control entirely (exports, downloads, removable media).

For an AI-first startup this criterion has a distinctly modern edge: your biggest data-movement channels are third-party AI APIs (customer data in prompts sent to Anthropic/OpenAI), webhooks, and analytics pipelines — plus the classic ones (email, Google Drive sharing, database dumps to laptops). The auditor wants to see that these flows are enumerated, authorized, and encrypted, and that ad-hoc exfiltration paths are constrained: production data does not get dumped to laptops for debugging, Drive files are not link-shared to the world, USB backups do not exist.

Encryption in transit is the easy 80%: managed TLS on your endpoints, TLS to your cloud databases, HTTPS to every SaaS/AI API. The harder 20% is the "restricts... to authorized users and processes" clause — which for a small team means a written data-handling policy, Workspace sharing defaults locked down, and a short inventory of sanctioned data flows (including exactly what data classes may be sent to which AI providers, and under what agreements — e.g., zero-retention or no-training API terms).

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Restricts the ability to perform transmission** — only authorized users and processes can transmit, move, or remove protected information.
- **Uses encryption technologies or secure communication channels to protect data** — data transmitted over public networks is encrypted (e.g., TLS/VPN).
- **Protects removal media** — data on removable media is encrypted and media use is restricted/controlled.
- **Protects mobile devices and endpoints** — devices that can store or transmit protected information (laptops, phones) are protected (encryption, MDM controls).

## What the auditor will ask for

- Data flow inventory/diagram: what protected data moves where (app ↔ DB, app ↔ AI APIs, webhooks, analytics, backups), with the encryption on each hop.
- TLS configuration evidence for production endpoints and confirmation internal service-to-datastore connections are encrypted.
- Data handling/classification policy covering exports, removable media, and use of production data outside production.
- Google Workspace sharing settings (external sharing defaults, link-sharing posture) and Drive audit examples.
- List of third parties receiving protected data — including AI providers — with the applicable agreements (DPAs, zero-data-retention terms) and what data classes each receives.
- Endpoint controls preventing uncontrolled removal: disk encryption, MDM policy, removable-media policy.
- For sampled data exports during the period: the authorization behind them.

## How a tiny AI-first startup satisfies it

- **TLS on every hop.** Public endpoints: platform-managed certs, TLS 1.2+ (verified by the CC6.6 probe). Databases: Cloud SQL with `requireSsl`/connectors, or serverless datastores that are TLS-only by construction. All SaaS/AI API calls are HTTPS by default — state it and verify no `http://` endpoints exist in config.
- **Enumerate AI data flows explicitly.** A section in `policies/data-handling.md`: which data classes (public / internal / customer) may be sent to which AI providers; production customer data goes only to providers with a DPA and zero-retention/no-training API terms; API keys are per-environment and scoped. This is the paragraph auditors increasingly ask AI startups for.
- **Lock Workspace sharing.** External sharing default off or warn-and-justify; link sharing default "restricted"; Drive DLP is overkill at this size, but the sharing-settings export is cheap evidence.
- **No prod data on laptops.** Policy: debugging uses anonymized fixtures or IAP-tunneled queries, not dumps. Where an export is genuinely needed, it requires a Linear ticket (authorization trail) and deletion after use.
- **No removable media.** Prohibited for production data by policy; laptops are FileVault-encrypted anyway (CC6.4), so any incidental copy is at least encrypted at rest.
- **Webhooks and integrations** are HTTPS with signature verification (e.g., GitHub webhook secrets); inventory them alongside endpoints.

## Automated shadow checks

> Datastore commands are per-stack: Cloud SQL shown. On Firestore stacks (the blessed `provision/gcp`), the equivalents are `gcloud firestore databases describe` (PITR, delete protection, state) and `gcloud firestore backups list` / `backups schedules list` (schedule present, recent snapshots).

| Check | Source | Method |
|---|---|---|
| Public endpoints enforce TLS 1.2+, valid certs | Network | TLS probe of `inventory/endpoints.yaml` entries (shared with CC6.6) |
| HTTP→HTTPS redirect on all public hosts | Network | `curl -sI http://…` → 301/308 to https |
| Cloud SQL requires SSL / no public plaintext path | GCP | `gcloud sql instances describe --format=json` → `settings.ipConfiguration.requireSsl` / `sslMode` |
| No `http://` URLs to external services in code/config | GitHub | `gh api /search/code?q=org:{org}+"http://"` or clone-and-grep excluding localhost — flag hits for review |
| Workspace external-sharing settings locked | Google Workspace | Admin API sharing settings export — MANUAL if API access to Drive settings not configured; else compare to baseline |
| Externally shared / anyone-with-link files in Drive | Google Workspace | Drive API `files.list` with `visibility` filters (domain-wide delegation) → flag `anyoneWithLink` on protected folders |
| AI provider data-flow policy section exists | Repo | Grep `policies/data-handling.md` for provider inventory section |
| DPAs / zero-retention terms on file for each AI provider | Repo | File-existence check `evidence/vendors/<provider>-dpa.pdf` per provider list — substance is MANUAL |
| Data-export tickets exist for period exports | Linear | Linear API: issues labeled `data-export` — completeness of the population is MANUAL |
| Webhook endpoints use HTTPS + secrets | GitHub | `gh api /repos/{org}/{repo}/hooks` → `config.url` scheme, `config.secret` set |
| Removable-media policy documented | Repo | Grep `policies/data-handling.md` for removable-media prohibition |
| Endpoint encryption fleet-wide | MDM | Reuse CC6.4 MDM encryption check; MANUAL without MDM |

## Evidence artifacts

- `docs/data-flows.md` — data flow inventory/diagram incl. AI provider flows, version-controlled.
- `policies/data-handling.md` — classification, transmission, export-authorization, removable-media rules.
- `evidence/network/tls-scan-<date>.txt` — TLS verification output (shared with CC6.6).
- `evidence/gcp/sql-instances-<date>.json` — showing SSL enforcement.
- `evidence/workspace/sharing-settings-<date>.png|csv` and Drive external-share report.
- `evidence/vendors/` — DPAs and AI-provider retention terms (Anthropic/OpenAI zero-retention confirmations).
- Linear `data-export` tickets exported to `evidence/lifecycle/` per quarter.
- `evidence/github/webhooks-<date>.json` — webhook configuration export.
