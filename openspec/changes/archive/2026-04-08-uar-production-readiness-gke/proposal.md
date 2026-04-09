## Why

The Universal Agent Runtime has reached 81% functional completion with a B+ UI/UX grade, but has 3 critical blockers preventing safe Kubernetes production deployment: no graceful shutdown (SIGTERM handling), shallow health probes (always-200), and non-JSON logging format. Beyond these blockers, the assessment identified 17 additional improvements needed for production quality, competitive parity, and operational excellence. A complete GKE deployment pipeline with GitHub Actions CI/CD is required to ship the runtime to the `client-cluster` GKE environment with ArgoCD and Envoy Gateway already in place.

## What Changes

### P0 - Critical Fixes (Must fix before production)
- Add SIGTERM graceful shutdown handler with connection draining via `tokio::signal` + `axum::serve().with_graceful_shutdown()`
- Upgrade health probes (`/healthz`, `/readyz`) to verify PostgreSQL, Redis, and SurrealDB connectivity
- Switch tracing-subscriber from `.compact()` to `.json()` structured logging

### P1 - Production Quality
- Wire `ProgressiveSummarization` context strategy to actual LLM summarizer (currently stubs to KeepFirstLast)
- Add `/v1/models` endpoint for OpenAI SDK compatibility
- Add `HorizontalPodAutoscaler` to K8s manifests
- Add agent status labels to chat UI ("Thinking...", "Executing tool...", "Searching...")
- Add A2UI artifact testing/preview UI to admin dashboard
- Implement Prometheus metrics exporter via `metrics` + `prometheus` crates

### P2 - Competitive Parity
- Add light/dark mode toggle to UI (currently dark-only)
- Add Kubernetes `NetworkPolicy` manifests for inter-service isolation
- Implement tool call approval workflow for high-risk MCP operations
- Add context window usage visualization to chat UI (token budget, strategy indicator)
- Improve token estimation accuracy (replace char-count heuristic with tiktoken-equivalent)
- Add Helm chart as alternative deployment method to OpenTofu

### P3 - Nice to Have
- Add MCP server health/status dashboard to admin
- Add skill hot-reload without server restart
- Add OpenAPI/Swagger API documentation generation
- Add PWA service worker for offline support
- Implement A2A v0.3 gRPC transport support

### Deployment Infrastructure
- Update Dockerfile to latest Rust stable base image, optimize multi-stage build for current React+Vite frontend
- Create complete GKE deployment manifests in `k8s/` targeting `client-cluster`
- Create GitHub Actions workflow for build, push to Artifact Registry, and ArgoCD-triggered deployment
- All PVCs use `volumeBindingMode: Immediate` storage class
- HTTPRoute for Envoy Gateway (not Ingress)

## Capabilities

### New Capabilities
- `graceful-shutdown`: SIGTERM signal handling with in-flight request draining and resource cleanup
- `deep-health-probes`: Dependency-aware health and readiness checks (PostgreSQL, Redis, SurrealDB)
- `json-logging`: Structured JSON log output for Kubernetes log aggregation
- `progressive-summarization`: LLM-powered conversation summarization for context management
- `openai-models-endpoint`: `/v1/models` API endpoint listing available models from the catalog
- `prometheus-metrics`: Runtime metrics exposition via Prometheus `/metrics` endpoint
- `agent-status-ui`: Real-time agent status labels in chat UI (thinking, tool execution, searching)
- `a2ui-testing-ui`: Admin dashboard page for testing and previewing A2UI artifact types
- `light-mode`: Light/dark theme toggle with system preference detection
- `tool-approval-workflow`: Human-in-the-loop approval for high-risk tool calls
- `context-visualization`: Token budget and context strategy visualization in chat UI
- `tiktoken-estimation`: Accurate token counting using tiktoken-compatible tokenizer
- `helm-chart`: Helm chart packaging for K8s deployment
- `network-policies`: Kubernetes NetworkPolicy manifests for service isolation
- `hpa-autoscaling`: HorizontalPodAutoscaler configuration for UAR pods
- `mcp-health-dashboard`: Admin page showing MCP server connection status and health
- `skill-hot-reload`: Runtime skill reloading without server restart
- `openapi-docs`: Auto-generated OpenAPI/Swagger documentation from Axum routes
- `pwa-offline`: Progressive Web App service worker for offline capability
- `a2a-grpc`: A2A v0.3 gRPC transport implementation
- `gke-deployment`: Complete GKE deployment manifests, Dockerfile update, and CI/CD pipeline

## Impact

### Backend (Rust)
- `src/server.rs`: Graceful shutdown, health probes, new endpoints (`/v1/models`, `/metrics`)
- `src/uar/telemetry/mod.rs`: JSON logging format switch
- `src/uar/runtime/context/manager.rs`: Wire ProgressiveSummarization
- `src/uar/runtime/context/token_service.rs`: Replace char-count with tiktoken
- `src/uar/runtime/manager.rs`: Tool approval workflow, agent status events
- `src/uar/runtime/skills/registry.rs`: Hot-reload support
- `src/uar/api/a2a/`: gRPC transport addition
- `src/mcp/registry.rs`: Health monitoring
- New module: `src/uar/api/openapi.rs` for Swagger generation

### Frontend (React/TypeScript)
- `frontend/src/features/chat/`: Agent status labels, context visualization, tool approval UI
- `frontend/src/admin/`: A2UI testing page, MCP health dashboard
- `frontend/src/`: Theme toggle, service worker registration
- `tailwind.config.ts`: Light mode color tokens

### Infrastructure
- `Dockerfile`: Updated Rust base image, optimized build layers
- `k8s/`: New directory with all GKE manifests (namespace, deployments, statefulsets, services, configmaps, secrets, PVCs, HTTPRoute, NetworkPolicy, HPA, RBAC)
- `k8s/helm/`: Helm chart with `values.yaml`
- `.github/workflows/deploy.yml`: Build + push + ArgoCD sync workflow

### Dependencies
- New crate: `metrics`, `metrics-exporter-prometheus` (Prometheus)
- New crate: `tonic` (gRPC for A2A v0.3)
- New crate: `utoipa` or `aide` (OpenAPI generation)
- New npm: Theme provider, service worker tooling
