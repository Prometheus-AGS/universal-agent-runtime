# ── SurrealDB 3.0 ───────────────────────────────────────────────────────────
# Single-replica StatefulSet backed by a 20 Gi SSD PVC.
# Reachable within the cluster at:
#   ws://surreal-svc.uar.svc.cluster.local:8000
# The endpoint is exported to the UAR application via the ConfigMap so the
# memory service connects to this instance instead of the embedded RocksDB.

resource "kubernetes_stateful_set" "surreal" {
  metadata {
    name      = "surreal"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "surreal"
      "app.kubernetes.io/component" = "database"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
      "app.kubernetes.io/version"   = "3"
    }
  }

  spec {
    service_name = "surreal-svc"
    replicas     = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = "surreal"
        "app.kubernetes.io/component" = "database"
      }
    }

    template {
      metadata {
        labels = {
          "app.kubernetes.io/name"      = "surreal"
          "app.kubernetes.io/component" = "database"
          "app.kubernetes.io/part-of"   = "universal-agent-runtime"
        }
      }

      spec {
        container {
          name  = "surreal"
          image = var.surreal_image

          # SurrealDB v3 start command
          command = [
            "/surreal",
            "start",
            "--bind", "0.0.0.0:8000",
            "--user", "$(SURREAL_USER)",
            "--pass", "$(SURREAL_PASS)",
            "rocksdb:/data/memory.db",
          ]

          port {
            name           = "ws"
            container_port = 8000
            protocol       = "TCP"
          }

          # ── Credentials from secret ───────────────────────────────────────
          env {
            name = "SURREAL_USER"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_surreal_credentials.metadata[0].name
                key  = "SURREAL_USER"
              }
            }
          }

          env {
            name = "SURREAL_PASS"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_surreal_credentials.metadata[0].name
                key  = "SURREAL_PASS"
              }
            }
          }

          # ── Data volume ───────────────────────────────────────────────────
          volume_mount {
            name       = "surreal-data"
            mount_path = "/data"
          }

          # ── Resource limits ───────────────────────────────────────────────
          resources {
            requests = {
              cpu    = "250m"
              memory = "512Mi"
            }
            limits = {
              cpu    = "1"
              memory = "2Gi"
            }
          }

          # ── Health checks ─────────────────────────────────────────────────
          liveness_probe {
            http_get {
              path   = "/health"
              port   = 8000
            }
            initial_delay_seconds = 30
            period_seconds        = 15
            timeout_seconds       = 5
            failure_threshold     = 5
          }

          readiness_probe {
            http_get {
              path   = "/health"
              port   = 8000
            }
            initial_delay_seconds = 10
            period_seconds        = 10
            timeout_seconds       = 3
            failure_threshold     = 3
          }
        }

        # ── Volume binding ────────────────────────────────────────────────
        volume {
          name = "surreal-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.surreal_data.metadata[0].name
          }
        }
      }
    }

    update_strategy {
      type = "RollingUpdate"
    }
  }

  depends_on = [
    kubernetes_persistent_volume_claim.surreal_data,
    kubernetes_secret.uar_surreal_credentials,
  ]
}

# ── SurrealDB ClusterIP Service ───────────────────────────────────────────────
# Internal DNS: surreal-svc.uar.svc.cluster.local:8000
resource "kubernetes_service" "surreal_svc" {
  metadata {
    name      = "surreal-svc"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "surreal"
      "app.kubernetes.io/component" = "database"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = "surreal"
      "app.kubernetes.io/component" = "database"
    }

    port {
      name        = "ws"
      port        = 8000
      target_port = 8000
      protocol    = "TCP"
    }
  }
}

# ── Surrealist Web UI ─────────────────────────────────────────────────────────
# Surrealist is a static SPA that the browser loads and then uses to connect
# DIRECTLY to a SurrealDB endpoint from the client machine. Therefore the
# SurrealDB API must also be reachable from outside the cluster — see ingress.tf
# for the surreal-api.know-me.tools WebSocket ingress.

resource "kubernetes_deployment" "surrealist" {
  metadata {
    name      = "surrealist"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "surrealist"
      "app.kubernetes.io/component" = "ui"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = "surrealist"
        "app.kubernetes.io/component" = "ui"
      }
    }

    template {
      metadata {
        labels = {
          "app.kubernetes.io/name"      = "surrealist"
          "app.kubernetes.io/component" = "ui"
          "app.kubernetes.io/part-of"   = "universal-agent-runtime"
        }
      }

      spec {
        container {
          name  = "surrealist"
          image = var.surrealist_image

          port {
            name           = "http"
            container_port = 8080
            protocol       = "TCP"
          }

          resources {
            requests = {
              cpu    = "50m"
              memory = "64Mi"
            }
            limits = {
              cpu    = "200m"
              memory = "128Mi"
            }
          }

          readiness_probe {
            http_get {
              path = "/"
              port = 8080
            }
            initial_delay_seconds = 5
            period_seconds        = 10
          }
        }
      }
    }
  }
}

# ── Surrealist ClusterIP Service ──────────────────────────────────────────────
resource "kubernetes_service" "surrealist_svc" {
  metadata {
    name      = "surrealist-svc"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "surrealist"
      "app.kubernetes.io/component" = "ui"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = "surrealist"
      "app.kubernetes.io/component" = "ui"
    }

    port {
      name        = "http"
      port        = 80
      target_port = 8080
      protocol    = "TCP"
    }
  }
}
