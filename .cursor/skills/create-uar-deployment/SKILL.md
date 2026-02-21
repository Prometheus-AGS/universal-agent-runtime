---
name: create-uar-deployment
description: >
  Generate a complete OpenTofu (Terraform-compatible) GKE deployment for the
  Universal Agent Runtime. Creates the Postgres 17 Dockerfile (pgvector + pgmq
  + all UAR migrations), SSD PersistentVolumeClaims, Redis, the UAR Deployment,
  and an nginx + cert-manager Ingress for a user-specified domain. Use when
  the user asks to deploy UAR to a GKE cluster, create a Kubernetes deployment
  for UAR, or set up a new UAR environment.
allowed-tools: file_system web_search tavily
---

# Create UAR GKE Deployment

Generates the full `k8s/` infrastructure for deploying Universal Agent Runtime
to an existing GKE cluster using OpenTofu.

## Phase 1 — Collect Required Parameters

Gather these before generating any files. Use `AskQuestion` when available;
otherwise ask conversationally. Group into required and optional.

### Required

| Parameter | Description | Example |
|-----------|-------------|---------|
| `cluster_name` | Existing GKE cluster name | `client-cluster` |
| `gcp_project_id` | GCP project ID | `my-project-123456` |
| `gcp_region` | GCP region + zone for cluster | `us-central1` |
| `hostname` | Full public DNS hostname for the Ingress | `uar.example.com` |
| `uar_image` | UAR Docker image reference | `tribehealth/universal-agent-runtime:02202026` |
| `postgres_image` | Custom Postgres image to build+push | `yourorg/uar-postgres:pg17` |
| `namespace` | Kubernetes namespace (default: `uar`) | `uar` |
| `llm_provider` | LLM provider type | `openai` \| `azure` \| `other` |
| `llm_model` | Model identifier | `gpt-4o` |
| `llm_base_url` | LLM API base URL | `https://api.openai.com` |

### Sensitive (never hard-coded — go into tfvars)

- `postgres_password` — PostgreSQL password
- `llm_api_key` — LLM provider API key
- `jwt_secret` — JWT signing secret
- `openai_api_key` — OpenAI embeddings key (for memory system)
- `tavily_api_key` — Tavily web-search MCP tool key
- `unstructured_api_key` — Unstructured.io document ingestion key

### Optional / Defaults

| Parameter | Default | Notes |
|-----------|---------|-------|
| `postgres_db` | `uar` | PostgreSQL database name |
| `postgres_user` | `uar` | PostgreSQL username |
| `postgres_data_size` | `20Gi` | Postgres PVC size |
| `redis_data_size` | `5Gi` | Redis PVC size |
| `uar_uploads_size` | `10Gi` | File upload PVC size |
| `uar_data_size` | `5Gi` | Runtime data PVC size |
| `storage_class` | `premium-rwo` | GKE SSD StorageClass |
| `cert_issuer` | `letsencrypt-prod` | cert-manager ClusterIssuer name |
| `ingress_class` | `nginx` | Kubernetes IngressClass |
| `replicas` | `1` | UAR replica count |
| `file_processing` | `unstructured` | `unstructured` \| `mistral` \| `auto` |
| `memory_enabled` | `true` | Enable UAR semantic memory system |
| `llm_protocol` | `auto` | `auto` \| `responses` \| `chat` |

## Phase 2 — Generate File Structure

Create exactly this layout under the project root:

```
k8s/
  postgres/
    Dockerfile
    initdb/
      00_extensions.sql      ← CREATE EXTENSION vector; pgmq;
      01_init_uar.sql        ← sessions, skills, knowledge_bases, knowledge_chunks
      02_create_agents.sql
      03_create_memories.sql
      04_knowledge_bases_v2.sql
      05_create_settings.sql
      06_fix_skills_schema.sql   ← rename skills.id → skill_id
      07_create_compiler_tables.sql
      08_create_agent_registry.sql
      09_chat_attachments.sql
      10_settings_tables.sql     ← enhanced settings + updated_at triggers
  opentofu/
    providers.tf     ← Google + Kubernetes providers, GCS backend
    variables.tf     ← all vars, sensitive = true for secrets
    terraform.tfvars ← all values, CHANGE_ME placeholders for secrets
    namespace.tf
    secrets.tf       ← uar-db-credentials, uar-app-secrets
    configmap.tf     ← non-sensitive env vars
    storage.tf       ← 4 SSD PVCs
    postgres.tf      ← StatefulSet + ClusterIP Service
    redis.tf         ← Deployment + ClusterIP Service
    deployment.tf    ← UAR Deployment + ClusterIP Service
    ingress.tf       ← nginx Ingress + cert-manager TLS
```

## Phase 3 — Postgres Dockerfile

### Template

```dockerfile
FROM pgvector/pgvector:pg17

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates gnupg \
    && curl -fsSL https://apt.fury.io/tembo/gpg.key \
        | gpg --dearmor -o /usr/share/keyrings/tembo.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/tembo.gpg trusted=yes] https://apt.fury.io/tembo/ /" \
        > /etc/apt/sources.list.d/tembo.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends postgresql-17-pgmq \
    && apt-get purge -y --auto-remove curl gnupg \
    && rm -rf /var/lib/apt/lists/*

COPY initdb/ /docker-entrypoint-initdb.d/
```

### Migration order rules

Files in `initdb/` run in **lexicographic order** on first container start
(when `PGDATA` is empty). The numbering `00`–`10` enforces correct dependency
order. All SQL must use `IF NOT EXISTS` / `IF EXISTS` guards for idempotency.

### Migration dependency requirements

1. `00` — extensions first (pgvector already in base image; pgmq installed above)
2. `01` — base tables (sessions, skills, knowledge_bases, knowledge_chunks)
3. `02–03` — independent tables (agents, memories)
4. `04` — ALTER knowledge_bases + knowledge_documents + ALTER knowledge_chunks
5. `05` — settings v1 (simple schema)
6. `06` — must come after `01` (renames `skills.id` → `skill_id`)
7. `07–09` — independent new tables
8. `10` — enhanced settings (adds triggers on existing tables from `05`)

## Phase 4 — OpenTofu File Specifications

### `providers.tf`

```hcl
terraform {
  required_version = ">= 1.0"
  required_providers {
    kubernetes = { source = "hashicorp/kubernetes", version = "~> 2.23" }
    google     = { source = "hashicorp/google",     version = "~> 4.84" }
  }
  backend "gcs" {
    bucket = "<gcs-state-bucket>"
    prefix = "universal-agent-runtime"
  }
}

provider "google" { project = var.gcp_project_id; region = var.gcp_region }

data "google_container_cluster" "primary" {
  name     = var.gke_cluster_name
  location = var.gke_cluster_location
}
data "google_client_config" "default" {}

provider "kubernetes" {
  host  = "https://${data.google_container_cluster.primary.endpoint}"
  token = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(
    data.google_container_cluster.primary.master_auth[0].cluster_ca_certificate
  )
}
```

### `secrets.tf` — two Kubernetes Secrets

**`uar-db-credentials`**: `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`,
`DATABASE_URL` (DSN built as `postgres://<user>:<pass>@postgres-svc.<ns>.svc.cluster.local:5432/<db>`)

**`uar-app-secrets`**: `LLM_API_KEY`, `UAR_SECURITY__JWT_SECRET`,
`TAVILY_API_KEY`, `UAR_UNSTRUCTURED__API_KEY`, `OPENAI_API_KEY`,
`REDIS_URL` (`redis://redis-svc.<ns>.svc.cluster.local:6379`)

### `configmap.tf` — non-sensitive configuration

| Key | Value / Source |
|-----|----------------|
| `UAR_SERVER__PORT` | `3000` |
| `UAR_SERVER__HOST` | `0.0.0.0` |
| `UAR_PERSISTENCE__PROVIDER` | `postgres` |
| `UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED` | `true` |
| `UAR_SECURITY__JWT_REQUIRED` | `true` |
| `UAR_RESILIENCE__RATE_LIMIT_ENABLED` | `true` |
| `UAR_FILE_PROCESSING__PROVIDER` | `var.file_processing` |
| `UAR_FILE_PROCESSING__UPLOAD_DIR` | `/uploads` |
| `UAR_UNSTRUCTURED__API_URL` | `https://api.unstructured.io/general/v0/general` |
| `UAR_MEMORY__ENABLED` | `var.memory_enabled` |
| `UAR_MEMORY__EMBEDDING_PROVIDER` | `openai` |
| `UAR_MEMORY__EMBEDDING_MODEL` | `text-embedding-3-small` |
| `LLM_PROTOCOL` | `var.llm_protocol` |
| `LLM_BASE_URL` | `var.llm_base_url` |
| `LLM_MODEL` | `var.llm_model` |
| `RUST_LOG` | `info` |

### `storage.tf` — SSD PVCs

All `ReadWriteOnce`. StorageClass: `var.storage_class` (default: `premium-rwo`).

| Resource name | Claim name | Size var | Mounted at |
|---------------|-----------|----------|------------|
| `postgres_data` | `postgres-data-pvc` | `postgres_data_size` | `/var/lib/postgresql/data` |
| `redis_data` | `redis-data-pvc` | `redis_data_size` | `/data` |
| `uar_uploads` | `uar-uploads-pvc` | `uar_uploads_size` | `/uploads` |
| `uar_data` | `uar-data-pvc` | `uar_data_size` | `/data` |

Add `lifecycle { prevent_destroy = true }` to `postgres_data`.

### `postgres.tf` — StatefulSet

- `service_name = "postgres-svc"`; `replicas = 1`
- Image: `var.postgres_image`
- Env: `POSTGRES_USER/PASSWORD/DB` from `uar-db-credentials` secret
- Volume: `postgres-data-pvc` → `/var/lib/postgresql/data` with `sub_path = "pgdata"`
- Liveness/readiness: `exec { command = ["pg_isready", "-U", "$(POSTGRES_USER)", "-d", "$(POSTGRES_DB)"] }`
- Resources: `requests: cpu=250m, memory=256Mi` / `limits: cpu=1, memory=2Gi`
- ClusterIP Service `postgres-svc` on port 5432

### `redis.tf` — Deployment

- `strategy.type = "Recreate"` (prevents two pods fighting over RWO PVC)
- Image: `redis:7-alpine`; Args: `redis-server --appendonly yes --dir /data`
- Volume: `redis-data-pvc` → `/data`
- Liveness/readiness: `exec { command = ["redis-cli", "ping"] }`
- Resources: `requests: cpu=100m, memory=128Mi` / `limits: cpu=500m, memory=512Mi`
- ClusterIP Service `redis-svc` on port 6379

### `deployment.tf` — UAR Application

- Init container: `busybox` — wait for postgres with `nc -z postgres-svc 5432`
- Image: `var.uar_image`; `replicas = var.replicas` (default 1)
- `env_from`: ConfigMap `uar-config`
- Individual `env` entries from both secrets (see `secrets.tf` section)
- Volumes: `uar-uploads-pvc` → `/uploads`, `uar-data-pvc` → `/data`
- Liveness: `httpGet /healthz :3000` (30s initial delay)
- Readiness: `httpGet /readyz :3000` (15s initial delay)
- Resources: `requests: cpu=250m, memory=256Mi` / `limits: cpu=1, memory=1Gi`
- ClusterIP Service `uar-svc` on port 3000

### `ingress.tf` — nginx + cert-manager

```hcl
annotations:
  kubernetes.io/ingress.class:                   nginx
  cert-manager.io/cluster-issuer:                <var.cert_issuer>
  nginx.ingress.kubernetes.io/ssl-redirect:       "true"
  nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
  nginx.ingress.kubernetes.io/proxy-read-timeout: "600"   # SSE streaming
  nginx.ingress.kubernetes.io/proxy-send-timeout: "600"
  nginx.ingress.kubernetes.io/proxy-buffering:    "off"   # SSE streaming
  nginx.ingress.kubernetes.io/proxy-body-size:    "100m"  # file uploads

tls:
  - hosts: [<hostname>]
    secretName: uar-tls

rules:
  - host: <hostname>
    http.paths:
      - path: /  pathType: Prefix  backend: uar-svc:3000
```

## Phase 5 — Pre-Deployment Checklist

```
- [ ] All CHANGE_ME values replaced in terraform.tfvars
- [ ] terraform.tfvars added to .gitignore
- [ ] Postgres image built and pushed: docker build -t <postgres_image> k8s/postgres/ && docker push <postgres_image>
- [ ] GCS state bucket exists (referenced in providers.tf backend)
- [ ] kubectl context points to target cluster: kubectl config current-context
- [ ] cert-manager ClusterIssuer exists: kubectl get clusterissuer <cert_issuer>
- [ ] nginx IngressClass exists: kubectl get ingressclass nginx
- [ ] DNS A record for <hostname> points to the nginx ingress LoadBalancer IP
- [ ] tofu init && tofu plan (review plan before apply)
- [ ] tofu apply
```

## Phase 6 — Verify Deployment

```bash
# All pods Running
kubectl -n <namespace> get pods

# Certificate issued
kubectl -n <namespace> get certificate uar-tls

# App health
kubectl -n <namespace> exec deploy/uar -- curl -s http://localhost:3000/healthz

# Public HTTPS
curl -I https://<hostname>/healthz
```

## Additional Resources

- For detailed migration SQL, see `migrations/` in the project root.
- For complete variable documentation, see `k8s/opentofu/variables.tf`.
- For an example of a fully-generated deployment, see `k8s/opentofu/` (generated
  for `uar.know-me.tools` on `client-cluster`).
