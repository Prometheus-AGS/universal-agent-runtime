# SSD-backed PersistentVolumeClaims for all UAR services.
# All PVCs use the Immediate-binding StorageClass (premium-rwo-immediate).
#
# GKE's built-in "premium-rwo" uses WaitForFirstConsumer, which deadlocks
# for standalone PVCs: the provisioner waits for a pod to pick a zone, but
# the pod waits for the PVC to bind first. Immediate binding provisions the
# disk at PVC creation time, breaking the deadlock for all workloads.

locals {
  ssd_storage_class = "premium-rwo-immediate"
  uar_namespace     = kubernetes_namespace.uar.metadata[0].name
}

# ── StorageClass: Immediate SSD ──────────────────────────────────────────────
resource "kubernetes_storage_class" "premium_rwo_immediate" {
  metadata {
    name = local.ssd_storage_class

    labels = {
      "app.kubernetes.io/part-of" = "universal-agent-runtime"
    }
  }

  storage_provisioner    = "pd.csi.storage.gke.io"
  volume_binding_mode    = "Immediate"
  reclaim_policy         = "Retain"
  allow_volume_expansion = true

  parameters = {
    type = "pd-ssd"
  }
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

  depends_on = [kubernetes_storage_class.premium_rwo_immediate]

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

  depends_on = [kubernetes_storage_class.premium_rwo_immediate]
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

  depends_on = [kubernetes_storage_class.premium_rwo_immediate]
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

  depends_on = [kubernetes_storage_class.premium_rwo_immediate]

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

  depends_on = [kubernetes_storage_class.premium_rwo_immediate]
}
