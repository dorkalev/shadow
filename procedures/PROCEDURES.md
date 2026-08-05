# Procedures — the machinery of a compliant-able project

This is the install list: every agent, workflow, webhook, scanner, and register that turns an ordinary repo into a project that generates SOC 2 evidence as a side effect of working. Runbook 02 installs them; runbook 03 re-detects them on every run; the website renders this table with live status.

Statuses: `not_installed` → `installed` (runbook 02 completed the step) → `verified` (the Detect check passed on the latest shadow audit). A procedure whose Detect check fails after being installed regresses to `failing`.

Format contract: the website seeder parses the tables below. Columns: ID | Name | Category | Criteria served | Install | Detect.

## GitHub organization

| ID | Name | Category | Criteria served | Install | Detect |
|---|---|---|---|---|---|
| org-2fa | Org-wide 2FA requirement | github-org | CC6.1, CC6.2 | runbook 02 §1 | `gh api /orgs/{org}` → `two_factor_requirement_enabled` |
| org-base-perms | Default repo permission = read, no public repo creation | github-org | CC6.3 | runbook 02 §1 | `gh api /orgs/{org}` → `default_repository_permission` |
| workspace-mfa | Google Workspace 2SV enforcement | identity | CC6.1 | runbook 02 §1 | Admin SDK 2SV enrollment report |

## Repo gates (per repo)

| ID | Name | Category | Criteria served | Install | Detect |
|---|---|---|---|---|---|
| branch-rulesets | Rulesets on main (PR + required checks, no force/deletion) and compliance-archives (no force/deletion) | repo-gates | CC8.1, CC6.3 | runbook 02 §2 | `gh api /repos/{org}/{repo}/rulesets` contains both and required checks remain enforced |
| main-topology | main is the sole production trunk; all normal change PRs target it | repo-gates | CC8.1 | runbook 02 §2 | default branch + recent PR bases + ruleset |
| pr-template | PR template: Summary / Tickets / Changes / Test Plan | repo-gates | CC8.1 | runbook 02 §3 | `.github/pull_request_template.md` exists with 4 sections |
| ci-tests | CI build + test workflow on PR | repo-gates | CC8.1 | runbook 02 §3 | workflow file + recent green runs on PRs |
| compliance-audit-agent | Compliance audit check (awaiting-review phase, `shadow-ci check`) | agents | CC8.1, CC4.1 | runbook 02 §3 | `compliance.yml` workflow + `shadow-ci:audit` comment on latest PR |
| compliance-review-gate | Review gate check (post-review phase, required reviewer + findings gate) | agents | CC8.1, CC4.1 | runbook 02 §3 | `compliance.yml` workflow + `compliance-review-gate` required context in ruleset |
| review-bot | Optional semantic adviser: built-in shadow-reviewer (review.yml, LLM) or a third-party bot | agents | CC8.1 | review.yml / bot app (optional) | completed marker or bot review present when the control is enabled |
| post-merge-archive | Post-merge archive → compliance-archives branch (JSON+MD per PR) | evidence | CC8.1, CC4.1, CC2.1 | runbook 02 §3 | workflow file + archive record for every merged PR since install |
| bypass-detection | Bypass-merge detection against live branch ruleset (+ Slack alert) | evidence | CC8.1, CC4.1 | runbook 02 §3 | `bypass_merge` block present in latest archive records |
| archives-branch | protected compliance-archives branch (new evidence records; no force/deletion) | evidence | CC8.1 | runbook 02 §2 | branch exists with protective ruleset and README |

## Scanners

| ID | Name | Category | Criteria served | Install | Detect |
|---|---|---|---|---|---|
| dependabot | Dependabot alerts + security updates | scanners | CC7.1 | runbook 02 §4 | `gh api /repos/{org}/{repo}/dependabot/alerts` returns 200 |
| secret-scanning | Secret scanning + push protection | scanners | CC6.1, CC7.1 | runbook 02 §4 | repo security settings via API |
| codeql | CodeQL default setup (supported languages) | scanners | CC7.1 | runbook 02 §4 | `/code-scanning/default-setup` state |

## Paper layer

| ID | Name | Category | Criteria served | Install | Detect |
|---|---|---|---|---|---|
| policies-repo | Private policies repo, 14-policy pack, frontmatter lifecycle | paper | CC5.3, CC1.1, CC2.2 | runbook 02 §5 | repo exists; all 14 files; `review_by` dates current |
| risk-register | risk-register.md (likelihood×impact, treatment, owner) | paper | CC3.1–CC3.4, CC9.1 | runbook 02 §5 | file exists; touched ≤ 12 months |
| vendor-register | vendor-register.md incl. LLM providers, with review decisions | paper | CC9.2 | runbook 02 §5 | file exists; reviews ≤ 12 months |
| access-register | access-register.md (people × systems × roles) | paper | CC6.2, CC6.3 | runbook 02 §5 | file exists; matches live member lists |
| onboard-offboard | Onboarding + offboarding runbooks with checklist tickets | paper | CC6.2, CC1.4 | runbook 02 §5 | runbook files exist; ticket per join/leave |
| incident-runbook | Incident response runbook | paper | CC7.3–CC7.5 | runbook 02 §5 | runbook file exists; incidents have tickets + postmortems |
| hotfix-runbook | Hotfix procedure: documented emergency bypass (incident ticket + remediation PR through normal gates) | paper | CC8.1, CC7.4 | runbook 02 §5 | `runbooks/hotfix.md` exists; every bypass has a linked incident ticket + remediation PR |
| ai-policy | AI Development & Agent Use Policy (machine identities disclosed, no AI counted as a person, prompt-on-ticket) | paper | CC8.1, CC6.3, CC9.2 | runbook 02 §5 | policy file exists, attested |

## Monitoring & cloud

| ID | Name | Category | Criteria served | Install | Detect |
|---|---|---|---|---|---|
| uptime-alerting | Uptime checks + alerting policy | monitoring | CC7.2, A1.1 | provision/gcp Terraform | `gcloud monitoring uptime list-configs` non-empty |
| error-monitoring | Error tracking (Sentry or Cloud Error Reporting) | monitoring | CC7.2 | manual/cloud | DSN in config or API check |
| audit-logging | Cloud audit logs retained; app-level audit_events for privileged actions | monitoring | CC7.2, CC2.1 | provision/gcp Terraform + app code | project audit configs + log-bucket retention ≥ 90d; audit_events table exists |
| backups-pitr | Automated backups + PITR on primary datastore | cloud | A1.2 | provision/gcp Terraform | `gcloud firestore databases describe` → PITR enabled + backup schedule present (SQL stacks: `sql instances describe`) |
| restore-test | Automated restore-from-backup test (throwaway clone, evidence logged) | cloud | A1.3 | restore-test.yml | `evidence/**/restore-test*` younger than 12 months |
| slack-webhook | Slack notifications: merges, bypasses, shadow regressions | monitoring | CC2.2, CC4.2 | runbook 02 §3/§6 | webhook secret set; recent delivery |

## Cadence

| ID | Name | Category | Criteria served | Install | Detect |
|---|---|---|---|---|---|
| daily-verify | Daily shadow audit cron (runbook 03) writing gauge + tickets | cadence | CC4.1 | runbook 02 §6 | gauge_history row < 48h old |
| quarterly-access-review | Quarterly access review: packet auto-generated by `shadow-ci access-review`, human signs | cadence | CC6.2, CC6.3 | quarterly-rituals.yml | `evidence/{YYYY}/{QN}/access-review*` present with sign-off filled |
| quarterly-mgmt-review | Quarterly management review: packet auto-generated by `shadow-ci mgmt-packet`, humans meet + fill minutes | cadence | CC1.2, CC4.2 | quarterly-rituals.yml | `evidence/{YYYY}/{QN}/management-review*` present with minutes filled |
| annual-rituals | Annual: risk refresh, policy re-approval + staff attestation, incident tabletop | cadence | CC3.x, CC5.3, CC7.5 | runbook 02 §6 | evidence files younger than 12 months |
