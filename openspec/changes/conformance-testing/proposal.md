# CH-14 conformance-testing

## Why

CH-12 (agent-spec-v2) let an agent author *declare* what their compiled
agent needs — a model with certain capabilities, a specific prompt
dialect, a message-history trimming strategy — but nothing checked
whether those declarations were actually satisfiable against a real
deployment, or whether the runtime functions that make routing/dialect/
context decisions would actually honor them. `assessment.md` found zero
hits for "conformance" anywhere in `src/` or `tests/` — this is genuinely
new test infrastructure, not an extension of something existing.

## What changed

New module `src/uar/compiler/conformance.rs`, exposing
`check_conformance(ir: &AgentDescriptorIR, router: &ModelRouter) ->
ConformanceReport`. It checks three of the five v2 sections against the
real runtime functions a production request would call — not a
re-implementation of their logic:

- **`model_requirements`** (load-time): builds a `RouteRequirements` from
  the declared section and calls the actual `ModelRouter::route()` against
  the caller's real `ProviderRegistry`. `Some(model)` → satisfiable;
  `None` → unsatisfiable, with the resolved model recorded for the dialect
  check below.
- **`prompt_dialect`** (run-time): if an explicit `dialect` override is
  declared, compares it against `PromptDialect::detect(resolved_model)` —
  the same detection function the real request path uses. The resolved
  model is either an explicit deployment-profile pin
  (`deployment.profiles[].provider.model`, which takes priority — an
  operator's pin means that pin) or whatever `model_requirements` routed
  to.
- **`context_strategy`** (run-time): converts the declared section into
  the runtime `uar::context::strategy::ContextStrategy` type (unset fields
  fall back to the exact same defaults the runtime itself uses — see
  "Incidental refactor" below, not duplicated magic numbers) and feeds a
  synthetic 10-message transcript through the real `apply_strategy()`,
  asserting the trim result matches what that variant's own semantics
  predict.

`rag_configuration` and `api_harness` are out of scope: they're
declarative-only sections with no corresponding runtime *decision* to
conform against yet (RAG posture is read directly by the retrieval
pipeline; API harness just advertises transport support).

**Design choice — "not declared" vs. "declared and failing":** a v2
section left at its parsed default is indistinguishable from "the author
wrote nothing here" (`#[serde(default)]` produces the same value either
way), so `CheckResult::NotDeclared` is reported instead of a pass/fail
verdict — there's nothing to conform *to*. This applies to
`ContextStrategySection::Auto` too (the section's own `#[default]`
variant): the runtime's own model-aware selection (CH-05) applies, with
nothing fixed declared to conform to.

## Incidental refactors (small, in service of correctness/reuse)

- `PromptDialect::name(self) -> &'static str` added to
  `src/llm/prompt_dialect.rs` — the inverse of `detect()`'s model-id
  sniffing, kept in the same module so the two can't drift apart.
- The 8 `default_*` functions in `src/uar/context/strategy.rs` (e.g.
  `default_max_messages`) changed from private to `pub(crate)` so the
  conformance harness reuses them as the single source of truth for "what
  does the runtime default to" instead of duplicating those numbers.
- `ModelRequirementsSection` gained `#[derive(PartialEq)]` so the harness
  can cheaply detect "left at default" via equality.
- `parser.rs`'s `minimal_agent_md()` test fixture moved from inside its
  private `mod tests` to module scope as `pub(crate)` (still
  `#[cfg(test)]`-gated) so this change's own tests build IR fixtures by
  appending v2-section YAML to it, instead of duplicating all 15 required
  v1.1 sections.

## Verification

- `cargo check --lib`: clean.
- `cargo test --lib compiler::conformance::`: 10/10 green (covers: all-
  default → NotDeclared; model_requirements satisfiable/unsatisfiable;
  prompt_dialect match/mismatch/no-resolvable-model; context_strategy
  sliding-window/truncate-middle honored; Auto → NotDeclared).
- `cargo test --lib`: 360/360 green (confirms the `parser.rs` fixture
  relocation and the `strategy.rs`/`ir.rs`/`prompt_dialect.rs` visibility
  changes didn't regress anything).
- `cargo clippy --lib`: zero new warnings attributable to
  `conformance.rs` (all pre-existing warnings elsewhere in the crate are
  unrelated to this change).
