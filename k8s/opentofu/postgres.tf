# PostgreSQL 17 StatefulSet with pgvector + pgmq extensions.
# The custom image bakes in all UAR migrations so the database is
# fully migrated on first start — no init Job required.

resource "kubernetes_stateful_set" "postgres" {
  metadata {
    name      = "postgres"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "postgres"
      "app.kubernetes.io/component" = "database"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
      "app.kubernetes.io/version"   = "17"
    }
  }

  spec {
    service_name = "postgres-svc"
    replicas     = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = "postgres"
        "app.kubernetes.io/component" = "database"
      }
    }

    template {
      metadata {
        labels = {
          "app.kubernetes.io/name"      = "postgres"
          "app.kubernetes.io/component" = "database"
          "app.kubernetes.io/part-of"   = "universal-agent-runtime"
        }
      }

      spec {
        container {
          name  = "postgres"
          image = var.postgres_image

          port {
            name           = "postgres"
            container_port = 5432
            protocol       = "TCP"
          }

          # ── Credentials from secret ───────────────────────────────────────
          env {
            name = "POSTGRES_USER"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "POSTGRES_USER"
              }
            }
          }

          env {
            name = "POSTGRES_PASSWORD"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "POSTGRES_PASSWORD"
              }
            }
          }

          env {
            name = "POSTGRES_DB"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "POSTGRES_DB"
              }
            }
          }

          # ── Data volume ───────────────────────────────────────────────────
          volume_mount {
            name       = "postgres-data"
            mount_path = "/var/lib/postgresql/data"
            sub_path   = "pgdata"
          }

          # ── Resource limits ───────────────────────────────────────────────
          resources {
            requests = {
              cpu    = "250m"
              memory = "256Mi"
            }
            limits = {
              cpu    = "1"
              memory = "2Gi"
            }
          }

          # ── Health checks ─────────────────────────────────────────────────
          liveness_probe {
            exec {
              command = ["pg_isready", "-U", var.postgres_user, "-d", var.postgres_db]
            }
            initial_delay_seconds = 30
            period_seconds        = 10
            timeout_seconds       = 5
            failure_threshold     = 5
          }

          readiness_probe {
            exec {
              command = ["pg_isready", "-U", var.postgres_user, "-d", var.postgres_db]
            }
            initial_delay_seconds = 10
            period_seconds        = 5
            timeout_seconds       = 3
            failure_threshold     = 3
          }
        }

        # ── Volume binding ────────────────────────────────────────────────
        volume {
          name = "postgres-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.postgres_data.metadata[0].name
          }
        }
      }
    }

    update_strategy {
      type = "RollingUpdate"
    }
  }

  depends_on = [
    kubernetes_storage_class.premium_rwo_immediate,
    kubernetes_persistent_volume_claim.postgres_data,
    kubernetes_secret.uar_db_credentials,
  ]
}

# ── ClusterIP Service ─────────────────────────────────────────────────────
# Reachable within the cluster as postgres-svc.uar.svc.cluster.local:5432
resource "kubernetes_service" "postgres_svc" {
  metadata {
    name      = "postgres-svc"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "postgres"
      "app.kubernetes.io/component" = "database"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = "postgres"
      "app.kubernetes.io/component" = "database"
    }

    port {
      name        = "postgres"
      port        = 5432
      target_port = 5432
      protocol    = "TCP"
    }
  }
}
