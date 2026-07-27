---
id: CC6.1
family: CC6 — Logical and Physical Access Controls
category: Security (Common Criteria)
coso: null
title: Logical Access Security Architecture
weight: 3
automatable: partial
nature: technical
---

# CC6.1 — Logical Access Security Architecture

## Criterion (AICPA TSP Section 100, verbatim)
> The entity implements logical access security software, infrastructure, and architectures over protected information assets to protect them from security events to meet the entity's objectives.

## What it means

CC6.1 is the umbrella access-control criterion. It asks: do you know what your protected information assets are (code, customer data, production infrastructure, secrets, SaaS accounts), and have you put technical access controls in front of every one of them? "Logical access security software, infrastructure, and architectures" is auditor-speak for the whole stack — identity providers, MFA, IAM policies, network boundaries, encryption, and secret management — working together so that only authenticated, authorized identities can touch protected assets.

For a 1–10 person AI-first startup, the asset inventory is short and the identity plane is narrow: a Google Workspace tenant, a GitHub org, one or two cloud projects (GCP/AWS), a handful of SaaS tools (Linear, Slack, AI providers), and employee laptops. That is an advantage. The auditor does not expect a PAM appliance or a network operations center; they expect that every one of those surfaces requires strong authentication (MFA/SSO), that cloud IAM follows least privilege, that data is encrypted at rest and in transit, that secrets are not sitting in the repo, and that you can articulate the system boundary and point to the control at each entry point.

The most common tiny-startup failures under CC6.1 are boring: a GitHub org that recommends but does not enforce 2FA, long-lived service-account JSON keys downloaded to laptops, a shared "admin@" login, default-open firewall rules, and API keys committed to code. The criterion also covers architecture decisions — e.g., production data lives only in the cloud project, not on laptops; AI coding agents run with scoped tokens, not owner credentials.

CC6.1 is where the auditor tests the design of access control; CC6.2 and CC6.3 test the lifecycle (provisioning, deprovisioning, reviews). Expect findings here to cascade: if 2FA enforcement fails, half of CC6 fails with it.

## Points of focus (2022 revision, summarized)

These are summaries of AICPA points of focus — guidance for evaluation, not requirements:

- **Identifies and manages the inventory of information assets** — the entity knows what data, software, and infrastructure it protects and where they live.
- **Restricts logical access** — access to protected assets is limited through access control software and rule sets.
- **Identifies and authenticates users** — users, devices, and other systems are uniquely identified and authenticated before access.
- **Considers network segmentation** — the network is segmented (or the architecture otherwise isolates) to limit unnecessary access between assets.
- **Manages points of access** — entry points to the system (VPNs, APIs, consoles) are identified, inventoried, and managed.
- **Manages identification, authentication, and credentials** — credentials for users and for infrastructure/software (service accounts, tokens, keys) are issued, rotated, and removed under a defined process.
- **Uses encryption to protect data** — encryption protects data at rest where appropriate to the risk.
- **Protects cryptographic keys** — encryption keys are protected during generation, storage, use, and destruction.

## What the auditor will ask for

- System description and asset inventory: list of protected information assets, data stores, and in-scope systems with owners.
- Identity provider configuration: Google Workspace SSO/MFA enforcement settings and an MFA enrollment report for all users.
- GitHub org settings screenshot/export showing 2FA enforcement, base permissions, and the member list.
- Cloud IAM policy export (GCP `gcloud projects get-iam-policy` / AWS IAM) demonstrating least privilege and no broad `Owner`/`*` grants to individuals.
- Evidence of encryption at rest for production data stores (cloud default-encryption documentation or KMS configuration) and key management approach.
- Secret management approach: where secrets live (Secret Manager/SSM, GitHub Actions secrets), and evidence secret scanning is enabled with alerts triaged.
- Network configuration: firewall rules / security groups export showing no unintended public exposure (e.g., no 0.0.0.0/0 on SSH/DB ports).
- Population of service accounts and API keys with justification and rotation evidence for the audit period.

## How a tiny AI-first startup satisfies it

- **One identity plane.** Google Workspace is the IdP; every SaaS that supports Google SSO uses it (GitHub via SAML if on Enterprise, otherwise Google-linked accounts + enforced 2FA). Workspace enforces 2-Step Verification for all users, security-key or authenticator only (no SMS).
- **GitHub org hardening.** `two_factor_requirement_enabled: true`, base permission `read` or `none`, branch protection on `main` (required PR review, required status checks, no force push), secret scanning + push protection, and Dependabot alerts on all repos.
- **Cloud IAM least privilege.** No individual holds `roles/owner` day-to-day; humans get predefined roles scoped to what they do. No user-managed service-account JSON keys — use Workload Identity Federation for CI (GitHub Actions OIDC → GCP/AWS) and attached service accounts for workloads. Where keys are unavoidable, they are inventoried and rotated ≤90 days.
- **Encryption everywhere by default.** GCP/AWS managed encryption at rest (document it — it is inherited, and that is fine), TLS 1.2+ on every endpoint, HTTPS-only load balancers.
- **Secrets out of code.** GCP Secret Manager / AWS SSM for runtime secrets, GitHub Actions encrypted secrets for CI, push protection blocks committed credentials. AI coding agents get scoped fine-grained PATs or app tokens, never a founder's personal credentials.
- **Network boundary.** Default-deny firewall posture: production ingress only via the load balancer; SSH via IAP/SSM (or nothing at all); databases private-IP only.
- **Laptops.** Full-disk encryption (FileVault) verified via lightweight MDM, or a signed per-device attestation with a screenshot if MDM is genuinely not feasible yet.
- **Write it down.** A one-page access control policy naming the asset inventory, the identity plane, and the rules above. The auditor needs the policy to test the control against.

## Automated shadow checks

| Check | Source | Method |
|---|---|---|
| GitHub org 2FA enforcement enabled | GitHub | `gh api /orgs/{org}` → `two_factor_requirement_enabled == true` |
| Org base permission is `read`/`none` | GitHub | `gh api /orgs/{org}` → `default_repository_permission` |
| Branch protection on default branch (all repos) | GitHub | `gh api /repos/{org}/{repo}/branches/{default}/protection` — required reviews, status checks, no force push |
| Secret scanning + push protection enabled | GitHub | `gh api /repos/{org}/{repo}` → `security_and_analysis.secret_scanning.status`, `secret_scanning_push_protection.status` |
| No open critical secret-scanning alerts | GitHub | `gh api /orgs/{org}/secret-scanning/alerts?state=open` |
| Dependabot alerts enabled per repo | GitHub | `gh api /repos/{org}/{repo}/vulnerability-alerts` (204 = enabled) |
| No individual users with `roles/owner` | GCP | `gcloud projects get-iam-policy PROJECT --format=json` → filter `user:` members on `roles/owner` |
| No user-managed SA keys (or all < 90 days) | GCP | `gcloud iam service-accounts keys list --managed-by=user` per SA; compare `validAfterTime` |
| No firewall rules open to 0.0.0.0/0 on SSH/RDP/DB ports | GCP | `gcloud compute firewall-rules list --format=json` → source ranges + ports |
| Buckets/data stores not publicly readable | GCP | `gcloud storage buckets get-iam-policy` → no `allUsers`/`allAuthenticatedUsers` |
| Workspace MFA enrollment 100% | Google Workspace | Admin SDK Reports API `usage/users` → `accounts:is_2sv_enrolled` per user |
| Access control policy document exists | Repo | File-existence check: `policies/access-control.md` (or configured path) |
| TLS on public endpoints | Network | Scripted `curl -sI https://…` / TLS probe of configured production domains |
| Laptop disk encryption on all devices | MDM | MANUAL unless MDM API configured — otherwise collect signed attestations |
| Asset inventory current and complete | Docs | MANUAL — review inventory doc against actual cloud/SaaS footprint |

## Evidence artifacts

- `evidence/github/org-settings.json` — `gh api /orgs/{org}` export (timestamped, quarterly).
- `evidence/github/branch-protection/<repo>.json` — per-repo protection rule exports.
- `evidence/gcp/iam-policy-<project>-<date>.json` — IAM policy dumps at period start/end.
- `evidence/gcp/sa-keys-<date>.json` — service-account key inventory showing none or rotation.
- `evidence/gcp/firewall-rules-<date>.json` — firewall/security-group export.
- `evidence/workspace/mfa-report-<date>.csv` — 2SV enrollment report from Admin console.
- `policies/access-control.md` — access control policy incl. asset inventory (version-controlled).
- `evidence/endpoints/mdm-encryption-report-<date>.csv` or signed device attestations in `evidence/endpoints/attestations/`.
