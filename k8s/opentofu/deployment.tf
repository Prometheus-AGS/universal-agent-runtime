# Universal Agent Runtime application Deployment.
# Environment is assembled from the ConfigMap (non-sensitive) and both
# Secrets (credentials + API keys).  File uploads and runtime data
# are stored on separate SSD PVCs.

resource "kubernetes_deployment" "uar" {
  depends_on = [
    kubernetes_stateful_set.postgres,
    kubernetes_stateful_set.surreal,
    kubernetes_deployment.redis,
    kubernetes_config_map.uar_config,
    kubernetes_secret.uar_db_credentials,
    kubernetes_secret.uar_app_secrets,
    kubernetes_secret.uar_surreal_credentials,
  ]
  metadata {
    name      = "uar"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "uar"
      "app.kubernetes.io/component" = "application"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
      "app.kubernetes.io/version"   = "02202026"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = "uar"
        "app.kubernetes.io/component" = "application"
      }
    }

    template {
      metadata {
        labels = {
          "app.kubernetes.io/name"      = "uar"
          "app.kubernetes.io/component" = "application"
          "app.kubernetes.io/part-of"   = "universal-agent-runtime"
          "app.kubernetes.io/version"   = "02202026"
        }
      }

      spec {
        # Wait for all backing services before starting the app.
        init_container {
          name  = "wait-for-postgres"
          image = "busybox:1.36"
          command = [
            "sh", "-c",
            "until nc -z postgres-svc 5432; do echo 'waiting for postgres...'; sleep 2; done",
          ]
        }

        init_container {
          name  = "wait-for-surreal"
          image = "busybox:1.36"
          command = [
            "sh", "-c",
            "until nc -z surreal-svc 8000; do echo 'waiting for surreal...'; sleep 2; done",
          ]
        }

        init_container {
          name  = "wait-for-redis"
          image = "busybox:1.36"
          command = [
            "sh", "-c",
            "until nc -z redis-svc 6379; do echo 'waiting for redis...'; sleep 2; done",
          ]
        }

        container {
          name  = "uar"
          image = "tribehealth/universal-agent-runtime:02212026.13"

          port {
            name           = "http"
            container_port = 3000
            protocol       = "TCP"
          }

          # ── Non-sensitive configuration (ConfigMap) ──────────────────────
          env_from {
            config_map_ref {
              name = kubernetes_config_map.uar_config.metadata[0].name
            }
          }

          # ── Sensitive values (Secrets) ────────────────────────────────────

          # DB credentials secret
          env {
            name = "UAR_PERSISTENCE__DATABASE_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "DATABASE_URL"
              }
            }
          }

          env {
            name = "DATABASE_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "DATABASE_URL"
              }
            }
          }

          # App secrets — iterate individual keys from uar-app-secrets
          env {
            name = "LLM_API_KEY"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "LLM_API_KEY"
              }
            }
          }

          env {
            name = "UAR_SECURITY__JWT_SECRET"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "UAR_SECURITY__JWT_SECRET"
              }
            }
          }

          env {
            name = "TAVILY_API_KEY"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "TAVILY_API_KEY"
              }
            }
          }

          env {
            name = "UAR_UNSTRUCTURED__API_KEY"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "UAR_UNSTRUCTURED__API_KEY"
              }
            }
          }

          env {
            name = "OPENAI_API_KEY"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "OPENAI_API_KEY"
              }
            }
          }

          env {
            name = "UAR_PERSISTENCE__REDIS_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "REDIS_URL"
              }
            }
          }

          env {
            name = "UAR_MEMORY__SURREAL_PASS"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_app_secrets.metadata[0].name
                key  = "UAR_MEMORY__SURREAL_PASS"
              }
            }
          }

          # ── Volumes ───────────────────────────────────────────────────────
          volume_mount {
            name       = "uar-uploads"
            mount_path = "/uploads"
          }

          volume_mount {
            name       = "uar-data"
            mount_path = "/data"
          }

          # ── Resources ─────────────────────────────────────────────────────
          resources {
            requests = {
              cpu    = "250m"
              memory = "256Mi"
            }
            limits = {
              cpu    = "1"
              memory = "1Gi"
            }
          }

          # ── Health checks ─────────────────────────────────────────────────
          liveness_probe {
            http_get {
              path = "/healthz"
              port = 3000
            }
            initial_delay_seconds = 30
            period_seconds        = 15
            timeout_seconds       = 5
            failure_threshold     = 3
          }

          readiness_probe {
            http_get {
              path = "/readyz"
              port = 3000
            }
            initial_delay_seconds = 15
            period_seconds        = 10
            timeout_seconds       = 3
            failure_threshold     = 3
          }
        }

        # ── Volume bindings ───────────────────────────────────────────────
        volume {
          name = "uar-uploads"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.uar_uploads.metadata[0].name
          }
        }

        volume {
          name = "uar-data"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.uar_data.metadata[0].name
          }
        }
      }
    }
  }
}

# ── ClusterIP Service ──────────────────────────────────────────────────────
# Exposed to the nginx Ingress on port 3000.
resource "kubernetes_service" "uar_svc" {
  metadata {
    name      = "uar-svc"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "uar"
      "app.kubernetes.io/component" = "application"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = "uar"
      "app.kubernetes.io/component" = "application"
    }

    port {
      name        = "http"
      port        = 3000
      target_port = 3000
      protocol    = "TCP"
    }
  }
}
