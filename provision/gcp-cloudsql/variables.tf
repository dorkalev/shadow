variable "project_id" {
  description = "GCP project to provision (must already exist, with billing enabled)"
  type        = string
}

variable "region" {
  description = "Region for all resources"
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

variable "db_tier" {
  description = "Cloud SQL tier. db-f1-micro is the cheapest for pilots (~$10/month); shared-core tiers carry no SLA — move to db-custom-* before availability commitments (A1.1)"
  type        = string
  default     = "db-f1-micro"
}

variable "log_retention_days" {
  description = "Retention for the _Default log bucket (CC7.2 / CC2.1 evidence window)"
  type        = number
  default     = 90
}
