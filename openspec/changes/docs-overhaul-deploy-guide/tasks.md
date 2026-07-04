## 1. ARCHITECTURE.md narrative gaps

- [x] 1.1 "Provider Health & Failover" section (`ProviderHealthMonitor`,
      cooldown mechanism, who consults `is_available()`)
- [x] 1.2 "Prompt Dialect Engine" section (`PromptDialect::detect`,
      `PromptDialectEngine::request_params`)
- [x] 1.3 "Cost & Budget Tracking" section (`BudgetScope`, `BudgetLimit`,
      spend aggregation, cost-dashboard as the read surface)

## 2. Agent Spec v2 & Conformance section

- [x] 2.1 Table of all 5 v2 IR sections + which runtime type each mirrors
- [x] 2.2 Conformance harness summary (load-time vs run-time checks,
      `NotDeclared` design choice)
- [x] 2.3 Template library summary (4 templates, `compile` subcommand, CI
      release artifact, regression test)

## 3. Architectural Decisions

- [x] 3.1 D-A (RAG in-process) — one paragraph
- [x] 3.2 D-B (MemPalace off by default) — rationale + cross-reference to
      `Cargo.toml`'s own comment, not duplicated
- [x] 3.3 D-C (LibreFang scoped to UAR side) — cross-reference
      `docs/librefang-integration.md`
- [x] 3.4 D-D (dependency pins deliberate) — cross-reference
      `docs/DEPENDENCY_MANAGEMENT.md`, not duplicated

## 4. Source Layout update

- [x] 4.1 Added `health.rs`, `prompt_dialect.rs` under `llm/`
- [x] 4.2 Added `uar/compiler/{ir,parser,stages,pipeline,conformance,cli,signing}.rs`
      block
- [x] 4.3 Added `uar/runtime/cost_budget.rs` and `templates/`

## 5. Consolidated deployment guide (`docs/DEPLOYMENT.md`, new)

- [x] 5.1 Container image section (Dockerfile, build command, the two
      relevant CI workflows)
- [x] 5.2 "Two deployment paths" section disclosing the AKS-vs-GKE
      drift between `deploy.yml` and the Helm chart's storage class,
      confirmed via `git log --follow` on `deploy.yml` (originally
      GKE, rewritten to AKS) rather than assumed
- [x] 5.3 Helm chart `values.yaml` key table (replicas, resources,
      probes, env, secrets, subchart toggles, storage class, gateway,
      HPA, network policies)
- [x] 5.4 Configuration reference (env var precedence, minimum-required
      vars for a working deployment)
- [x] 5.5 Health-check endpoints section (`/healthz`, `/readyz`)
- [x] 5.6 Cross-references to `ci-gke-deploy-secrets.md` (flagged
      historical), `DEPENDENCY_MANAGEMENT.md`, `ARCHITECTURE.md`

## 6. Verify

- [x] 6.1 Every code fact (function/field/config-key names, defaults)
      checked against actual source, not assumed
- [x] 6.2 All cross-referenced docs confirmed to exist
- [x] 6.3 Markdown fence balance checked in both files
- [x] 6.4 Docs-only change — relies on the phase's existing green state
      (363/363 lib, 56/56 integration as of CH-15) rather than re-running
      the full suite for a non-code change

## 7. Phase closure

- [x] 7.1 This is the 7th and final candidate change for
      `uar-spec-v2-and-polish` — G4 (CH-12/13/14/15/17) and G5
      (CH-19/20) are both fully landed
