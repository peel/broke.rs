terraform {
  required_providers {
    google = {
      source = "hashicorp/google"
      version = "4.48"
    }
    kubernetes = {
      source = "hashicorp/kubernetes"
      version = "2.16"
    }
  }
}

variable "gke_username" {}
variable "gke_password" {}
variable "service_account" {}
variable "project_id" {}

resource "random_pet" "prefix" {}

locals {
  prefix = "peel-${random_pet.prefix.id}"
  region = "europe-central2"
  zone = "${local.region}-b"
  node_count = 1
}

provider "google" {
  project = var.project_id
  region = local.region
}

resource "google_compute_firewall" "default" {
  name     = "gke-firewall"
  network  = google_compute_network.vpc.name
  allow {
    protocol = "icmp"
  }
allow {
    protocol = "tcp"
    ports    = ["22", "4222", "6222", "7422", "7522", "7777", "8222", "30000-30001"]
  }
  target_tags = ["firewall"]
  source_ranges = ["0.0.0.0/0"]
}

resource "google_compute_network" "vpc" {
  name = "${local.prefix}-vpc"
  auto_create_subnetworks = false
}
resource "google_compute_subnetwork" "subnet" {
  name = "${local.prefix}-subnet"
  region = local.region
  network = google_compute_network.vpc.name
  ip_cidr_range = "10.10.0.0/24"
}

data "google_container_engine_versions" "central2b" {
  provider       = google-beta
  location       = local.zone
  version_prefix = "1.25."
  project = var.project_id
}

resource "google_container_cluster" "primary" {
  name = "${local.prefix}-gke"
  location = local.zone
  min_master_version = data.google_container_engine_versions.central2b.latest_node_version
  remove_default_node_pool = true
  initial_node_count = 1
  logging_service = "none"
  monitoring_service = "none"

  network = google_compute_network.vpc.name
  subnetwork = google_compute_subnetwork.subnet.name

  master_auth {
    client_certificate_config {
      issue_client_certificate = false
    }
  }
}

resource "google_container_node_pool" "primary" {
  name = "${google_container_cluster.primary.name}-node-pool"
  location = local.zone
  cluster = google_container_cluster.primary.name
  node_count = local.node_count

  node_config {
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
    labels = {
      env = var.project_id
    }
    service_account = var.service_account
    preemptible = false
    machine_type = "n1-standard-1"
    tags = [ local.prefix, "gke-node" ]
  }
}

provider "kubernetes" {
  load_config_file = "false"

  host     = google_container_cluster.primary.endpoint
  username = var.gke_username
  password = var.gke_password

  client_certificate     = google_container_cluster.primary.master_auth.0.client_certificate
  client_key             = google_container_cluster.primary.master_auth.0.client_key
  cluster_ca_certificate = google_container_cluster.primary.master_auth.0.cluster_ca_certificate
}

output "k8s_cluster_name" {
  value = google_container_cluster.primary.name
}

output "k8s_cluster_region" {
  value = local.zone
}
