# Production Deployment Guide

This consolidates the deployable artifacts that already exist in this
repository — the multi-stage `Dockerfile`, the `k8s/helm/uar` chart, and the
GitHub Actions CI/CD workflows — into one narrative. **It documents what's
here, including a real inconsistency between the Helm chart and the current
live CI/CD path** (see [Two deployment paths](#two-deployment-paths-read-this-first)
below) rather than paper over it.

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

Two CI workflows build this image:

- **`.github/workflows/build-image.yml`** ("Build Image (no deploy)") —
  build-only, for verifying the Dockerfile compiles without deploying anywhere.
- **`.github/workflows/deploy.yml`** ("Build and Deploy to AKS") — builds
  **and** deploys; see below.

## Two deployment paths (read this first)

This repository currently contains **two independent deployment
mechanisms that target different clouds and do not share configuration**:

1. **The live CI/CD path — Azure AKS, image-bump only.**
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
   GKE-specific. No CI workflow currently runs `helm install`/`helm upgrade`
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
