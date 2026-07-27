# The bare-minimum SOC 2-compliant backend, as code — 100% always-free-tier eligible.
#
# One Cloud Run service + Firestore + keyless CI deploys. Every resource is
# annotated with the TSC criterion it evidences — this file is itself CC8.1
# evidence (infrastructure changes are authorized, documented, reviewed, and
# applied from version control).
#
# What is deliberately NOT here:
#   - user-managed service-account keys (none exist anywhere in this design)
#   - connection strings or database passwords (Firestore auth is IAM-native —
#     there is literally no datastore secret to store, leak, or rotate)
#   - unencrypted transport (Firestore and Cloud Run are TLS-only by construction)
#   - human deploy credentials

locals {
  services = [
    "run.googleapis.com",
    "firestore.googleapis.com",
    "secretmanager.googleapis.com", # for app secrets (API keys etc.) — pattern below
    "artifactregistry.googleapis.com",
    "iamcredentials.googleapis.com",
    "sts.googleapis.com",
    "monitoring.googleapis.com",
    "logging.googleapis.com",
  ]
}

resource "google_project_service" "required" {
  for_each           = toset(local.services)
  service            = each.value
  disable_on_destroy = false
}

# ---------------------------------------------------------------------------
# Identity: two service accounts, least privilege, zero keys.  CC6.1 / CC6.3
# ---------------------------------------------------------------------------

# Runtime identity — what the app runs as. Can read/write Firestore, nothing else.
resource "google_service_account" "runtime" {
  account_id   = "${var.service_name}-runtime"
  display_name = "Runtime SA for ${var.service_name} (least privilege — CC6.3)"
}

resource "google_project_iam_member" "runtime_firestore" {
  project = var.project_id
  role    = "roles/datastore.user"
  member  = "serviceAccount:${google_service_account.runtime.email}"
}

# Deploy identity — what GitHub Actions assumes via Workload Identity
# Federation. No exported key ever exists (CC6.1: no long-lived credentials;
# CC8.1: production deploys can only originate from this repo's CI).
resource "google_service_account" "deploy" {
  account_id   = "${var.service_name}-deploy"
  display_name = "CI deploy SA for ${var.service_name} (WIF-only, keyless)"
}

resource "google_project_iam_member" "deploy_roles" {
  for_each = toset(["roles/run.admin", "roles/artifactregistry.writer"])
  project  = var.project_id
  role     = each.value
  member   = "serviceAccount:${google_service_account.deploy.email}"
}

# Deploy SA may act as the runtime SA (required to deploy the service) — and
# only as that one, not project-wide.
resource "google_service_account_iam_member" "deploy_actas_runtime" {
  service_account_id = google_service_account.runtime.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.deploy.email}"
}

# Workload Identity Federation: trust GitHub's OIDC issuer, but ONLY tokens
# minted for var.github_repo.  CC6.1 / CC6.6 — the deploy door is one repo wide.
resource "google_iam_workload_identity_pool" "github" {
  workload_identity_pool_id = "github"
  display_name              = "GitHub Actions"
  depends_on                = [google_project_service.required]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github-oidc"
  display_name                       = "GitHub OIDC"
  attribute_mapping = {
    "google.subject"       = "assertion.sub"
    "attribute.repository" = "assertion.repository"
  }
  attribute_condition = "assertion.repository == \"${var.github_repo}\""
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

resource "google_service_account_iam_member" "deploy_wif" {
  service_account_id = google_service_account.deploy.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"
}

# ---------------------------------------------------------------------------
# Data: Firestore with PITR, daily backups, and delete protection.
# A1.2 (backups/recovery) · CC6.7 (TLS-only by construction) · CC6.5 (disposal
# is deliberate) · zero connection secrets (CC6.1)
# ---------------------------------------------------------------------------

resource "google_firestore_database" "app" {
  name        = "(default)"
  location_id = var.region
  type        = "FIRESTORE_NATIVE"

  point_in_time_recovery_enablement = "POINT_IN_TIME_RECOVERY_ENABLED" # A1.2 — 7-day PITR
  delete_protection_state           = "DELETE_PROTECTION_ENABLED"      # CC6.5 — disposal is a deliberate, logged act

  depends_on = [google_project_service.required]
}

# Daily managed backups; restore-test.yml proves them quarterly (A1.3).
resource "google_firestore_backup_schedule" "daily" {
  database  = google_firestore_database.app.name
  retention = format("%ds", var.backup_retention_days * 86400)
  daily_recurrence {}
}

# App secrets pattern (API keys, webhook URLs, …): one secret per credential,
# runtime SA granted per-secret access — never project-wide. CC6.1 / CC6.3
# resource "google_secret_manager_secret" "example" {
#   secret_id = "${var.service_name}-example"
#   replication { auto {} }
# }
# resource "google_secret_manager_secret_iam_member" "runtime_reads_example" {
#   secret_id = google_secret_manager_secret.example.id
#   role      = "roles/secretmanager.secretAccessor"
#   member    = "serviceAccount:${google_service_account.runtime.email}"
# }

# ---------------------------------------------------------------------------
# Compute: Cloud Run. Autoscaling handles A1.1; the platform's SOC 2 covers
# CC6.4 physical + environmental (carved out as a subservice organization).
# ---------------------------------------------------------------------------

resource "google_artifact_registry_repository" "app" {
  repository_id = var.service_name
  format        = "DOCKER"
  location      = var.region
  depends_on    = [google_project_service.required]
}

resource "google_cloud_run_v2_service" "app" {
  name     = var.service_name
  location = var.region
  ingress  = "INGRESS_TRAFFIC_ALL"

  template {
    service_account = google_service_account.runtime.email

    scaling {
      min_instance_count = 0 # scale-to-zero keeps the always-free tier real
      max_instance_count = 4 # A1.1 — capacity is managed; raise deliberately
    }

    containers {
      # placeholder until the first CI deploy replaces it (deploy.yml).
      # Firestore clients need no env: project + credentials come from the
      # metadata server under the runtime SA.
      image = "us-docker.pkg.dev/cloudrun/container/hello"
    }
  }

  depends_on = [google_project_service.required]

  lifecycle {
    # CI owns the image; terraform owns the shape. CC8.1 — config changes are
    # reviewed here, code changes deploy through the gated pipeline.
    ignore_changes = [template[0].containers[0].image]
  }
}

# Public app (remove for internal-only services).
resource "google_cloud_run_v2_service_iam_member" "public" {
  name     = google_cloud_run_v2_service.app.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# ---------------------------------------------------------------------------
# Observability: audit logs retained, uptime watched, humans alerted.
# CC7.2 (monitoring/anomalies) · CC2.1 (quality information) · CC4.2
# ---------------------------------------------------------------------------

# Data-access audit logs for the datastore: who wrote what, and admin reads.
# (Firestore audit logs surface under the datastore.googleapis.com service.)
resource "google_project_iam_audit_config" "firestore" {
  project = var.project_id
  service = "datastore.googleapis.com"
  audit_log_config {
    log_type = "ADMIN_READ"
  }
  audit_log_config {
    log_type = "DATA_WRITE"
  }
}

resource "google_project_iam_audit_config" "secrets" {
  project = var.project_id
  service = "secretmanager.googleapis.com"
  audit_log_config {
    log_type = "ADMIN_READ"
  }
  audit_log_config {
    log_type = "DATA_READ" # secret reads are the interesting event
  }
}

resource "google_logging_project_bucket_config" "default_retention" {
  project        = var.project_id
  location       = "global"
  bucket_id      = "_Default"
  retention_days = var.log_retention_days # CC7.2 — evidence survives the audit window
}

resource "google_monitoring_notification_channel" "email" {
  display_name = "shadow alerts"
  type         = "email"
  labels = {
    email_address = var.alert_email
  }
  depends_on = [google_project_service.required]
}

resource "google_monitoring_uptime_check_config" "app" {
  display_name = "${var.service_name} uptime"
  timeout      = "10s"
  period       = "300s"

  http_check {
    path         = "/"
    port         = 443
    use_ssl      = true
    validate_ssl = true
  }

  monitored_resource {
    type = "uptime_url"
    labels = {
      project_id = var.project_id
      host       = replace(google_cloud_run_v2_service.app.uri, "https://", "")
    }
  }
}

# Alert the humans when the uptime check fails. CC7.2 → CC7.3 handoff:
# this alert is where "anomaly" becomes "evaluated event".
resource "google_monitoring_alert_policy" "uptime" {
  display_name = "${var.service_name} down"
  combiner     = "OR"

  conditions {
    display_name = "uptime check failing"
    condition_threshold {
      filter          = "metric.type=\"monitoring.googleapis.com/uptime_check/check_passed\" AND resource.type=\"uptime_url\" AND metric.labels.check_id=\"${google_monitoring_uptime_check_config.app.uptime_check_id}\""
      comparison      = "COMPARISON_GT"
      threshold_value = 0   # any failing check in the window trips the alert
      duration        = "300s"
      aggregations {
        alignment_period     = "300s"
        per_series_aligner   = "ALIGN_NEXT_OLDER"
        cross_series_reducer = "REDUCE_COUNT_FALSE"
        group_by_fields      = ["resource.label.host"]
      }
      trigger {
        count = 1
      }
    }
  }

  notification_channels = [google_monitoring_notification_channel.email.id]
}
