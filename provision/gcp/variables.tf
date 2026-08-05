variable "project_id" {
  description = "GCP project to provision (must already exist, with billing enabled)"
  type        = string
}

variable "region" {
  description = "Region for all resources (us-central1 keeps Cloud Run in the always-free tier)"
  type        = string
  default     = "us-central1"
}

variable "github_repo" {
  description = "GitHub repo allowed to deploy, as owner/name — the WIF trust boundary (CC8.1: deploys come only from this repo's CI)"
  type        = string
}

variable "alert_email" {
  description = "Where uptime/alerting notifications go (CC7.2)"
  type        = string
}

variable "service_name" {
  description = "Cloud Run service name"
  type        = string
  default     = "app"
}

variable "public_invoker" {
  description = "Expose Cloud Run to unauthenticated callers. Keep false unless the application enforces authentication and resource-level authorization itself."
  type        = bool
  default     = false
}

variable "backup_retention_days" {
  description = "Retention for daily Firestore backups (A1.2 evidence window; max 98)"
  type        = number
  default     = 14
}

variable "log_retention_days" {
  description = "Retention for the _Default log bucket (CC7.2 / CC2.1 evidence window)"
  type        = number
  default     = 90
}
