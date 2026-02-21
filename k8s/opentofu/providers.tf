terraform {
  required_version = ">= 1.0"
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 4.84"
    }
  }

  # Backend configuration for state storage
  backend "gcs" {
    bucket = "terraform-state-prometheus-461323"
    prefix = "universal-agent-runtime"
  }
}

# Google Cloud Provider configuration
provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

# Get GKE cluster credentials
data "google_container_cluster" "primary" {
  name     = var.gke_cluster_name
  location = var.gke_cluster_location
}

# Configure Kubernetes provider to connect to GKE
provider "kubernetes" {
  host  = "https://${data.google_container_cluster.primary.endpoint}"
  token = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(
    data.google_container_cluster.primary.master_auth[0].cluster_ca_certificate
  )
}

# Get Google Cloud client configuration
data "google_client_config" "default" {}