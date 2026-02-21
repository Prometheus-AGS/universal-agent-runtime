# ── GCP / GKE ──────────────────────────────────────────────────────────────

variable "gcp_project_id" {
  description = "The GCP project ID"
  type        = string
}

variable "gcp_region" {
  description = "The GCP region"
  type        = string
  default     = "us-central1"
}

variable "gke_cluster_name" {
  description = "The name of the GKE cluster"
  type        = string
}

variable "gke_cluster_location" {
  description = "The location/zone of the GKE cluster"
  type        = string
}

# ── PostgreSQL ──────────────────────────────────────────────────────────────

variable "postgres_image" {
  description = "Full image reference for the custom Postgres 17 image (pgvector + pgmq + migrations)"
  type        = string
  default     = "tribehealth/uar-postgres:pg17"
}

variable "postgres_db" {
  description = "PostgreSQL database name"
  type        = string
  default     = "uar"
}

variable "postgres_user" {
  description = "PostgreSQL username"
  type        = string
  default     = "uar"
}

variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
}

# ── LLM ────────────────────────────────────────────────────────────────────

variable "llm_base_url" {
  description = "Base URL for the LLM API"
  type        = string
  default     = "https://api.openai.com"
}

variable "llm_model" {
  description = "LLM model identifier (e.g. gpt-4o)"
  type        = string
  default     = "gpt-4o"
}

variable "llm_api_key" {
  description = "API key for the LLM provider"
  type        = string
  sensitive   = true
}

# ── OpenAI (embeddings) ─────────────────────────────────────────────────────

variable "openai_api_key" {
  description = "OpenAI API key used by the UAR memory embedding service"
  type        = string
  sensitive   = true
}

# ── Application secrets ─────────────────────────────────────────────────────

variable "jwt_secret" {
  description = "Secret key used to sign and verify JWT tokens"
  type        = string
  sensitive   = true
}

variable "tavily_api_key" {
  description = "Tavily API key for the web-search MCP tool"
  type        = string
  sensitive   = true
}

variable "unstructured_api_key" {
  description = "Unstructured.io API key for document ingestion"
  type        = string
  sensitive   = true
}

# ── SurrealDB ────────────────────────────────────────────────────────────────

variable "surreal_image" {
  description = "SurrealDB 3.x image reference"
  type        = string
  default     = "surrealdb/surrealdb:v3"
}

variable "surreal_user" {
  description = "Root username for the SurrealDB instance"
  type        = string
  default     = "root"
}

variable "surreal_pass" {
  description = "Root password for the SurrealDB instance"
  type        = string
  sensitive   = true
}

# ── Surrealist Web UI ────────────────────────────────────────────────────────

variable "surrealist_image" {
  description = "Surrealist web UI image (SurrealDB v3-compatible)"
  type        = string
  default     = "surrealdb/surrealist:3.7.2"
}

# ── DbGate ───────────────────────────────────────────────────────────────────

variable "dbgate_image" {
  description = "DbGate database administration UI image"
  type        = string
  default     = "dbgate/dbgate:latest"
}
