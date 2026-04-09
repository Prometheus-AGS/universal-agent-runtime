## Context

The Universal Agent Runtime (UAR) is a Rust/Axum agentic LLM runtime at 81% functional completion. A production readiness assessment identified 20 improvements across 4 priority tiers plus a complete GKE deployment pipeline. The target environment is `client-cluster` on GKE with Envoy Gateway and ArgoCD already installed. The codebase uses liter-llm for 142+ LLM providers, MCP for tool integration, and a React/TypeScript frontend with shadcn/ui.

**Current architecture:**
- Backend: Rust (Axum 0.8, Tokio 1.50, SQLx 0.8, SurrealDB 3.0)
- Frontend: React 19, Vite 7, Zustand, @assistant-ui/react
- Databases: PostgreSQL 17 (pgvector), Redis 7, SurrealDB v3
- Protocols: A2A (JSON-RPC), AG-UI (SSE), A2UI (schema registry), MCP (rmcp)

## Goals / Non-Goals

**Goals:**
- Achieve safe Kubernetes production deployment with zero-downtime rolling updates
- Fix all 3 critical blockers (graceful shutdown, deep health probes, JSON logging)
- Implement all 17 additional improvements across P1-P3 priorities
- Create complete GKE deployment manifests with immediate-binding storage
- Create CI/CD pipeline via GitHub Actions with ArgoCD integration
- Maintain backward compatibility with existing API consumers

**Non-Goals:**
- Creating the GKE cluster, Envoy Gateway, or CertBot (already installed)
- Multi-region deployment or disaster recovery automation
- Custom Kubernetes operators or CRDs
- Migrating away from liter-llm or changing the core LLM abstraction
- Full WCAG 2.1 AAA accessibility compliance (target AA)
- Production database backup automation (separate concern)

## Decisions

### D1: Graceful Shutdown Strategy
**Decision**: Use `tokio::signal::ctrl_c()` + `tokio::signal::unix::signal(SignalKind::terminate())` combined with `axum::serve().with_graceful_shutdown()`.

**Rationale**: This is the idiomatic Axum/Tokio pattern. The graceful shutdown signal propagates to all active connections, allowing in-flight SSE streams and HTTP requests to complete within a configurable timeout (default 30s, matching K8s `terminationGracePeriodSeconds`).

**Alternative considered**: Manual connection tracking with `Arc<AtomicBool>` — rejected as unnecessarily complex when Axum provides built-in support.

### D2: Health Probe Architecture
**Decision**: Split into two distinct endpoints:
- `/healthz` (liveness): Lightweight check that the process is alive and can serve HTTP. No dependency checks.
- `/readyz` (readiness): Verifies PostgreSQL (via `sqlx::query("SELECT 1")`), Redis (via `PING`), and SurrealDB (via health endpoint). Returns 503 with JSON body listing failed dependencies.

**Rationale**: Liveness probes must be fast and side-effect-free — checking dependencies risks false pod kills during transient DB issues. Readiness probes gate traffic routing, so dependency checks are appropriate there.

### D3: Logging Format
**Decision**: Use `tracing_subscriber::fmt::layer().json()` with `tracing-bunyan-formatter` for K8s-compatible structured output. Add an env var `UAR_LOG_FORMAT` (`json` | `compact` | `pretty`) defaulting to `json` in production, `pretty` in development.

**Rationale**: JSON logging is required for ELK/Loki/CloudWatch. But developers need readable logs locally. Making it configurable via env var satisfies both.

### D4: Token Estimation
**Decision**: Use the `tiktoken-rs` crate for OpenAI-compatible tokenization. For non-OpenAI models, fall back to a conservative char-based estimate with a 4:1 chars-to-tokens ratio (instead of current simple char count).

**Rationale**: `tiktoken-rs` provides exact token counts for OpenAI models (which are the most common). The improved fallback ratio is based on empirical measurement across model families. Full per-provider tokenizers would add significant binary size for marginal accuracy gains.

**Alternative considered**: `tokenizers` crate from HuggingFace — already a dependency for embedding, but loading model-specific tokenizers for every provider is impractical.

### D5: Prometheus Metrics
**Decision**: Use `metrics` crate (facade) with `metrics-exporter-prometheus` for exposition at `/metrics`. Key metrics:
- `uar_requests_total` (counter, labels: method, path, status)
- `uar_request_duration_seconds` (histogram, labels: method, path)
- `uar_llm_tokens_total` (counter, labels: provider, model, direction)
- `uar_tool_calls_total` (counter, labels: tool_name, status)
- `uar_active_sessions` (gauge)
- `uar_mcp_server_status` (gauge, labels: server_name)

**Rationale**: The `metrics` crate is the Rust ecosystem standard and decouples instrumentation from the exporter. Prometheus is the de facto K8s monitoring standard.

### D6: Tool Approval Workflow
**Decision**: Implement a configurable tool approval policy via Cedar policy rules. When a tool call matches a "requires_approval" policy, the run pauses and emits an `agui.tool_call.approval_required` event. The frontend renders an approval dialog. The user's response routes through `/api/uar/runs/{run_id}/tool-approval` which resumes the run.

**Rationale**: Cedar policy engine is already integrated for governance. Extending it to tool approval keeps authorization logic centralized and declarative.

### D7: GKE Deployment Architecture
**Decision**: Plain Kubernetes YAML manifests in `k8s/` organized by component, with Kustomize overlays for environment-specific configuration. ArgoCD watches this directory for GitOps-style deployment.

**Structure:**
```
k8s/
  base/
    namespace.yaml
    uar-deployment.yaml
    uar-service.yaml
    uar-hpa.yaml
    uar-serviceaccount.yaml
    postgres-statefulset.yaml
    postgres-service.yaml
    surrealdb-statefulset.yaml
    surrealdb-service.yaml
    redis-deployment.yaml
    redis-service.yaml
    configmap.yaml
    secrets.yaml
    storage-class.yaml
    network-policies.yaml
    httproute.yaml
  overlays/
    production/
      kustomization.yaml
      patches/
```

**Rationale**: Kustomize is native to kubectl, works natively with ArgoCD, and avoids Helm's templating complexity for a single-application deployment. The Helm chart is provided as an alternative for teams that prefer it.

**Alternative considered**: OpenTofu (already exists in `k8s/opentofu/`) — kept as an alternative but Kustomize is more appropriate for ArgoCD GitOps.

### D8: GitHub Actions Pipeline
**Decision**: Single workflow `deploy.yml` with 3 jobs:
1. `build`: Multi-platform Docker build, push to Artifact Registry (`gcr.io` or `us-docker.pkg.dev`)
2. `deploy`: Update image tag in K8s manifests, commit to repo, ArgoCD auto-syncs
3. `smoke-test`: Wait for rollout, hit `/readyz` endpoint

**Triggers**: Push to `main` branch, manual `workflow_dispatch` with optional image tag override.

**Rationale**: ArgoCD watches the repo for manifest changes, so the pipeline only needs to build the image and update the tag. This is the standard GitOps pattern.

### D9: Helm Chart
**Decision**: Create a Helm chart in `k8s/helm/uar/` with `values.yaml` exposing all configurable parameters. The chart generates the same manifests as the Kustomize base.

**Rationale**: Some teams prefer Helm for its release management, rollback, and values override ergonomics. Providing both Kustomize and Helm maximizes adoption.

### D10: Frontend Theme Toggle
**Decision**: Implement CSS-variable-based theming with `class="dark"` / `class="light"` on `<html>`. Store preference in localStorage with system preference detection via `prefers-color-scheme`. Use Zustand for theme state.

**Rationale**: The existing Tailwind config already uses `darkMode: "class"`. Adding light mode tokens to the CSS variables and a toggle component is minimal effort.

### D11: A2A gRPC Transport
**Decision**: Use `tonic` crate to expose A2A v0.3 gRPC endpoints alongside the existing JSON-RPC endpoint. Share the same handler logic, only the transport layer differs.

**Rationale**: A2A v0.3 adds gRPC as a first-class transport. `tonic` is the standard Rust gRPC framework and integrates well with Tokio.

## Risks / Trade-offs

| Risk | Severity | Mitigation |
|------|----------|------------|
| Graceful shutdown timeout too short for long-running SSE streams | Medium | Configure `terminationGracePeriodSeconds: 60` in K8s, match in app config |
| `tiktoken-rs` adds ~5MB to binary size | Low | Acceptable for accuracy improvement; feature-gate if needed |
| Prometheus cardinality explosion from tool_name label | Medium | Cap unique tool names at 100; use `other` bucket for overflow |
| Helm chart and Kustomize diverge over time | Medium | CI job validates both produce equivalent manifests |
| A2A gRPC port conflicts with HTTP server | Low | Run gRPC on separate port (50051) or use `tonic` with Axum integration |
| Light mode CSS tokens incomplete | Low | Start with auto-generated light palette from existing dark tokens; iterate |
| PVC immediate binding may fail if no nodes in zone | Medium | Use `WaitForFirstConsumer` as documented fallback; user explicitly requested immediate |
| ArgoCD sync may fail if secrets not pre-created | Medium | Document secret creation as prerequisite; provide `kubectl create secret` commands |

## Migration Plan

### Phase 1: Critical Fixes (P0) - Deploy First
1. Implement graceful shutdown, deep health probes, JSON logging
2. Update Dockerfile to latest Rust
3. Create K8s base manifests and GitHub Actions workflow
4. Deploy to `client-cluster` via ArgoCD
5. Verify health probes and rolling update behavior

### Phase 2: Production Quality (P1)
6. Add Prometheus metrics, `/v1/models` endpoint
7. Wire ProgressiveSummarization
8. Deploy frontend updates (agent status labels, A2UI testing)
9. Verify metrics scraping via Prometheus

### Phase 3: Competitive Parity (P2)
10. Add light mode, network policies, tool approval, context visualization
11. Improve token estimation
12. Create Helm chart
13. Deploy and validate NetworkPolicy enforcement

### Phase 4: Nice to Have (P3)
14. MCP health dashboard, skill hot-reload, OpenAPI docs
15. PWA service worker, A2A gRPC
16. Final deployment and smoke tests

### Rollback Strategy
- ArgoCD provides instant rollback to previous git commit
- Database migrations are forward-only; no destructive migrations in this change
- Feature flags not used; rollback is full image revert
