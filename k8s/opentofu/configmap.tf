# Non-sensitive UAR runtime configuration.
# Sensitive values (API keys, passwords, JWT secret) live in secrets.tf.
resource "kubernetes_config_map" "uar_config" {
  metadata {
    name      = "uar-config"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/part-of" = "universal-agent-runtime"
    }
  }

  data = {
    # ── Server ──────────────────────────────────────────────────────────────
    UAR_SERVER__PORT      = "1906"
    UAR_SERVER__GRPC_PORT = "50051"
    UAR_SERVER__HOST      = "0.0.0.0"

    # ── Persistence ──────────────────────────────────────────────────────────
    UAR_PERSISTENCE__PROVIDER               = "postgres"
    UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED = "true"
    # Dimensions for the pgvector column; must match the embedding model.
    # text-embedding-3-small → 1536, text-embedding-3-large → 3072
    UAR_PERSISTENCE__VECTOR_DIMENSION = "1536"

    # ── Security ────────────────────────────────────────────────────────────
    # JWT is validated when present but not enforced — requests without a token
    # run as anonymous. Set to "true" to require a valid token on every request.
    UAR_SECURITY__JWT_REQUIRED = "false"

    # ── Resilience ──────────────────────────────────────────────────────────
    UAR_RESILIENCE__RATE_LIMIT_ENABLED = "true"

    # ── File processing ──────────────────────────────────────────────────────
    UAR_FILE_PROCESSING__PROVIDER   = "unstructured"
    UAR_FILE_PROCESSING__UPLOAD_DIR = "/uploads"

    # Unstructured.io public API endpoint (key injected from secret)
    UAR_UNSTRUCTURED__API_URL = "https://api.unstructured.io/general/v0/general"

    # ── Memory system ────────────────────────────────────────────────────────
    UAR_MEMORY__ENABLED            = "true"
    UAR_MEMORY__EMBEDDING_PROVIDER = "openai"
    UAR_MEMORY__EMBEDDING_MODEL    = "text-embedding-3-small"
    UAR_MEMORY__AUTO_CAPTURE       = "true"
    UAR_MEMORY__INJECT_CONTEXT     = "true"
    UAR_MEMORY__MCP_HTTP_ENABLED   = "true"

    # External SurrealDB 3.0 instance (ws:// = plain WebSocket within cluster)
    UAR_MEMORY__SURREAL_ENDPOINT = "ws://surreal-svc.uar.svc.cluster.local:8000"
    UAR_MEMORY__SURREAL_USER     = var.surreal_user

    # ── LLM ─────────────────────────────────────────────────────────────────
    LLM_PROTOCOL = "auto"
    LLM_BASE_URL = var.llm_base_url
    LLM_MODEL    = var.llm_model

    # ── Observability ────────────────────────────────────────────────────────
    RUST_LOG = "info"
  }
}
