---
description: One-command provisioning of the SOC 2-compliant GCP baseline — Terraform apply with a short interview, then keyless deploys forever
---
# /shadow:provision — Stand Up the Compliant Backend

Provisions the bare-minimum compliant runtime (`provision/gcp/`): Cloud Run +
Firestore (PITR + daily backups, IAM-native auth — zero database secrets) +
two least-privilege service accounts + Workload Identity Federation + audit
logs + uptime alerting. 100% always-free-tier eligible at pilot scale.
This is the ONE step that needs an authenticated human — WIF cannot be
bootstrapped from nothing. Every deploy afterward is keyless CI.

## Phase 1 — GATHER (zero questions)

```bash
gcloud auth list --filter=status:ACTIVE --format='value(account)'   # must be a human, logged in
gcloud projects list --format='table(projectId,name)'               # candidate projects
gh repo view --json nameWithOwner -q .nameWithOwner                 # the WIF trust boundary
terraform -version                                                  # >= 1.5 required
```

STOP if: no active gcloud login (`gcloud auth login` + `gcloud auth application-default login` first), or terraform missing.

## Phase 2 — INTERVIEW (judgment only)

Ask, with the gathered facts as defaults:
1. **Project** — which existing project ID (offer the list; creating projects/billing is out of scope here — the human does that in the console once).
2. **Region** — default `us-central1`.
3. **Alert email** — where "the app is down" goes (CC7.2). No default; must be a monitored inbox.
4. **Service name** — default `app`.
4b. **Datastore** — Firestore (`provision/gcp`, $0, secretless — default) or Cloud SQL Postgres (`provision/gcp-cloudsql`, ~$10/month) — decided by the app's data model, not cost. On SQL, also swap the restore drill for `gcp-cloudsql/restore-test-cloudsql.yml`.
5. Confirm the plan summary in one line: "Cloud Run + Firestore (PITR + daily backups, delete-protected) + WIF for {repo} + audit logs + uptime alert to {email} in {project} — $0 at pilot scale. Apply?" — the human must answer **apply**.

## Phase 3 — FILE (zero questions)

```bash
cd provision/gcp   # or provision/gcp-cloudsql per the datastore answer
terraform init
terraform plan -out=tfplan \
  -var project_id={project} -var region={region} \
  -var github_repo={owner/name} -var alert_email={email} -var service_name={service}
# show the plan resource count; then
terraform apply tfplan
```

Then wire the repo (all plain variables — nothing here is secret; that is the point of WIF):

```bash
terraform output -raw wif_provider           | xargs -I{} gh variable set GCP_WIF_PROVIDER --body {}
terraform output -raw deploy_service_account | xargs -I{} gh variable set GCP_DEPLOY_SA --body {}
terraform output -raw artifact_repo          | xargs -I{} gh variable set GCP_ARTIFACT_REPO --body {}
gh variable set GCP_REGION --body {region}
gh variable set GCP_SERVICE --body {service}
```

Finish:
1. Update `quarterly-rituals.yml`'s `GCP_PROJECTS` and `restore-test.yml`'s `GCP_PROJECT`/`LOCATION` so the shadow watches what was just built.
2. Commit `provision/gcp/.terraform.lock.hcl` and any tfvars **through the normal PR flow** (the provisioning itself is a CC8.1 change; state files are never committed).
3. Report: every resource created, the service URL, and the one-line audit story — "runtime is two subservice organizations (GCP, GitHub), zero long-lived credentials, deploys only from {repo}@main via CI."

STOP conditions: plan shows destroys against an existing environment (present and require explicit human approval of each destroy); apply errors (report verbatim, never retry destructive operations blind).
