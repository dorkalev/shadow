---
id: CC6.8
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: Malicious Software Prevention and Detection
weight: 3
automatable: partial
nature: technical
---

# CC6.8 — Malicious Software Prevention and Detection

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements controls to prevent or detect and act upon the introduction of unauthorized or malicious software to meet the entity's objectives.

## What it means

CC6.8 asks how you keep unauthorized or malicious software out of two places: your production environment and your endpoints. "Unauthorized" is broader than "malware" — it includes an engineer deploying code that never went through review, an unapproved binary in a container image, and a compromised npm package pulled in by `npm install`. For a modern startup the dominant real-world vectors are supply-chain (malicious dependencies, typosquatted packages, poisoned GitHub Actions) and endpoint malware — not viruses on servers, because your "servers" are ephemeral containers.

The tiny-startup story that satisfies auditors: production code changes only land via reviewed PRs on protected branches (change control doubles as unauthorized-software prevention); deployments happen only from CI, so nobody hand-installs software on prod; dependencies are monitored by Dependabot and lockfiles pin what actually runs; containers are rebuilt from pinned bases and can be vulnerability-scanned; endpoints run at least the built-in protections (macOS XProtect/Gatekeeper) and ideally a lightweight EDR, with MDM restricting who can install system software.

AI-assisted development adds a wrinkle worth addressing head-on: AI agents generate and sometimes execute code and can hallucinate dependencies. The controls are the same ones — agents' output enters production only via the same reviewed-PR pipeline, agents run with scoped tokens, and dependency review flags new packages on every PR — but you should say so explicitly in your SDLC policy, because auditors are starting to ask.

## Points of focus (2022 revision, summarized)

Summaries of AICPA points of focus — guidance, not requirements:

- **Restricts application and software installation** — the ability to install software on system components and endpoints is limited to authorized personnel/processes.
- **Detects unauthorized changes to software and configuration parameters** — mechanisms exist to detect changes to production software and configuration (e.g., IaC drift, integrity monitoring, deploy-pipeline exclusivity).
- **Uses a defined change control process** — software is introduced to production only through the entity's change management process.
- **Uses antivirus and anti-malware software** — malware protection is deployed and updated where relevant (chiefly endpoints).
- **Scans information assets from outside the entity** — files, software, and dependencies received from outside are scanned/evaluated before use.

## What the auditor will ask for

- SDLC/change management policy showing all production code passes review and CI before deploy (including AI-generated code).
- Branch protection configuration exports and a sample of merged PRs with approvals and passing checks.
- Evidence deployments occur only via CI (deploy service account exclusivity; humans lack direct prod deploy rights).
- Dependency management evidence: Dependabot alert reports, remediation SLAs, examples of alerts triaged/closed during the period.
- Container/image scanning results or an explanation of the scanning posture.
- Endpoint anti-malware posture: EDR/MDM report, or documented reliance on macOS built-ins plus install restrictions.
- How unauthorized change to production would be detected (audit logs on deploys/IAM, IaC drift detection).
- Malware/security incident records for the period, or attestation of none.

## How a tiny AI-first startup satisfies it

- **Reviewed-PR-only production.** Branch protection on `main` with ≥1 non-author approval, required status checks, `enforce_admins`, no force pushes. AI agents open PRs like anyone else; a human approves. This is the single control that carries "prevent unauthorized software" for the app itself.
- **CI is the only deployer.** GitHub Actions (OIDC → cloud) holds the only deploy permission; humans do not have `run.admin`/deploy roles on prod. Cloud Audit Logs record every deployment, so an out-of-band deploy is detectable, not silent.
- **Supply chain hygiene.** Lockfiles committed; Dependabot alerts + security updates enabled on all repos; `dependency-review-action` on PRs to flag newly introduced or malicious-flagged packages (also catches AI-hallucinated dependencies); GitHub Actions pinned to SHAs for third-party actions.
- **Image scanning where cheap.** GCP Artifact Registry vulnerability scanning (on-push) or `trivy` in CI; triage criticals within a defined SLA (e.g., 7 days).
- **Endpoints.** MDM blocks non-admin software installs and enforces Gatekeeper; if budget allows, a light EDR (e.g., SentinelOne via MDM, or at minimum macOS built-ins documented as the control). Malware detection events route to the incident process.
- **Say the AI part out loud.** One paragraph in `policies/sdlc.md`: AI coding tools are approved tooling; their output is subject to identical review/CI gates; agent credentials are scoped, short-lived, and cannot push to protected branches.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| Branch protection with required review + status checks + enforce_admins | GitHub | `gh api /repos/{org}/{repo}/branches/{default}/protection` |
| Merged PRs in period all had approval and passing checks | GitHub | `gh api /repos/{org}/{repo}/pulls?state=closed` → sample `reviews` and check-runs per merged PR |
| Dependabot alerts enabled on all repos | GitHub | `gh api /repos/{org}/{repo}/vulnerability-alerts` (204 = enabled) |
| No open critical/high Dependabot alerts beyond SLA | GitHub | `gh api /repos/{org}/{repo}/dependabot/alerts?state=open&severity=critical,high` → age check |
| Dependency review action present in PR workflows | GitHub | Fetch `.github/workflows/*` via `gh api /repos/{org}/{repo}/contents` → grep `dependency-review-action` |
| Lockfiles present in all active repos | GitHub | Contents API check for `package-lock.json`/`pnpm-lock.yaml`/`uv.lock` etc. |
| Third-party Actions pinned to SHA | GitHub | Workflow file parse → flag `uses: owner/action@v*` for third-party actions |
| Humans lack prod deploy roles; only CI SA can deploy | GCP | `gcloud projects get-iam-policy` → deploy-capable roles bound only to the CI service account |
| Artifact Registry scanning enabled / recent scan results clean | GCP | `gcloud artifacts docker images list --show-occurrences` / Container Analysis API for vulnerabilities |
| Deploy audit logs present and continuous | GCP | `gcloud logging read` filtered to deploy methods → confirm entries exist for known releases |
| Secret scanning push protection (blocks one malware-adjacent vector) | GitHub | `gh api /repos/{org}/{repo}` → `security_and_analysis` block |
| EDR/MDM software-restriction compliance | MDM/EDR API | Vendor API device compliance report; MANUAL without MDM/EDR |
| Malware incidents triaged per incident process | Linear | Linear issues labeled `security-incident` reviewed — population completeness is MANUAL |

## Evidence artifacts

- `evidence/github/branch-protection/<repo>.json` — protection exports (shared with CC6.1/CC6.3).
- `evidence/github/dependabot-alerts-<date>.json` — open/closed alert exports with timestamps demonstrating SLA.
- `evidence/github/pr-sample-<quarter>.json` — merged-PR review/check evidence.
- `evidence/gcp/image-scan-<date>.json` — Artifact Registry / trivy scan output.
- `evidence/gcp/deploy-audit-logs-<quarter>.json` — deployment log extract; IAM export showing CI-only deploy rights.
- `evidence/endpoints/edr-report-<date>.csv` or MDM software-restriction policy export.
- `policies/sdlc.md` — change management incl. the AI-generated-code paragraph, version-controlled.
- Linear `security-incident` issues (or a signed "no malware incidents" quarterly attestation) in `evidence/incidents/`.
