# Production Deployment Guide

This consolidates the deployable artifacts that already exist in this
repository — the multi-stage `Dockerfile`, the `k8s/helm/uar` chart, and the
GitHub Actions deployment workflow — into one narrative. **It documents what's
here, including a real inconsistency between the Helm chart and the current
live deployment path** (see [Two deployment paths](#two-deployment-paths-read-this-first)
below) rather than paper over it.

## Native services

`packaging/native/` installs the `server-full` release and React bundle as a
supervised service. Native defaults bind HTTP `1906` and A2A gRPC `50051` to
`127.0.0.1`; both listeners inherit `server.host`.

Build once from the repository root:

```bash
pnpm install --frozen-lockfile
pnpm build
cargo build --locked --release --no-default-features --features server-full
```

### macOS user LaunchAgent

```bash
source "$HOME/.bash_profile"
packaging/native/macos/install.sh \
  --binary target/release/universal-agent-runtime \
  --static-dir static
packaging/native/macos/control.sh status
```

The label is `com.prometheus.universal-agent-runtime`. Program, YAML,
environment, static assets, and state live under `~/.uar`. Logs are restricted
to `~/.prometheus/logs/universal-agent-runtime/`; backups go to
`~/.prometheus/backups/uar/`. `control.sh` supports `start`, `stop`, `restart`,
and `status`. `upgrade.sh` accepts the install build arguments.
`refresh-credentials.sh` regenerates only approved provider variables from the
current shell. `uninstall.sh` removes the service/program while preserving
configuration, database state, environment, backups, and logs.

### Linux systemd

```bash
sudo --preserve-env=KIMI_API_KEY,KIMI_CODING_API_KEY,KIMI_CODING_KEY,MINIMAX_API_KEY,MINIMAX_KEY,DASHSCOPE_API_KEY,QWEN_API_KEY,QWEN_TOKEN_PLAN_API_KEY,MOONSHOT_API_KEY,ZAI_API_KEY \
  packaging/native/linux/install.sh \
  --binary target/release/universal-agent-runtime \
  --static-dir static
sudo packaging/native/linux/control.sh status
```

The unit is `uar.service`. Configuration is under `/etc/uar`; state and logs
are under `/var/lib/uar`, with operator logs in
`/var/lib/uar/.prometheus/logs/`; program files are in `/usr/local/lib/uar`.
The unit uses direct `ExecStart`, `SIGTERM`, and `Restart=on-failure`.
Uninstall preserves `/etc/uar` and `/var/lib/uar`.

### Windows SCM

From an elevated PowerShell session containing approved provider variables:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\packaging\native\windows\install.ps1 `
  -Binary .\target\release\universal-agent-runtime.exe `
  -StaticDir .\static
.\packaging\native\windows\control.ps1 status
```

The SCM name is `PrometheusUniversalAgentRuntime`, running as
`NT AUTHORITY\LocalService`. Program files live below
`%ProgramFiles%\Prometheus\UniversalAgentRuntime`; configuration, state, and
logs live below `%ProgramData%\Prometheus\UniversalAgentRuntime`, with logs in
`.prometheus\logs`. `.cmd` wrappers accompany the PowerShell entrypoints.
Uninstall preserves ProgramData.

### Preservation and troubleshooting

Install and upgrade back up existing YAML before adding only absent native
listener/provider entries. They preserve operator values and database-backed
provider/default-model settings except for the exact native Alibaba migration
from `alibaba/qwen3.7-max` to released `alibaba/qwen3.8-max`, the exact
phase-owned `qwen3-coder-plus` seed, and the malformed
`QWEN_TOKENPLAN_API_KEY` reference. The generated service
environment contains only canonical Kimi, MiniMax, DashScope, Moonshot, and
Z.AI credentials resolved from the invoking process. Canonical names win over
documented aliases; no secret is written to YAML or printed.

On failure, use the platform `status` command, inspect its `.prometheus` log
path, validate the YAML, and query `/healthz` and `/readyz`. Those probes do not
prove inference. Linux and Windows packages are structure/cross-compile checked
from macOS; runtime claims require observation on the target platform.

## Container image

`Dockerfile` is a polyglot multi-stage build: a `toolchain` stage with five
language toolchains (Rust nightly + `wasm32-wasip2`/`wasm32-wasip1`/
`wasm32-unknown-unknown`, Node.js LTS ≥24, Python 3.13, Go, `wasmtime`) so
skills/plugins authored in any UAR-supported language can be compiled
in-container, and a `runtime` stage that carries the same toolchains forward
(intentionally ~3GB — there is no slim variant yet). Build locally with:

```bash
docker buildx build --platform linux/amd64 -t uar:latest .
```

Build and verify candidate images locally. **`.github/workflows/deploy.yml`**
("Build and Deploy to AKS") may build the deployment image only as part of an
actual deployment; its build path must not invoke product tests or routine
development checks directly or indirectly.

## Two deployment paths (read this first)

This repository currently contains **two independent deployment
mechanisms that target different clouds and do not share configuration**:

1. **The live deployment path — Azure AKS, image-bump only.**
   `.github/workflows/deploy.yml` builds the image, pushes it to Azure
   Container Registry (`prometheusagsacr.azurecr.io`), then runs
   `kubectl set image deploy/uar uar=<acr>/<image>:<tag> -n uar` against an
   **operator-managed** AKS cluster (`main` cluster, `prometheus-rg` resource
   group). The `uar` namespace's ConfigMap and Secrets predate this repo and
   are deliberately **not** touched (`kubectl apply -k` is intentionally not
   used) — this workflow only bumps the running image. Rollout uses a
   scale-to-0-then-1 dance (not a rolling update) because the deployment uses
   ReadWriteOnce PVCs, which deadlock a RollingUpdate on a Multi-Attach error.
   `docs/ci-gke-deploy-secrets.md` documents secrets for a **GKE**-based
   variant of this same workflow that predates the current AKS version — it
   does not match `deploy.yml` as it stands today; treat it as historical
   unless you're specifically reintroducing a GKE deploy path.
2. **The Helm chart — self-contained, GKE-oriented, not currently wired to CI.**
   `k8s/helm/uar/` is a complete chart with its own Postgres, SurrealDB, and
   Redis subcharts, an Envoy Gateway `HTTPRoute`, an HPA, and network
   policies. Its `storageClass` template (`pd.csi.storage.gke.io`) is
   GKE-specific. No deployment workflow currently runs `helm install`/`helm upgrade`
   against it — it's the right starting point for a **fresh** cluster
   bootstrap (a new environment, a GKE target, or local `kind`/`minikube`
   testing), not a description of what's currently live.

If you're standing up a new environment, start from the Helm chart (below)
and adjust `storageClass` for your provider. If you're changing what's live
in the existing AKS cluster, that's `deploy.yml`'s image-bump path, and the
chart is not involved.

## Deploying via the Helm chart

```bash
helm install uar k8s/helm/uar/ \
  --namespace uar --create-namespace \
  --set uar.image.repository=<your-registry>/universal-agent-runtime \
  --set uar.image.tag=<tag> \
  --set-file uar.secrets.UAR_LLM__API_KEY=./llm-api-key.b64
```

Key `values.yaml` knobs:

| Key | Default | Notes |
|---|---|---|
| `uar.replicaCount` | `2` | The chart's own default assumes a `ReadWriteMany`-capable storage class or a stateless deployment; the live AKS path above runs single-replica with RWO volumes — pick a replica count consistent with your storage class |
| `uar.resources` | `250m`/`256Mi` req, `1`/`1Gi` limit | Tune per workload; LLM streaming is I/O-bound, not CPU-bound |
| `uar.livenessProbe` / `readinessProbe` | `/healthz`, `/readyz` | Same endpoints the AKS smoke-test job checks post-deploy |
| `uar.env` | `UAR_SERVER__*`, `UAR_MEMORY__*`, `DATABASE_URL`, `RUST_LOG` | Merge with your own `UAR_LLM__MODEL` / provider config — see `.env.example` |
| `uar.secrets` | placeholder `CHANGEME` base64 values | **Must** be overridden — `UAR_LLM__API_KEY`, `UAR_SECURITY__JWT_SECRET`, `POSTGRES_PASSWORD`, `SURREAL_USER`, `SURREAL_PASS` |
| `postgres.enabled` / `surrealdb.enabled` / `redis.enabled` | all `true` | Set to `false` and point `DATABASE_URL`/`UAR_MEMORY__SURREAL_ENDPOINT` at externally-managed instances if you don't want the chart to own stateful services |
| `storageClass.provisioner` | `pd.csi.storage.gke.io` | **Change this for non-GKE clusters** (e.g. `disk.csi.azure.com` for AKS, `ebs.csi.aws.com` for EKS) |
| `gateway.enabled` | `false` | Envoy Gateway `HTTPRoute`; set `gateway.hostname` when enabling |
| `hpa.enabled` | `true`, `2`–`10` replicas | Horizontal Pod Autoscaler |
| `networkPolicies.enabled` | `true` | Namespace-scoped `NetworkPolicy` resources |

## SurrealDB 2.x to 3.2.4 migration gate

Every checked-in Compose, Kustomize, Helm, and OpenTofu SurrealDB workload is
pinned to
`surrealdb/surrealdb:v3.2.4@sha256:51baed8709f57f67dcf04b30e3177db846803fa9342dae2be58c6fa5f8d59843`.
That pin is safe for a new datastore. It does **not** make an existing 2.x data
directory readable by 3.x. The uncomfortable failure mode is an operator
replacing the image first and discovering only then that the 3.x server cannot
open the 2.x store.

Use the official
[SurrealDB 2.x-to-3.x procedure](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/build/migrating/from-old-surrealdb-versions/2x-to-3x.mdx)
before applying these manifests to a 2.x-backed environment:

1. Quiesce application writes, capture a restorable backup of the 2.x volume,
   and record representative namespace/database counts and queries. Resolve the
   migration diagnostics, including every item that requires manual changes.
2. Keep the 2.x server running against its original volume. From the SurrealDB
   3.2.4 CLI, create a v3-compatible export:

   ```bash
   surreal v2 export --v3 \
     --namespace <namespace> \
     --database <database> \
     --token <v2-token> \
     v2-export-for-v3.surql
   ```

3. Start an empty 3.2.4 target on a different endpoint and a new volume. Import
   the export; never point the 3.x process at the 2.x data directory:

   ```bash
   surreal import \
     --namespace <namespace> \
     --database <database> \
     --endpoint <v3-endpoint> \
     --token <v3-token> \
     v2-export-for-v3.surql
   ```

4. Repeat the recorded counts and representative queries against the 3.2.4
   target, then prove an authenticated create/read/query cycle before changing
   the application endpoint. Keep the 2.x volume and manifest available for
   rollback until the 3.2.4 verification window closes.

Rehearse the same sequence against disposable representative data before a
production window. A rehearsal proves the command path and catches schema
incompatibilities; it is not permission to migrate or delete production state.

## Configuration reference

Both paths ultimately configure the same binary via `UAR_*` environment
variables (see `.env.example` for the full list and `CLAUDE.md`'s
precedence order: CLI args > `UAR_*` env > legacy `LLM_*` env > provider-shortcut
env > `config.yaml` > compiled defaults). At minimum, a production
deployment needs: `UAR_LLM__MODEL` + `UAR_LLM__API_KEY` (or a provider
shortcut like `OPENAI_API_KEY`), `UAR_PERSISTENCE__PROVIDER` +
`UAR_PERSISTENCE__DATABASE_URL`, and `UAR_SECURITY__JWT_SECRET` if
`UAR_SECURITY__JWT_REQUIRED=true`.

## Health checks

- `GET /healthz` — liveness (process is up)
- `GET /readyz` — readiness (dependencies, e.g. persistence, are reachable)

Both the AKS smoke-test job and the Helm chart's probes check these same two
endpoints.

## See also

- `docs/ci-gke-deploy-secrets.md` — secrets for the historical GKE variant
  of the deploy workflow (see caveat above).
- `docs/DEPENDENCY_MANAGEMENT.md` — why several dependencies are pinned to
  git commits rather than crates.io versions (D-D).
- `docs/ARCHITECTURE.md` — system design, including the "Architectural
  Decisions" section (D-A through D-D).
- `docs/dev-tools.md` — `uar-jwt-proxy`, a local-only reverse proxy that
  injects a JWT for development; not a production auth gateway and out of
  scope for the deployment paths above.
