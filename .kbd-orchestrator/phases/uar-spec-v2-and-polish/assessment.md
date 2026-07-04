# Assessment — uar-spec-v2-and-polish

**Date:** 2026-07-04
**Method:** codebase inspection (grep/read against actual source), not
assumption — per the explicit lesson recorded in `uar-next-harness`'s
reflection ("verify 'already committed' against git log, not just
progress.json"; several changes in the prior phase turned out to be
committed-but-unwired or genuinely-unstarted despite looking done).

## Per-change findings

### CH-12 agent-spec-v2 — NOT STARTED

`AgentDescriptorIR` (`src/uar/compiler/ir.rs:20-68`) has 14 sections
(metadata, identity, ui, capabilities, skills, tools, mcp_servers,
knowledge, memory, a2a, governance, budgets, execution, observability,
deployment). None of the five target v2 fields (`model_requirements`,
`prompt_dialect`, `rag_configuration`, `context_strategy`, `api_harness`)
exist. A versioning seam already exists (`version: String` and
`schema_version: Option<String>` at `ir.rs:244,246`), giving a natural,
low-risk path to backward-compatible v1.1 parsing — this is a real
advantage over building versioning from scratch.

### CH-13 compiler-v2-stages — NOT STARTED (clear target exists)

`src/uar/compiler/stages/` has 8 wired PMPO stages: `s01_frontmatter.rs`
through `s08_emit.rs` (a2ui, mcp, a2a_schemas, cedar, actor_endpoints, pep,
emit). `s08_emit.rs` is exactly the "descriptor emit/sign" stage CH-13
needs to extend for the five new v2 sections — the stage pipeline
architecture is proven and doesn't need to be invented, just extended.
Depends on CH-12 landing first.

### CH-14 conformance-testing — NOT STARTED

Zero hits for "conformance" anywhere in `src/` or `tests/`. Depends on
CH-13.

### CH-15 agent-template-library — NOT STARTED

Zero `.agent.md` template files anywhere in the repo. Depends on CH-13.

### CH-17 eval-targeted-suites — NOT STARTED

`evals/` contains only `README.md` and `starter.yaml` (the general harness
from `uar-eval-harness`/`eval-harness-hardening`, both already-closed
phases). No skill-activation/routing/context-efficiency-specific suite
files exist yet. Depends on CH-08 (skill-activation-metrics, done) and
CH-09 (capability-registry-benchmarks, done) — both prerequisites are
already satisfied, so this is unblocked once its own scope starts.

### CH-19 docs-overhaul-deploy-guide — PARTIALLY EXISTS

`docs/ARCHITECTURE.md` exists (269 lines) and references `ModelRouter` in
its file-tree listing, but does not narratively document the dialect
engine, health monitoring, or cost-budget systems built in
`uar-next-harness`. `k8s/helm/` charts already exist on disk — the
deployable artifacts are there — but no consolidated production
deployment guide/narrative doc exists tying them together. Meaningfully
less work than "write from scratch": this is an update-and-consolidate
task, not a greenfield one.

### CH-20 perf-security-load — NOT STARTED

No `benches/` directory, no k6/locust/load-test references anywhere. No
prompt-injection test suite found (beyond the guardrails heuristic
already shipped in an earlier phase, which is a mitigation, not a test
suite proving it). `src/server.rs` is now **5,068 lines** (grew from the
previously-noted 4,922 during `uar-next-harness`'s work) — reinforcing
that the "split evaluation" item in this change's scope is warranted, not
speculative.

## Summary

Unlike `uar-next-harness` (where the phase's own history had already
delivered most of G1-G3 without anyone updating the tracker), this phase's
7 changes are genuinely, verifiably unstarted work — a real greenfield
engineering effort, not a reconciliation exercise. The one exception is
CH-19, which is a documentation *update* against already-existing
artifacts (ARCHITECTURE.md, k8s/helm charts) rather than a from-scratch
write.

The dependency chain from plan.md/goals.md holds: CH-12 → CH-13 →
{CH-14, CH-15} (parallel) → CH-17 is genuinely sequential where claimed
(CH-13 needs CH-12's IR fields to extend the emit stage; CH-14/CH-15 both
need CH-13's stage plumbing; CH-17 only needs CH-08/CH-09, both already
done, so it does NOT strictly need to wait for CH-12/13/14/15 despite
being listed after them in G4 — it can run in parallel with the CH-12-15
chain). CH-19 and CH-20 (G5) are independent of each other and of the G4
chain except that CH-19's "document what exists" scope benefits from CH-12
onward having landed first (nothing to document otherwise) — CH-20's
perf/security work is fully independent and could start immediately.
