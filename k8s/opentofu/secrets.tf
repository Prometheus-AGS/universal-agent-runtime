# ── Database credentials ────────────────────────────────────────────────────
# Consumed by the PostgreSQL StatefulSet and the UAR application Deployment.
resource "kubernetes_secret" "uar_db_credentials" {
  metadata {
    name      = "uar-db-credentials"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/part-of" = "universal-agent-runtime"
    }
  }

  # type = "Opaque" is the default; explicit for clarity.
  type = "Opaque"

  data = {
    POSTGRES_USER     = var.postgres_user
    POSTGRES_PASSWORD = var.postgres_password
    POSTGRES_DB       = var.postgres_db
    # Full DSN consumed by the UAR app via UAR_PERSISTENCE__DATABASE_URL
    DATABASE_URL = "postgres://${var.postgres_user}:${var.postgres_password}@postgres-svc.uar.svc.cluster.local:5432/${var.postgres_db}"
  }
}

# ── SurrealDB credentials ────────────────────────────────────────────────────
# Consumed by the SurrealDB StatefulSet and the UAR memory service.
resource "kubernetes_secret" "uar_surreal_credentials" {
  metadata {
    name      = "uar-surreal-credentials"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/part-of" = "universal-agent-runtime"
    }
  }

  type = "Opaque"

  data = {
    SURREAL_USER = var.surreal_user
    SURREAL_PASS = var.surreal_pass
  }
}

# ── Application secrets ─────────────────────────────────────────────────────
# LLM, JWT, third-party API keys, and the Redis URL.
resource "kubernetes_secret" "uar_app_secrets" {
  metadata {
    name      = "uar-app-secrets"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/part-of" = "universal-agent-runtime"
    }
  }

  type = "Opaque"

  data = {
    LLM_API_KEY               = var.llm_api_key
    UAR_SECURITY__JWT_SECRET  = var.jwt_secret
    TAVILY_API_KEY            = var.tavily_api_key
    UAR_UNSTRUCTURED__API_KEY = var.unstructured_api_key
    # Used by the memory embedding service; falls back to OPENAI_API_KEY env var
    OPENAI_API_KEY = var.openai_api_key
    # Redis DSN — the Redis Service is defined in redis.tf
    REDIS_URL = "redis://redis-svc.uar.svc.cluster.local:6379"
    # SurrealDB password — read by UAR memory service as UAR_MEMORY__SURREAL_PASS
    UAR_MEMORY__SURREAL_PASS = var.surreal_pass
  }
}
