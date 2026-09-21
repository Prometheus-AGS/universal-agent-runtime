## Context

See proposal.md for the observed gap. This is a Spec-stage design boundary, not an implemented result. The phase analysis selects reuse of the existing host and pins; the following Plan prerequisites remain binding.

scope: `src/llm/prompt_dialect.rs`, `src/llm/liter_driver.rs`, `src/llm/orchestrator.rs`, `src/uar/compiler/ir.rs`, `src/uar/runtime/prompt/`, `src/uar/runtime/manager.rs` and scenario fixtures under existing test roots. Plan must enumerate exact edit files before Execute. Shared files are edited in the phase index's total order; no concurrent writer to manager, orchestrator, prompt assembly or a build target.

## Goals / Non-Goals

Goals: satisfy this change's spec scenarios through existing governed production paths with local evidence.

Non-goals: replacement agent frameworks, dependency upgrades, UI redesign, kernel mutations, deployment or release. No product-code edit is performed by this Spec stage.

## Decisions

1. Reuse typed fragment assembly and the existing router; create one preparation contract called by all production entry paths. Preserve fixed section order and role authority. Model-specific layout varies only in declared compatible slots.

2. Resolve endpoint/model profiles with immutable revisions, template/counting identities and reviewed aliases. Descriptor/operator choices narrow under policy. Substring-only family selection is rejected because it cannot establish API compatibility.

3. Keep rendering data-only. Hugging Face chat formats and MiniJinja remain references; adopting an executable template engine is not required and is not authorized by these specs.

4. Apply settings and actual output ceilings at the driver boundary, then validate final serialized request size. Retry/failover starts from canonical history and current policy. Signed continuity restricts eligible destinations rather than being rewritten.

5. Reuse manifests for per-attempt profile/budget/selection evidence and account for summarizer/retry/child cost. Preserve existing semantic-commit retry boundaries and cursor replay behavior.

## Required planning prerequisites

Before implementing supported profiles, Plan must inventory configured provider/endpoint/model/revisions, retrieve current official documentation and capture allowed settings, roles, continuity, input/output semantics and exact/upper-bound counting methods. Analysis did not certify a complete provider matrix. Routing and compaction quality/cost/latency acceptance thresholds and dataset must be frozen before measuring improvement. Learned routing remains experimental.

Implementation follows harness-protected-context.

## Risks / Trade-offs

The uncomfortable scenario: A profile alias can drift upstream → pin reviewed provenance and revalidate before certifying compatibility. Exact counts may be unavailable → explicitly unsupported hard-bound dispatch rather than undocumented parameters. Rendering-only tests can miss driver mutation → capture final requests.

## Migration Plan

Introduce exact profiles alongside legacy dialect metadata, migrate documented destinations and connect every caller before claiming completion. Unsupported destinations return explicit diagnostics. Rollback restores only profiles that still satisfy protected-context constraints; no fallback to unsafe parameter guessing. All rollout and rollback steps are future implementation work with local validation; this artifact authorizes no deployment.

## Verification contract

Map every scenario to an executable fixture during Plan. Run Tier 0 per edit, Tier 1 per completed unit and Tier 2 at phase completion. Tier 3 belongs only to the separately registered local certification milestone. Request-level fixtures and production-path traces are required where specified; module presence and build success are not behavior proof. Artifact validation in this stage cannot establish runtime correctness.


## Plan resolution (2026-09-16)

The historical Spec-stage prerequisite list above is retained. Its concrete resolution and remaining execution gates are in `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`, `plan-research.md`, `execution-map.json`, `acceptance-contract.json`, `provider-profile-matrix.json` and `profile-certification.md` (repository-relative paths). The Plan manifest records current hashes; the earlier Spec manifest remains a historical snapshot. No implementation checkbox is completed by these planning artifacts.

- library: cand-001 — Existing UAR typed prompt/turn, routing, persistence and governed evaluation seams; verdict: adapt. Preserve capability inversion and established host integration; implement missing contracts rather than replacing runtime.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/prompt/assemble.rs — Inspected local source renders typed fragments with two family preference booleans; extend existing seam.
  Evidence: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/226d4a0af89811975662cf3f203c699f70e8cebc/src/uar/runtime/context/reduce.rs — Inspected reducer can drop severed tool groups; new protected preservation requires a stronger contract.
- library: cand-003 — Hugging Face model chat-template conventions; verdict: reference. Reference model-owned serialization semantics; do not introduce Transformers or Python into the host.
  Evidence: https://huggingface.co/docs/transformers/main/en/chat_templating — Official guide explains differing model control tokens and duplicate-special-token hazards.
  Evidence: https://huggingface.co/docs/transformers/main/en/chat_extras — Official guide describes tool schemas, calls and results within model chat formatting.
- library: cand-004 — MiniJinja; verdict: reference. Current typed renderer can implement versioned model templates; defer adding an engine until a concrete unsupported template requirement exists.
  Evidence: https://github.com/mitsuhiko/minijinja — Repository search identifies a Rust Jinja-compatible template engine.
  Evidence: https://github.com/mitsuhiko/minijinja/blob/main/src/template.rs — Context7 surfaced template inspection interfaces on main; no pinned compatibility or sandbox guarantee established.

Exact owned files, proposed fixtures, commands and per-task evidence directories are in execution-map.json. The reviewed order serializes shared files. No product behavior edit precedes its frozen baseline; no actual model profile is enabled before endpoint/count/output evidence; no phase completion precedes the separately registered local certification milestone.

### Production routing and predecessor boundary

Change 1 supplies common preparation and final output/envelope enforcement. This change adds exact model profiles/settings to those hooks. Task 2.4 connects a governed semantic classifier and authoritative host-derived requirements to RunManager routing; the observed default RouteRequirements construction must not remain the ordinary run-start path. Classification is a versioned soft ranking signal, never authority. The eight task-level cases in acceptance-contract.json measure classifications, provider outputs, task/tool assertions and usage separately from admission-only cases. Synthetic and live metrics remain explicitly separate.
