# Bounded phase plan

## This phase: #462 exact-tree Rust baseline (dependency for #463–475)
Useful outcome: preserved presentation-history coverage compiles and a lightweight local safeguard prevents acceptance of uncompilable library test targets. Non-goals: runtime rewrites, provider calls, installed release certification, frontend changes, workflow/Actions edits.

Implementation ledger (lead owns Rust fixture and phase/spec files; baseline_safeguard owns script, hook and usage doc):
1. Record production contract, phase exit and OpenSpec delta; preserve prior waypoint/history.
2. Remove only the duplicate caller_handoff_confirmation initializer in src/uar/runtime/presentation_history_tests.rs.
3. Add scripts/verify-rust-test-baseline.sh as a serial compile-only safeguard for minimal, server-full, server-full,postgres-backend; integrate into existing lefthook pre-commit Rust/manifest selector. Provide exact committed/clean-tree mode for attaching acceptance evidence. Do not alter release signing/promotion tooling.
4. Document local invocation and distinguish compilation from runtime acceptance. Prepare existing presentation-history and embeddings/OpenAPI commands; no new mirrored tests.
5. Record implementation-complete inventory, then execute only planned phase-end checks. Fix failures and rerun affected commands. Retain exact source/config/toolchain, initial failures and residual blockers.

Exit contract: all three library harness profiles compile on the exact committed candidate without temporary source edits; existing presentation-history replay/provenance tests and 11 embeddings tests plus OpenAPI test pass; applicable Rust format and full server-full test inventory is recorded truthfully (unrelated failures separately); safeguard refuses dirty evidence mode and propagates compile failure. No skipped/failed/blocked command earns a pass.

Planned commands after implementation boundary, serially: bash scripts/verify-rust-test-baseline.sh --acceptance; cargo test --locked --no-default-features --features server-full --lib presentation_history_tests; cargo test --locked --no-default-features --features server-full --lib uar::api::openai::embeddings::tests; cargo test --locked --no-default-features --features server-full --lib uar::api::openapi::tests; cargo fmt --all -- --check; cargo test --locked --no-default-features --features server-full. Hook and safeguard refusal controls use a synthetic local Git fixture and fake Cargo, after the boundary only. No CI runs.

Authority: October 31 operator request supersedes prior whole-program/per-command verification lock for these bounded useful phases: no tests during investigation/implementation; planned acceptance only at phase exit; affected repair/recheck allowed. External effects retain explicit authorization boundaries. Source inspection/setup/documentation are distinct from verification.

## Dependency order after baseline
- Capability/dispatch slice #463+#464, preserving #388; continuous #466 claim updates.
- Applicable P1 state/adapter slices #468+#469, then #470+#471, reusing current public-path scripts and deterministic peers.
- #467 for any promoted WASM AOT lane; otherwise explicit exclusion. #473–475 only for selected telemetry/memory/team promises. #472 measured claims only with bounded approved workload.
- #465 final actual-artifact qualification, consuming #421 transferred obligations and #466 guarantee table. Freeze per-phase scenario/exit contracts before implementation; do not lump backlog into this phase.

Each subsequent phase gets its own KBD plan, scenario matrix, outcome/non-goals and evidence inventory before source edits. New unrelated discoveries go to triage.

## Boundary inventory — 2026-10-31

All production implementation/documentation items 1–4 are implemented. Item 5 contains the inventory and subsequent acceptance, not additional implementation. Handoffs: .agent-team/uar-core/production-readiness-baseline-{research,implementation}.md and production-readiness-contract-research.md. Exact file digests are in implementation-inventory.json. No runtime behavior, dependency pin, workflow or release publication changed. Tests/checks have not run. The source commit is pending the existing pre-commit gates; acceptance mode must wait for that commit and a clean tree.

Known environment prerequisite: another Cargo test process in the original checkout uses host-global /Volumes/my-passport/cargo/build-dir. Do not kill it or start a second Cargo writer. Synthetic controls and artifact-only review can run independently at this completed implementation boundary. The new phase uses the existing file-based KBD skill protocol; kbd and kbd-runtime executables are absent from PATH. The previous runtime projection is preserved, not relabeled complete.

Policy drift for #466/#465: versions.toml pins Liter 31c7e632 but committed gitlink is 0d74c566 (prepared-streaming-body repair). Preserve source; operator decision requested, no pin edit.

## Operator decision and bounded amendment — 2026-10-31

The operator answered “Retain current Liter source” to the explicit choice between retaining the committed 0d74c566 source and updating policy, or restoring the earlier policy-pinned source. This authorizes aligning versions.toml's liter_llm_commit to 0d74c56659943e3fa44a3c8e6d51a27514f3b35c, retaining version2.3.0 and all dependency source bytes. Owner: lead. This is #466/#465 provenance alignment, not a dependency upgrade or acceptance certificate.

Added implementation item: update the one policy commit and its dated rationale; append the decision and supersede the pending decision in current KBD state. Renew the implementation inventory after writing. Source inspection/review must confirm policy equals both HEAD's gitlink and the materialized submodule commit. No compiler check is needed to establish this policy-only equality. The planned real #462 compile/regression gate remains pending shared-writer availability; prior synthetic evidence stays historical to the script, with no new product acceptance claim.

## Observed baseline repair cohort — full inventory attempt001

Exact committed candidate a3bceb9a412d86f23c4ad30dee450872b34e36bd passed the clean three-profile library compilation safeguard,4presentation-history,11embeddings and1OpenAPI tests. The planned full `cargo test --locked --no-default-features --features server-full` inventory then failed compilation (exit101), before test execution. Config also sets RUN_LLM_TESTS=0 to avoid paid opt-in integration tests. No passing whole-suite claim follows.

Add one bounded fixture-compatibility repair cohort to this same phase, not a new phase per edit. No runtime implementation or assertion changes. Observed items:
- `tests/typed_turn_assembly.rs`, `tests/jwks_es256_verification.rs`, `tests/turn_shadow_parity.rs`, `tests/test_mcp_optional.rs`, `tests/context_history_integrity.rs`, `tests/bauar_full_harness_cursor.rs`, `tests/support/uar_1_1_1_runtime_fixture.rs`: adapt only stale VectorMatcher constructors to the current SwappableEmbeddingBackend contract, retaining the same underlying backend.
- `tests/common/ollama.rs`: supply the current ModelConfig.execution_profile default explicitly.
- `tests/lifecycle_owned_resources.rs`: convert the unit-valued oneshot send error into the existing anyhow error channel without dropping the failure.
- Warning cohort: unused imports in tests/support/sidecar_process.rs; shared harness run_scoped and shared stub DelayedContent/with_prompt_prefix are intentionally unused in specific including targets. Remove genuinely unused imports or use precise platform cfg; use reasoned #[expect] only at scopes where the observed unused shared fixture code is intentional. No blanket allow or lowered lint.

Ownership: fixture constructor/error/model files go to baseline_compile_repair; unused imports and target-scoped shared-helper warning annotations go to baseline_warning_repair. Lead owns KBD/OpenSpec/history/receipts. Writers must preserve each other's edits. Production source remains unchanged.

Verification gate CLOSED during this cohort. No tests, lint, format, Cargo/compiler probes, builds or review-driven verification until all cohort edits and handoffs are complete and a renewed implementation inventory is recorded. Then isolated Rust review and the failed inventory gate run serially; source-relevant core baseline/regressions must bind the new committed candidate. Preserve initial failures and do not claim broad-suite success when unrelated runtime assertions or external fixture cells remain unqualified.

## Final fixture/API compatibility cohort and inventory-scope correction

Candidate9ed71e93cb2b3e130ff2c042a04b6ed3bcbd51ca passed clean3-profile compilation and the same16specified regressions. Full inventory attempt002 again failed compilation before any test body ran, reaching later targets: skill_activation_runtime, prompt_assembly, uar_integration, kb_embedding_space and mcp_projection still pass bare backends to current swappable matcher/ingester APIs; mcp_projection omits the new optional request metadata argument at two call sites. approval_recovery_process references the development-only InMemoryProvider unconditionally while the shipping profile intentionally excludes it. Raw diagnostics preserved. Prior warning repairs no longer emitted their warnings in this attempt.

The unfiltered full-test command is corrected: it discovers legacy opt-in installed/approval qualifiers whose external contracts are not this phase's work, including the old Dot07 qualifier requiring explicit backend features and installed-client inputs. The tracker explicitly forbids reviving cancelled historical/all-product gates. Both broad attempts failed before execution, so no legacy qualifier was revived. Do not make such execution mandatory for #462 or treat missing authentication as an operator blocker. This is a correction of the phase plan, not a passing-suite claim.

Add one final bounded source compatibility cohort: current swappable constructors in the five named files, optional MCP RequestMetaObject default at the two named calls, and explicit development-only fixture applicability in tests/support/approval_recovery_process.rs. The same backend/arguments/assertions/effect semantics must remain. Do not expose InMemoryProvider in shipping production, add dependency features, drop assertions, mark tests ignored, or fabricate a passing memory-control receipt without the feature. The legacy fixture must fail its explicit required-feature precondition before side effects if directly invoked without the required development feature; retain its original memory assertions under that feature.

Corrected phase-exit inventory after all cohort work completes: ordinary hook and clean committed3-profile baseline; required4history+11embedding+1OpenAPI assertions; `cargo test --locked --no-default-features --features server-full --no-run` compiles all test targets without running opt-in historical qualifiers; `cargo test --locked --no-default-features --features server-full --lib` executes the full library inventory safely without those external/manual target bodies. Existing intentional ignored cases, certification stubs, paid opt-ins and unqualified installed lanes remain explicitly not exercised. No assertion/target is silently removed. Actual installed recovery and protocol qualification stays in #465 using the selected contract and transferred obligations.

Gate CLOSED during this complete cohort; source inspection only. Lead collects both handoffs and renews exact implementation inventory before any review/check/build/test. Source and clean committed candidate receipts must reflect these persistent repairs, never restored-after-test patches.

Source inspection of the same observed matcher/ingester API mismatch found eight additional stale call sites not reached before Cargo stopped scheduling: tests/test_chat_completion.rs, tests/bauar_receipt_projection.rs, tests/mcp_preapproved_tools.rs, tests/test_provider_resolution.rs, tests/truthful_tool_outcomes.rs, tests/model_path_resiliency.rs, tests/durable_ingestion.rs and tests/kb_chunk_strategy_ingest.rs. These join the same final compatibility cohort, owned by baseline_compile_repair; no new behavior or test assertions. Already-swappable tool_call_protocol, ingestion_admission and embedding_lifecycle_failover call sites remain untouched.

The development-memory assertion block in the legacy approval fixture will retain its original backend/assertions under the explicit existing feature, with an early qualify precondition before effects. The test target itself remains available and is not ignored. Add the narrow compilation proof `cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test approval_recovery --no-run` to verify that retained feature-specific block without executing the old qualifier or promoting development memory to a shipping lane. This is an affected fixture compile, not a runtime recovery certificate.
