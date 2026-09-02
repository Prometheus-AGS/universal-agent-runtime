<!-- mirror of openspec/changes/typed-turn-assembly/tasks.md -->
# Tasks — typed-turn-assembly

scope: src/uar/runtime/manager.rs, src/llm/orchestrator.rs (per-step inputs), src/server.rs (memory prepend), src/config.rs, src/uar/runtime/turn/**, tests/typed_turn_assembly.rs, tests/turn_shadow_parity.rs

## 1. Failing tests first

- [ ] 1.1 `tests/typed_turn_assembly.rs`: a `RunExecutionRequest` built from the legacy positional arguments resolves to a `ResolvedTurn` whose fragments, policy, and credentials equal the legacy path's
- [ ] 1.2 A contributor that attempts to add a tool outside the effective policy is rejected at assembly with a typed error
- [ ] 1.3 After `activate_skill` on step 1, the `ResolvedStep` for step 2 contains the skill's tools; the frozen legacy list did not
- [ ] 1.4 A direct `start_run` call with memory hits produces a memory fragment (today it produces none)
- [ ] 1.5 `tests/turn_shadow_parity.rs`: in `shadow` mode, every request in the corpus reports zero differences outside the intentional-delta allowlist (`tests/fixtures/turn_parity/intentional_deltas.json`, each entry naming the dependency change that introduces it), every allowlisted difference is observed at least once, and only the legacy request is dispatched

## 2. Types and registry

- [ ] 2.1 Add `turn/request.rs` (`RunExecutionRequest`), `turn/plan.rs` (`TurnAssemblyPlan`), `turn/resolved.rs` (`ResolvedTurn`, `ResolvedStep`)
- [ ] 2.2 Add `turn/contributors.rs` with the seven stage traits and `ContributorRegistry`; enforce narrow-only on policy and Cedar consultation out of band
- [ ] 2.3 Add `HarnessConfig { mode }` to config and settings schema

## 3. Compose

- [ ] 3.1 Move artifact instructions, effective policy, memory, RAG, skills, MCP/tools, and context into contributors using the functions extracted by changes 1 to 5
- [ ] 3.2 Orchestrator takes a `ResolvedStep` per model call; tool set and active skills re-projected each step
- [ ] 3.3 Adapters: `start_run*` and route decoding build `RunExecutionRequest`

## 4. Shadow and flip

- [ ] 4.1 Add `turn/shadow.rs`: render both paths, diff, classify each difference as allowlisted or unexpected, record in manifest, dispatch legacy
- [ ] 4.2 Add the parity corpus and `intentional_deltas.json`; run at Tier 2 and produce `parity-report.json` (unexpected-difference count per request)
- [ ] 4.3 Default remains `legacy`; the flip is `typed-turn-default-flip`, which consumes the parity report and live-smoke evidence

## 5. Verification

- [ ] 5.1 Tier 1: the two new test targets
- [ ] 5.2 Tier 2: fmt check and full test run
- [ ] 5.3 `openspec validate typed-turn-assembly --strict`
