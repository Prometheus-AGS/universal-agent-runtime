## 1. Harness core

- [x] 1.1 `src/uar/compiler/conformance.rs`: `CheckResult` enum
      (`NotDeclared` / `Satisfied` / `Unsatisfied`) + `is_ok()` helper
- [x] 1.2 `ConformanceReport` struct + `all_satisfied()`
- [x] 1.3 `check_conformance(&AgentDescriptorIR, &ModelRouter) ->
      ConformanceReport` entry point
- [x] 1.4 Registered in `src/uar/compiler/mod.rs` (`pub mod conformance;`
      + re-exports)

## 2. `model_requirements` check (load-time)

- [x] 2.1 Builds `RouteRequirements` from the declared section, calls the
      real `ModelRouter::route()`
- [x] 2.2 `NotDeclared` when the section equals `ModelRequirementsSection::default()`
- [x] 2.3 Resolved model threaded through to the `prompt_dialect` check

## 3. `prompt_dialect` check (run-time)

- [x] 3.1 Compares declared `dialect` string against
      `PromptDialect::detect(resolved_model)`
- [x] 3.2 Deployment-profile explicit model pin takes priority over the
      router's pick as "the resolved model"
- [x] 3.3 `Unsatisfied` (not a panic) when a dialect is declared but no
      model could be resolved to check it against
- [x] 3.4 Added `PromptDialect::name()` (inverse of `detect()`) instead of
      duplicating the dialect-name string list a second time

## 4. `context_strategy` check (run-time)

- [x] 4.1 `to_runtime_strategy()`: IR section -> runtime `ContextStrategy`,
      unset fields fall back to the runtime's own `pub(crate) default_*`
      functions (not duplicated magic numbers)
- [x] 4.2 `expected_trim_len()`: independently-derived expected trim count
      per variant (not calling `apply_strategy` itself, so the test isn't
      a tautology)
- [x] 4.3 Synthetic 10-message transcript through the real
      `apply_strategy()`, compared against `expected_trim_len()`
- [x] 4.4 `Auto` (the section's own default) -> `NotDeclared`, consistent
      with the "can't distinguish declared-default from not-declared"
      rule applied to `model_requirements` too

## 5. Test fixture reuse

- [x] 5.1 Relocated `parser.rs`'s `minimal_agent_md()` out of its private
      `mod tests` to module scope (`pub(crate)`, still `#[cfg(test)]`)
- [x] 5.2 `conformance.rs` tests build fixtures via
      `minimal_agent_md() + extra v2-section YAML` through the real
      `parser::parse()`, not a hand-built `AgentDescriptorIR` literal
- [x] 5.3 Verified `parser.rs`'s own existing tests still pass after the
      relocation (8/8 green)

## 6. Verify

- [x] 6.1 `cargo check --lib` clean
- [x] 6.2 `cargo test --lib compiler::conformance::` 10/10 green
- [x] 6.3 `cargo test --lib` full suite 360/360 green
- [x] 6.4 `cargo clippy --lib` zero new warnings attributable to this
      change

## 7. Not this change (disclosed, out of scope)

- [ ] `rag_configuration` / `api_harness` conformance checks (declarative-
      only sections, no corresponding runtime decision to conform against
      yet)
- [ ] CH-15 agent-template-library (separate change, depends on CH-13 not
      CH-14 — no ordering dependency on this change, runs in the same
      tranche)
