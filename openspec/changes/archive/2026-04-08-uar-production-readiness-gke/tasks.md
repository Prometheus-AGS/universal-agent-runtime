## 1. P0: Graceful Shutdown

- [x] 1.1 Add `tokio::signal` SIGTERM and SIGINT handler in `src/server.rs`
- [x] 1.2 Wire `axum::serve().with_graceful_shutdown()` with the signal handler
- [x] 1.3 Add configurable shutdown timeout via `UAR_SERVER__SHUTDOWN_TIMEOUT_SECS` (default 30)
- [x] 1.4 Implement resource cleanup on shutdown (drop DB pools, close MCP connections, close Redis)
- [x] 1.5 Add integration test: verify server stops accepting new connections after SIGTERM

## 2. P0: Deep Health Probes

- [x] 2.1 Refactor `/healthz` to return lightweight JSON `{"status": "ok"}` without dependency checks
- [x] 2.2 Implement `/readyz` handler that checks PostgreSQL (`SELECT 1`), Redis (`PING`), and SurrealDB health
- [x] 2.3 Return structured JSON with per-dependency status and HTTP 503 when any dependency fails
- [x] 2.4 Ensure both endpoints bypass authentication middleware (verify existing exclusion in security middleware)
- [x] 2.5 Add integration test: verify `/readyz` returns 503 when database is unreachable

## 3. P0: JSON Structured Logging

- [x] 3.1 Add `UAR_LOG_FORMAT` env var to config (values: `json`, `compact`, `pretty`; default: `json`)
- [x] 3.2 Switch `tracing_subscriber::fmt::layer()` in `src/uar/telemetry/mod.rs` to conditionally use `.json()` or `.compact()` or `.pretty()` based on config
- [x] 3.3 Ensure JSON output uses RFC 3339 timestamps and single-line format
- [x] 3.4 Verify log output parses correctly with `jq` in a smoke test

## 4. P1: Progressive Summarization

- [x] 4.1 Create `src/uar/runtime/context/summarizer.rs` module with LLM-based summarization function
- [x] 4.2 Wire `ProgressiveSummarization` strategy in `manager.rs` to call the summarizer instead of falling back to `KeepFirstLast`
- [x] 4.3 Add `summarization_model` config option and fall back to current model when not set
- [x] 4.4 Emit `ContextAction` event with `summary_generated: true` when summarization occurs
- [x] 4.5 Add unit test: verify summarization is triggered when token count exceeds threshold

## 5. P1: OpenAI Models Endpoint

- [x] 5.1 Add `GET /v1/models` handler in `src/server.rs` returning OpenAI-format model list from catalog
- [x] 5.2 Filter response to only include models from configured providers
- [x] 5.3 Add `GET /v1/models/{model_id}` handler returning single model details
- [x] 5.4 Return 404 with OpenAI-compatible error for unknown models
- [x] 5.5 Add integration test: verify `/v1/models` returns expected structure

## 6. P1: Prometheus Metrics

- [x] 6.1 Add `metrics` and `metrics-exporter-prometheus` crates to `Cargo.toml`
- [x] 6.2 Create `src/uar/telemetry/metrics.rs` module defining all metric descriptors
- [x] 6.3 Add Axum middleware layer to record `uar_requests_total` and `uar_request_duration_seconds`
- [x] 6.4 Instrument LLM driver to record `uar_llm_tokens_total` per provider/model/direction
- [x] 6.5 Instrument MCP registry to record `uar_tool_calls_total` per tool/status
- [x] 6.6 Add `uar_active_sessions` gauge to session store
- [x] 6.7 Add `GET /metrics` endpoint (unauthenticated) serving Prometheus text format
- [x] 6.8 Verify metrics with `curl /metrics | promtool check metrics`

## 7. P1: Agent Status Labels in Chat UI

- [x] 7.1 Add `AgentStatusIndicator` React component with status text and fade animation
- [x] 7.2 Map SSE events to status states: `thinking` (message.delta), `executing` (tool_call.start), `searching` (search-related tools)
- [x] 7.3 Integrate status indicator into `EnhancedThread` component below the last message
- [x] 7.4 Clear status on `done` event with fade-out animation
- [x] 7.5 Add Zustand state slice for current agent status

## 8. P1: A2UI Testing UI in Admin

- [x] 8.1 Create `frontend/src/admin/A2uiTestingPage.tsx` component
- [x] 8.2 Fetch and display all registered A2UI schemas from `GET /api/uar/a2ui/schemas`
- [x] 8.3 Implement live preview renderer for each schema type (form, confirm, select, text-input, display)
- [x] 8.4 Add JSON editor for custom schema input with validation
- [x] 8.5 Show submission payload preview when user fills in artifact fields
- [x] 8.6 Add route and sidebar navigation entry for the new page

## 9. P2: Light Mode Toggle

- [x] 9.1 Define light mode CSS custom property values in `frontend/src/index.css` (all HSL tokens)
- [x] 9.2 Create `ThemeProvider` component using Zustand + localStorage + `prefers-color-scheme`
- [x] 9.3 Add theme toggle button to the top navigation bar
- [ ] 9.4 Verify WCAG 2.1 AA contrast ratios for all light mode component combinations
- [x] 9.5 Update Tailwind config `darkMode: "class"` setup (already correct, verify)

## 10. P2: Network Policies

- [x] 10.1 Create `k8s/base/network-policies.yaml` with default-deny ingress policy
- [x] 10.2 Add NetworkPolicy allowing UAR → PostgreSQL (port 5432)
- [x] 10.3 Add NetworkPolicy allowing UAR → Redis (port 6379)
- [x] 10.4 Add NetworkPolicy allowing UAR → SurrealDB (port 8000)
- [x] 10.5 Add NetworkPolicy allowing Envoy Gateway → UAR (port 3000)
- [x] 10.6 Add NetworkPolicy allowing Prometheus scraping → UAR `/metrics` (port 3000)

## 11. P2: Tool Approval Workflow

- [x] 11.1 Add Cedar policy rule schema for `requires_approval` tool tagging
- [x] 11.2 Add approval check in tool execution path (`src/uar/runtime/manager.rs`) before MCP `call_tool`
- [x] 11.3 Emit `agui.tool_call.approval_required` SSE event with tool name, args, risk reason
- [x] 11.4 Add `POST /api/uar/runs/{run_id}/tool-approval` endpoint accepting `{"approved": bool}`
- [x] 11.5 Implement run pause/resume mechanism (channel-based wait for approval)
- [x] 11.6 Create `ToolApprovalDialog` React component with approve/reject buttons
- [x] 11.7 Add 5-minute auto-reject timeout
- [ ] 11.8 Add integration test: verify tool call pauses and resumes on approval

## 12. P2: Context Visualization

- [x] 12.1 Add backend endpoint `GET /api/uar/sessions/{id}/context-stats` returning token usage, strategy, threshold
- [x] 12.2 Create `ContextUsageBar` React component with progress bar and token counts
- [x] 12.3 Update bar color based on threshold (green → amber → red)
- [x] 12.4 Display active context strategy label (SlidingWindow, KeepFirstLast, etc.)
- [x] 12.5 Show inline notification when `agui.context.update` event fires with compression stats
- [x] 12.6 Integrate into chat interface header or footer area

## 13. P2: Tiktoken Token Estimation

- [x] 13.1 Add `tiktoken-rs` crate to `Cargo.toml`
- [x] 13.2 Update `TokenService::estimate_string()` to use tiktoken for OpenAI models (`cl100k_base`, `o200k_base`)
- [x] 13.3 Implement 4:1 char-to-token fallback ratio for non-OpenAI models
- [x] 13.4 Update `estimate_messages()` to include per-message overhead (4 tokens)
- [x] 13.5 Add unit tests comparing tiktoken output against known token counts

## 14. P2: Helm Chart

- [x] 14.1 Create `k8s/helm/uar/Chart.yaml` with chart metadata
- [x] 14.2 Create `k8s/helm/uar/values.yaml` with all configurable parameters
- [x] 14.3 Create templates: `deployment.yaml`, `statefulset-postgres.yaml`, `statefulset-surrealdb.yaml`, `deployment-redis.yaml`
- [x] 14.4 Create templates: `service.yaml` (all 4), `configmap.yaml`, `secrets.yaml`
- [x] 14.5 Create templates: `pvc.yaml`, `storageclass.yaml`, `hpa.yaml`, `serviceaccount.yaml`, `networkpolicy.yaml`, `httproute.yaml`
- [x] 14.6 Create `_helpers.tpl` with common label and name templates
- [ ] 14.7 Verify `helm lint k8s/helm/uar/` passes
- [ ] 14.8 Verify `helm template uar k8s/helm/uar/` produces valid YAML

## 15. P2: HPA Autoscaling

- [x] 15.1 Create `k8s/base/uar-hpa.yaml` with CPU target 70%, min 2, max 10 replicas
- [x] 15.2 Add memory utilization target as secondary metric
- [x] 15.3 Configure scale-down stabilization window (300s)

## 16. P3: MCP Health Dashboard

- [x] 16.1 Add `GET /api/uar/mcp/health` backend endpoint returning all server statuses
- [x] 16.2 Implement health check in `McpRegistry` that pings each server (list_tools or HTTP health)
- [x] 16.3 Create `frontend/src/admin/McpHealthPage.tsx` component with server list, status indicators, tool counts
- [x] 16.4 Add refresh button and auto-refresh interval (30s)
- [x] 16.5 Add route and sidebar navigation entry

## 17. P3: Skill Hot-Reload

- [x] 17.1 Add filesystem watcher (`notify` crate) on the skills directory
- [x] 17.2 On file change, parse and re-register/deregister skills in the `SkillRegistry`
- [x] 17.3 Add `POST /api/uar/skills/reload` endpoint for manual trigger
- [x] 17.4 Ensure active runs use a snapshot of skill state (not affected by mid-run reload)
- [ ] 17.5 Add integration test: add skill file, verify it appears in registry

## 18. P3: OpenAPI Documentation

- [x] 18.1 Add `utoipa` and `utoipa-swagger-ui` crates to `Cargo.toml`
- [x] 18.2 Annotate key API handlers with `#[utoipa::path(...)]` macros
- [x] 18.3 Create `ApiDoc` struct implementing `utoipa::OpenApi` with all tagged paths
- [x] 18.4 Mount Swagger UI at `GET /api/docs` and JSON spec at `GET /api/openapi.json`
- [x] 18.5 Verify generated spec covers `/v1/chat/completions`, `/v1/models`, `/api/uar/*` endpoints

## 19. P3: PWA Service Worker

- [x] 19.1 Create service worker file (`frontend/public/sw.js`) with precache strategy for static assets
- [x] 19.2 Register service worker in `frontend/src/main.tsx
- [x] 19.3 Add offline detection and "You are offline" banner component
- [x] 19.4 Update `manifest.json` with PWA install metadata (icons, theme_color, display: standalone)
- [ ] 19.5 Verify Lighthouse PWA audit passes basic requirements

## 20. P3: A2A v0.3 gRPC Transport

- [x] 20.1 Add `tonic` and `prost` crates to `Cargo.toml`
- [x] 20.2 Create A2A proto file (`proto/a2a.proto`) defining `MessageSend`, `TaskGet`, `TaskCancel`, `MessageStream` RPCs
- [x] 20.3 Generate Rust types with `tonic-build` in `build.rs`
- [x] 20.4 Implement gRPC service struct delegating to existing A2A handler logic
- [x] 20.5 Add gRPC server on configurable port (default 50051) alongside HTTP server
- [ ] 20.6 Add integration test: send A2A message via gRPC, verify task state

## 21. GKE Deployment: Dockerfile Update

- [x] 21.1 Update Rust builder stage base image to `rust:1.87-slim-bookworm` (or latest stable)
- [x] 21.2 Update frontend builder stage to use current `package.json` and Vite build process
- [x] 21.3 Verify `docker build -t uar:latest .` succeeds with updated stages
- [x] 21.4 Optimize layer caching: separate dependency download from source compilation

## 22. GKE Deployment: Kustomize Base Manifests

- [x] 22.1 Create `k8s/base/kustomization.yaml` listing all resource files
- [x] 22.2 Create `k8s/base/namespace.yaml` defining the `uar` namespace
- [x] 22.3 Create `k8s/base/storage-class.yaml` with `volumeBindingMode: Immediate` and GCE PD provisioner
- [x] 22.4 Create `k8s/base/uar-deployment.yaml` with resource limits, probes, env from ConfigMap/Secrets, volume mounts
- [x] 22.5 Create `k8s/base/uar-service.yaml` (ClusterIP, port 3000)
- [x] 22.6 Create `k8s/base/uar-serviceaccount.yaml` with minimal RBAC
- [x] 22.7 Create `k8s/base/postgres-statefulset.yaml` with pgvector image, PVC (immediate binding), init scripts
- [x] 22.8 Create `k8s/base/postgres-service.yaml` (ClusterIP, port 5432)
- [x] 22.9 Create `k8s/base/surrealdb-statefulset.yaml` with RocksDB backend, PVC (immediate binding)
- [x] 22.10 Create `k8s/base/surrealdb-service.yaml` (ClusterIP, port 8000)
- [x] 22.11 Create `k8s/base/redis-deployment.yaml` with AOF persistence, maxmemory config
- [x] 22.12 Create `k8s/base/redis-service.yaml` (ClusterIP, port 6379)
- [x] 22.13 Create `k8s/base/configmap.yaml` with non-sensitive configuration (server port, log format, memory settings)
- [x] 22.14 Create `k8s/base/secrets.yaml` with placeholder structure for API keys, DB credentials, JWT secret
- [x] 22.15 Create `k8s/base/httproute.yaml` referencing existing Envoy Gateway for UAR service routing
- [x] 22.16 Verify `kubectl kustomize k8s/base/` produces valid YAML

## 23. GKE Deployment: Production Overlay

- [x] 23.1 Create `k8s/overlays/production/kustomization.yaml` with base reference and production patches
- [x] 23.2 Create production patches for replica count, resource limits, and environment-specific config
- [x] 23.3 Verify `kubectl kustomize k8s/overlays/production/` produces valid YAML

## 24. GitHub Actions: Build and Deploy Workflow

- [x] 24.1 Create `.github/workflows/deploy.yml` with triggers: push to main, workflow_dispatch
- [x] 24.2 Add `build` job: checkout, authenticate to GCP (workload identity or SA key), build Docker image, push to Artifact Registry
- [x] 24.3 Add `deploy` job: update image tag in `k8s/base/uar-deployment.yaml`, commit and push manifest change
- [x] 24.4 Add `smoke-test` job: wait for ArgoCD sync, `kubectl rollout status`, curl `/readyz`
- [x] 24.5 Add GCP authentication step using `google-github-actions/auth` with credentials reference
- [x] 24.6 Add Artifact Registry configuration (repository URL, image naming convention)
- [x] 24.7 Add workflow_dispatch input for optional image tag override
- [x] 24.8 Verify workflow YAML is valid with `actionlint` or manual review
