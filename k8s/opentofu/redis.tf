# Redis — used by UAR for external caching and rate-limit state.
# Single-replica; data is persisted to an SSD PVC.

resource "kubernetes_deployment" "redis" {
  metadata {
    name      = "redis"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "redis"
      "app.kubernetes.io/component" = "cache"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = "redis"
        "app.kubernetes.io/component" = "cache"
      }
    }

    strategy {
      # Recreate avoids two replicas competing for the same RWO PVC.
      type = "Recreate"
    }

    template {
      metadata {
        labels = {
          "app.kubernetes.io/name"      = "redis"
          "app.kubernetes.io/component" = "cache"
          "app.kubernetes.io/part-of"   = "universal-agent-runtime"
        }
      }

      spec {
        container {
          name  = "redis"
          image = "redis:7-alpine"

          # Enable AOF persistence so data survives pod restarts.
          args = ["redis-server", "--appendonly", "yes", "--dir", "/data"]

          port {
            name           = "redis"
            container_port = 6379
            protocol       = "TCP"
          }

          volume_mount {
            name       = "redis-data"
            mount_path = "/data"
          }

          resources {
            requests = {
              cpu    = "100m"
              memory = "128Mi"
            }
            limits = {
              cpu    = "500m"
              memory = "512Mi"
            }
          }

          liveness_probe {
            exec {
              command = ["redis-cli", "ping"]
            }
            initial_delay_seconds = 15
            period_seconds        = 10
            timeout_seconds       = 3
            failure_threshold     = 3
          }

          readiness_probe {
            exec {
              command = ["redis-cli", "ping"]
            }
            initial_delay_seconds = 5
            period_seconds        = 5
            timeout_seconds       = 2
            failure_threshold     = 3
          }
        }

        volume {
          name = "redis-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.redis_data.metadata[0].name
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_persistent_volume_claim.redis_data,
  ]
}

# ── ClusterIP Service ──────────────────────────────────────────────────────
# Reachable as redis-svc.uar.svc.cluster.local:6379
resource "kubernetes_service" "redis_svc" {
  metadata {
    name      = "redis-svc"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "redis"
      "app.kubernetes.io/component" = "cache"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = "redis"
      "app.kubernetes.io/component" = "cache"
    }

    port {
      name        = "redis"
      port        = 6379
      target_port = 6379
      protocol    = "TCP"
    }
  }
}
