# SSD-backed PersistentVolumeClaims for all UAR services.
# GKE's "premium-rwo" StorageClass provisions zonal SSD Persistent Disks.
# All volumes are ReadWriteOnce — appropriate for single-replica workloads.

locals {
  ssd_storage_class = "premium-rwo"
  uar_namespace     = kubernetes_namespace.uar.metadata[0].name
}

# ── PostgreSQL data ─────────────────────────────────────────────────────────
resource "kubernetes_persistent_volume_claim" "postgres_data" {
  metadata {
    name      = "postgres-data-pvc"
    namespace = local.uar_namespace

    labels = {
      "app.kubernetes.io/component" = "postgres"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    access_modes       = ["ReadWriteOnce"]
    storage_class_name = local.ssd_storage_class

    resources {
      requests = {
        storage = "20Gi"
      }
    }
  }

  # Prevent accidental deletion of database data during tofu destroy.
  lifecycle {
    prevent_destroy = true
  }
}

# ── Redis data ──────────────────────────────────────────────────────────────
resource "kubernetes_persistent_volume_claim" "redis_data" {
  metadata {
    name      = "redis-data-pvc"
    namespace = local.uar_namespace

    labels = {
      "app.kubernetes.io/component" = "redis"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    access_modes       = ["ReadWriteOnce"]
    storage_class_name = local.ssd_storage_class

    resources {
      requests = {
        storage = "5Gi"
      }
    }
  }
}

# ── UAR file uploads ────────────────────────────────────────────────────────
# Mounted at /uploads inside the UAR container.
# Holds files uploaded during chat sessions (chat_attachments) and
# documents ingested into knowledge bases.
resource "kubernetes_persistent_volume_claim" "uar_uploads" {
  metadata {
    name      = "uar-uploads-pvc"
    namespace = local.uar_namespace

    labels = {
      "app.kubernetes.io/component" = "uar"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    access_modes       = ["ReadWriteOnce"]
    storage_class_name = local.ssd_storage_class

    resources {
      requests = {
        storage = "10Gi"
      }
    }
  }
}

# ── SurrealDB data ───────────────────────────────────────────────────────────
# 20 Gi SSD volume for the SurrealDB 3.0 RocksDB storage backend.
resource "kubernetes_persistent_volume_claim" "surreal_data" {
  metadata {
    name      = "surreal-data-pvc"
    namespace = local.uar_namespace

    labels = {
      "app.kubernetes.io/component" = "surreal"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    access_modes       = ["ReadWriteOnce"]
    storage_class_name = local.ssd_storage_class

    resources {
      requests = {
        storage = "20Gi"
      }
    }
  }

  # Prevent accidental deletion of memory data during tofu destroy.
  lifecycle {
    prevent_destroy = true
  }
}

# ── UAR runtime data ─────────────────────────────────────────────────────────
# Mounted at /data inside the UAR container.
# Holds the local memory DB, ingest staging, and skills/policies directories.
resource "kubernetes_persistent_volume_claim" "uar_data" {
  metadata {
    name      = "uar-data-pvc"
    namespace = local.uar_namespace

    labels = {
      "app.kubernetes.io/component" = "uar"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    access_modes       = ["ReadWriteOnce"]
    storage_class_name = local.ssd_storage_class

    resources {
      requests = {
        storage = "5Gi"
      }
    }
  }
}
