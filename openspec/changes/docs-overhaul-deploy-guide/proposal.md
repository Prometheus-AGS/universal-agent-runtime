# CH-19 docs-overhaul-deploy-guide

## Why

`assessment.md` found `docs/ARCHITECTURE.md` (269 lines) referenced
`ModelRouter` in its file-tree listing but never narratively covered the
provider-health/failover system, the prompt-dialect engine, or the
cost-budget system built across `uar-next-harness` — nor anything from
this phase's own agent-spec-v2/conformance/template-library work. The
`k8s/helm/` Helm chart and CI deploy workflows existed, but no document
tied them together into an operator-usable deployment narrative. This is
the last change in the phase (Tranche D), run after Rounds 1-4 so there
was maximum content to document — and, per carried-over
`uar-next-harness` debt, it also documents D-D (dependency-pin rationale)
and D-B (MemPalace status).

## What changed

**`docs/ARCHITECTURE.md`** (269 → 335 lines): three new narrative
sections under "LLM Layer" (Provider Health & Failover, Prompt Dialect
Engine, Cost & Budget Tracking), a new "Agent Spec v2 & Conformance"
top-level section covering this phase's own CH-12/13/14/15 work (the five
v2 IR sections and what runtime type each mirrors, the conformance
harness's load/run-time checks, the template library), a new
"Architectural Decisions" section giving each of D-A/B/C/D one paragraph
(D-B and D-D cross-reference existing docs rather than duplicating their
detail — `Cargo.toml`'s own `memory-palace` feature comment, and
`docs/DEPENDENCY_MANAGEMENT.md` respectively), and an updated Source
Layout tree including `health.rs`, `prompt_dialect.rs`,
`uar/compiler/conformance.rs`, `uar/compiler/cli.rs`, and `templates/`.

**`docs/DEPLOYMENT.md`** (new): the consolidated production deployment
guide. Container image build (the polyglot multi-stage `Dockerfile`),
Helm chart walkthrough (`values.yaml` key table: replicas, resources,
probes, env, secrets, subchart toggles, storage class, gateway, HPA,
network policies), configuration reference (env var precedence,
minimum-required vars), and health-check endpoints.

**Disclosed, not smoothed over:** while writing this, direct inspection
of `.github/workflows/deploy.yml` (git history) revealed it was
originally GKE-based (`gke-gcloud-auth-plugin` in its earliest commits,
matching `docs/ci-gke-deploy-secrets.md`) and was later fully rewritten to
target Azure AKS (`azure/login`, `azure/aks-set-context`, ACR) — while
`k8s/helm/uar`'s `storageClass` template is still GKE-specific
(`pd.csi.storage.gke.io`) and no CI workflow currently runs `helm
install`/`upgrade` against it at all. `DEPLOYMENT.md` documents this as
"two deployment paths" up front — the live AKS image-bump path (against
an operator-managed, pre-existing namespace) and the self-contained but
currently-unwired Helm chart (right for a fresh/different-cloud
bootstrap) — rather than presenting a single unified story that would
mislead an operator about what's actually live. `ci-gke-deploy-secrets.md`
is flagged as documenting the historical GKE variant, not the current
workflow.

## Verification

- Every code fact stated (function names, field names, config keys,
  default values) was checked against the actual source
  (`src/llm/health.rs`, `src/llm/prompt_dialect.rs`,
  `src/uar/runtime/cost_budget.rs`, `src/uar/compiler/ir.rs`,
  `k8s/helm/uar/values.yaml`, `.github/workflows/deploy.yml`'s git
  history) rather than assumed.
- All cross-referenced docs (`docs/librefang-integration.md`,
  `docs/DEPENDENCY_MANAGEMENT.md`, `docs/ci-gke-deploy-secrets.md`)
  confirmed to exist.
- Markdown fence balance checked (`grep -c '^```'` even in both files —
  9 pairs in `ARCHITECTURE.md`, 2 pairs in `DEPLOYMENT.md`).
- Docs-only change; no code touched, so no `cargo check`/`test` re-run
  was needed beyond the phase's existing green state (363/363 lib, 56/56
  integration as of CH-15).

This closes out `uar-spec-v2-and-polish`: all 7 candidate changes (G4:
CH-12/13/14/15/17, G5: CH-19/20) are now done.
