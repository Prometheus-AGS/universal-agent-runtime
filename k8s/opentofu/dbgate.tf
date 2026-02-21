# ── DbGate — PostgreSQL Web Administration ───────────────────────────────────
# DbGate (MIT license) is a modern, actively maintained database GUI with
# first-class environment-variable pre-configuration, SQL editor, ER diagrams,
# CSV/JSON export, and an embedded AI query assistant.
#
# The Postgres connection is fully automated via env vars — no manual UI setup
# required after deploy. Credentials are read from the existing uar-db-credentials
# Secret so they stay in sync with the PostgreSQL instance.
#
# Exposed externally at: https://pg.know-me.tools   (see ingress.tf)

resource "kubernetes_deployment" "dbgate" {
  metadata {
    name      = "dbgate"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "dbgate"
      "app.kubernetes.io/component" = "db-admin"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        "app.kubernetes.io/name"      = "dbgate"
        "app.kubernetes.io/component" = "db-admin"
      }
    }

    template {
      metadata {
        labels = {
          "app.kubernetes.io/name"      = "dbgate"
          "app.kubernetes.io/component" = "db-admin"
          "app.kubernetes.io/part-of"   = "universal-agent-runtime"
        }
      }

      spec {
        container {
          name  = "dbgate"
          image = var.dbgate_image

          port {
            name           = "http"
            container_port = 3000
            protocol       = "TCP"
          }

          # ── Pre-configure the UAR Postgres connection ──────────────────────
          # DbGate uses connection name "pg" — each variable with the _pg suffix
          # is scoped to that named connection.

          env {
            name  = "CONNECTIONS"
            value = "pg"
          }

          env {
            name  = "LABEL_pg"
            value = "UAR PostgreSQL"
          }

          env {
            name  = "ENGINE_pg"
            value = "postgres@dbgate-plugin-postgres"
          }

          env {
            name  = "SERVER_pg"
            value = "postgres-svc.uar.svc.cluster.local"
          }

          env {
            name  = "PORT_pg"
            value = "5432"
          }

          env {
            name  = "DATABASE_pg"
            value = var.postgres_db
          }

          # USER_pg and PASSWORD_pg come from the shared db-credentials secret
          env {
            name = "USER_pg"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "POSTGRES_USER"
              }
            }
          }

          env {
            name = "PASSWORD_pg"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.uar_db_credentials.metadata[0].name
                key  = "POSTGRES_PASSWORD"
              }
            }
          }

          # ── Resource limits ───────────────────────────────────────────────
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

          readiness_probe {
            http_get {
              path = "/"
              port = 3000
            }
            initial_delay_seconds = 10
            period_seconds        = 10
            timeout_seconds       = 5
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_secret.uar_db_credentials,
    kubernetes_service.postgres_svc,
  ]
}

# ── DbGate ClusterIP Service ──────────────────────────────────────────────────
resource "kubernetes_service" "dbgate_svc" {
  metadata {
    name      = "dbgate-svc"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "dbgate"
      "app.kubernetes.io/component" = "db-admin"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }
  }

  spec {
    type = "ClusterIP"

    selector = {
      "app.kubernetes.io/name"      = "dbgate"
      "app.kubernetes.io/component" = "db-admin"
    }

    port {
      name        = "http"
      port        = 3000
      target_port = 3000
      protocol    = "TCP"
    }
  }
}
