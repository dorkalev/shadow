# Agent Runbook 01 — Scan the Platform (read-only)

You are the compliance shadow's discovery agent. Your job: build a complete, factual picture of the company's current posture against the 61 criteria in [`../CHECKLIST.md`](../CHECKLIST.md) — **without changing anything**. Every command in this runbook is read-only. Output: `shadow/scan-report.md` + `shadow/scan.json` in the target repo (or a directory the user names).

## Ground rules

- Read-only. No writes to GitHub settings, cloud resources, or IAM. If a command would mutate, don't run it.
- Record what you *couldn't* check (missing permissions, absent tooling) as `unknown`, never guess.
- Every finding carries: criterion ID(s), evidence (the actual command output, trimmed), and a verdict: `pass` / `fail` / `partial` / `unknown` / `n/a`.
- Ask the user up front: which cloud (gcloud/aws/other), which org/repos are in scope, which tracker (Linear/GitHub Issues), and which TSC categories they intend to scope (default: Security + Availability + Confidentiality).

## Phase 1 — Identity & source control (CC6.x, CC8.1)

```bash
gh auth status
gh api /user
gh api /orgs/{org}                              # note: two_factor_requirement_enabled  → CC6.1/6.2
gh api /orgs/{org}/members --paginate           # member inventory → access register seed
gh api /orgs/{org}/members --paginate -f role=admin   # admin count → CC6.3 least privilege
gh repo list {org} --limit 200 --json name,visibility,defaultBranchRef,pushedAt
# For each in-scope repo:
gh api /repos/{org}/{repo}/rulesets             # branch rulesets → CC8.1
gh api /repos/{org}/{repo}/branches/{default}/protection  # legacy protection (404 = none)
gh api /repos/{org}/{repo}/collaborators --paginate       # outside collaborators → CC6.2
gh api /repos/{org}/{repo}/dependabot/alerts --paginate   # → CC7.1  (403 = not enabled: finding)
gh api /repos/{org}/{repo}/secret-scanning/alerts         # → CC6.1/CC7.1
gh api /repos/{org}/{repo}/code-scanning/alerts           # CodeQL → CC7.1
gh api /repos/{org}/{repo}/actions/workflows              # existing CI → CC8.1
gh api "/repos/{org}/{repo}/pulls?state=closed&per_page=30"  # sample: were merges reviewed? → CC8.1
gh api /repos/{org}/{repo}/deploy-keys                    # stray deploy keys → CC6.1
```

From the PR sample, compute: % merged with ≥1 approval/review, % referencing a ticket ID in title/body, % with failing or absent checks at merge (bypass rate). These three numbers are the headline of the report.

## Phase 2 — Cloud (CC6.x, CC7.x, A1.x)

GCP (adapt equivalents for AWS):

```bash
gcloud projects list
gcloud projects get-iam-policy {project} --format=json    # bindings → CC6.3 (flag: owner/editor on users, allUsers)
gcloud iam service-accounts list --project {project}
gcloud iam service-accounts keys list --iam-account {sa}  # user-managed keys + age → CC6.1
gcloud compute firewall-rules list --format=json          # 0.0.0.0/0 on non-80/443 → CC6.6
gcloud sql instances describe {instance}                  # backupConfiguration, PITR, requireSsl → A1.2, CC6.7
gcloud storage buckets list --format=json                 # publicAccessPrevention, uniformBucketLevelAccess → CC6.1
gcloud logging sinks list                                  # audit log retention → CC7.2
gcloud monitoring uptime list-configs                      # uptime checks → CC7.2 / A1.1
gcloud alpha monitoring policies list                      # alerting → CC7.2
```

## Phase 3 — Workspace / identity provider (CC6.1, CC6.2)

If Google Workspace admin API access is available: user list, 2SV enrollment report, admin role assignments. If not: mark `unknown` and add to the manual attestation list — do not skip silently.

## Phase 4 — The paper layer (CC1.x, CC2.x, CC3.x, CC5.3, CC9.x)

Look for what exists, wherever it exists (repo, Drive export, Notion export, an existing GRC tool's export):

- Policies (the canon is ~11–27 docs; see `../policies/README.md`): security policy, access control, secure development, incident response, BC/DR, data classification/retention, vendor management, code of conduct, password/encryption. Note version, owner, last-approved date, employee-attestation trail.
- Risk register (any format). Note last-touched date.
- Vendor register with review decisions.
- Asset/access inventory.
- Incident log + postmortems.
- System description (SOC 2 Section III) draft, if any.
- Org chart / roles doc, onboarding & offboarding checklists, security-training evidence.

## Phase 5 — SDLC gap vs the dictated SDLC

Diff reality against [`../sdlc/SDLC.md`](../sdlc/SDLC.md), element by element: ticket-first? staging/main topology? PR gates? compliance agent? archives branch? release records? hotfix procedure? clock rituals (access reviews, management reviews) with evidence?

## Output

`shadow/scan-report.md`:
1. **Headline numbers** — review rate, ticket-traceability rate, bypass rate, org MFA on/off, gauge estimate.
2. **Per-criterion table** — all in-scope criteria: verdict + one-line evidence + link to detail.
3. **Top 10 gaps** ranked by (weight × distance), each with the concrete fix and which runbook (02/03/04) closes it.
4. **Manual attestation list** — everything automation can't see.

`shadow/scan.json`: machine-readable version, one record per check: `{criterion, check, verdict, evidence, ts}` — this seeds the website's SQLite (`website/SPEC.md`).

Then stop. Do not fix anything — that's Runbook 02, and it runs only after the user reviews this report.
