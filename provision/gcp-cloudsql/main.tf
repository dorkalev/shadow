# The relational variant: same compliant baseline as ../gcp, with Cloud SQL
# Postgres instead of Firestore (~$10/month — the only non-free resource).
# Choose this when the app genuinely needs SQL; otherwise ../gcp is $0.
#
# Every resource is annotated with the TSC criterion it evidences. What is
# deliberately NOT here: user-managed service-account keys, public database
# access, unencrypted transport, human deploy credentials.

locals {
  services = [
    "run.googleapis.com",
    "sqladmin.googleapis.com",
    "secretmanager.googleapis.com",
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

resource "google_service_account" "runtime" {
  account_id   = "${var.service_name}-runtime"
  display_name = "Runtime SA for ${var.service_name} (least privilege — CC6.3)"
}

resource "google_project_iam_member" "runtime_sql" {
  project = var.project_id
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.runtime.email}"
}

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

resource "google_service_account_iam_member" "deploy_actas_runtime" {
  service_account_id = google_service_account.runtime.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.deploy.email}"
}

# Workload Identity Federation: deploys can only originate from var.github_repo.
# CC6.1 / CC6.6 / CC8.1
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
# Data: Postgres with backups, PITR, and encrypted-only transport.
# A1.2 (backups/recovery) · CC6.7 (encryption in transit) · CC6.6 · CC6.5
# ---------------------------------------------------------------------------

resource "google_sql_database_instance" "db" {
  name             = "${var.service_name}-db"
  database_version = "POSTGRES_16"
  region           = var.region

  settings {
    tier = var.db_tier

    backup_configuration {
      enabled                        = true # A1.2 — automated daily backups
      point_in_time_recovery_enabled = true # A1.2 — PITR; proven by restore-test-cloudsql.yml (A1.3)
    }

    ip_configuration {
      ipv4_enabled = true
      ssl_mode     = "ENCRYPTED_ONLY" # CC6.7 — no cleartext connections, ever
      # no authorized_networks: nothing on the internet can reach this instance
      # directly; the app connects via the Cloud SQL connector (IAM-gated). CC6.6
    }
  }

  deletion_protection = true # CC6.5 — disposal is a deliberate, logged act
}

resource "google_sql_database" "app" {
  name     = var.service_name
  instance = google_sql_database_instance.db.name
}

resource "random_password" "db" {
  length  = 32
  special = false
}

resource "google_sql_user" "app" {
  name     = var.service_name
  instance = google_sql_database_instance.db.name
  password = random_password.db.result
}

# The one unavoidable secret in this variant, held in Secret Manager and
# readable by the runtime SA alone. CC6.1 — never in code, env files, or CI.
resource "google_secret_manager_secret" "database_url" {
  secret_id = "${var.service_name}-database-url"
  replication {
    auto {}
  }
  depends_on = [google_project_service.required]
}

resource "google_secret_manager_secret_version" "database_url" {
  secret = google_secret_manager_secret.database_url.id
  secret_data = format(
    "postgres://%s:%s@localhost/%s?host=/cloudsql/%s",
    google_sql_user.app.name,
    random_password.db.result,
    google_sql_database.app.name,
    google_sql_database_instance.db.connection_name,
  )
}

resource "google_secret_manager_secret_iam_member" "runtime_reads_db_url" {
  secret_id = google_secret_manager_secret.database_url.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.runtime.email}"
}

# ---------------------------------------------------------------------------
# Compute: Cloud Run.  A1.1 autoscaling; CC6.4 + environmental inherited (CSOC)
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
      min_instance_count = 0
      max_instance_count = 4 # A1.1 — capacity is managed; raise deliberately
    }

    containers {
      # placeholder until the first CI deploy replaces it (deploy.yml)
      image = "us-docker.pkg.dev/cloudrun/container/hello"

      env {
        name = "DATABASE_URL"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.database_url.secret_id
            version = "latest"
          }
        }
      }
    }

    volumes {
      name = "cloudsql"
      cloud_sql_instance {
        instances = [google_sql_database_instance.db.connection_name]
      }
    }
  }

  depends_on = [google_project_service.required]

  lifecycle {
    # CI owns the image; terraform owns the shape. CC8.1
    ignore_changes = [template[0].containers[0].image]
  }
}

resource "google_cloud_run_v2_service_iam_member" "public" {
  name     = google_cloud_run_v2_service.app.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# ---------------------------------------------------------------------------
# Observability.  CC7.2 · CC2.1 · CC4.2
# ---------------------------------------------------------------------------

resource "google_project_iam_audit_config" "sql" {
  project = var.project_id
  service = "sqladmin.googleapis.com"
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
  retention_days = var.log_retention_days
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
