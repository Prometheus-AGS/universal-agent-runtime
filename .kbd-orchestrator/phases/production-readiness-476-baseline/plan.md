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
