# After `terraform apply`, set these as GitHub repo variables (not secrets —
# none of them are sensitive; that is the point of WIF).

output "service_url" {
  value       = google_cloud_run_v2_service.app.uri
  description = "The app URL (uptime-checked)"
}

output "wif_provider" {
  value       = google_iam_workload_identity_pool_provider.github.name
  description = "Set as repo variable GCP_WIF_PROVIDER — used by deploy.yml's auth step"
}

output "deploy_service_account" {
  value       = google_service_account.deploy.email
  description = "Set as repo variable GCP_DEPLOY_SA"
}

output "artifact_repo" {
  value       = "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.app.repository_id}"
  description = "Where CI pushes images"
}

output "sql_connection_name" {
  value       = google_sql_database_instance.db.connection_name
  description = "For restore-test-cloudsql.yml (SOURCE_INSTANCE) and local proxy use"
}

output "shadow_env" {
  value       = "GCP_PROJECTS=${var.project_id}  # add to quarterly-rituals.yml so access reviews cover this project"
  description = "Reminder: point the shadow at this project"
}
