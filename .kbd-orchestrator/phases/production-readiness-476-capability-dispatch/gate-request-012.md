# Proposed gate012 — not executed

## Inventory and scope

Gate011 stopped at its first source review with E8; its second review and all B/C commands were NOT RUN. This new packet supersedes its execution permission. All gates are currently CLOSED; approval of this exact packet is required.

Implementation inventory012 records the integrated rc.2 working tree and E6/E7 corrections and E8 output-cap ordering repair. The standard merge has MERGE_HEAD af37c2f834f7b34f511ecc93f408e80205c94229 and current first parent33e765bb5d29a44e97d2813ad6d0be8bf3e2479d. The merge is resolved but deliberately uncommitted: normal commit hooks invoke closed gates. Own-work premerge snapshot a3b5b6f8e6b6222cfaa9ad419c4ca8e5df083bd7 is retained, not dropped. Other chat checkout, branch refs, config and services were not modified. No tests, checks, builds or review gates ran during this repair/integration.

All commands below are PROPOSED ONLY and run serially in `/Users/gqadonis/.claude/worktrees/production-readiness-476`. No gate starts without operator approval. Any implementation change closes the gate and requires renewed inventory/instruction. Stop on a failed gate; do not expand into unrelated suites. Logs/evidence will retain exact command, source/configuration and output. Source-only review is not runtime acceptance; filtered assertions below are supplemental, not substitutes for installed-container HTTP evidence.

## Gate A — bounded source-only corrective/integration review

Use the existing `context_artifact_critic` for one bounded source-only E8 corrective pass and `context_rust_source_fallback` for the pending final Rust-focused source assessment, serially. Provide fixed final artifact `target/production-readiness-476/repair-012-review/artifact.patch`, adjacent requirements, previous fixed integration artifact where needed and necessary baseline/pinned source only. No producer rationale, handoffs, memory or preferred verdict. First review scope: E8 output ordering and actual fallback orchestration assertions, preserving exact-contract/unknown-window branches, immutable scope, local-refusal health behavior and fresh final admission. The second assessment covers the final E6/E7/E8 result and relevant merge boundary already selected in gate011. No reopening old whole-phase finding hunts. Record same-family/non-specialist limitations. Source inspection and own report writes only; no compiler/test/build/API/container/machine screening or git diff --check. A concrete source defect violating selected exit behavior is blocking regardless of generic severity verdict. Stop on a blocking finding before the next review or gate. Any source edit closes the gate and requires new inventory/instruction.

Proposed reports: `.agent-team/uar-core/reviews/production-readiness-context-repair/code-corrective-012.md` and `rust-source-corrective-012.md`.

## Gate B — ordinary local merge commit and clean #462 baseline

Only after Gate A has no unresolved blocking source defects and no implementation changes have invalidated inventory012. Proposed shell setup, applicable to every command in this packet:

```bash
export PATH="/usr/local/bin:$PATH"
export CARGO_BUILD_BUILD_DIR=/Users/gqadonis/.claude/worktrees/production-readiness-476/target/build
export CARGO_CACHE_RUSTC_INFO=0
export RUSTC_WRAPPER=
export CARGO_TERM_COLOR=never
```

Explicit ownership/source preflights, proposed as part of this approval (not implicit extras):

```bash
lsof target/debug/.cargo-artifact-lock target/debug/.cargo-lock target/build/debug/.cargo-build-lock
git status --short --untracked-files=all
git submodule status --recursive
```

For lsof, exit1/no rows means no observed holder, not a lease. If a holder exists, wait without killing it or changing target/cache. Stage only this worktree's reviewed inventory; the recorded target/ directory is ignored and private runtime config remains there. Proposed commands:

```bash
git add -A
git commit -m "fix(runtime): integrate rc2 and preserve destination budgets"
bash scripts/verify-rust-test-baseline.sh --acceptance
```

The ordinary commit intentionally retains all checked-in hooks. `pre-commit` executes `pnpm github-actions-policy:validate` and `bash scripts/verify-rust-test-baseline.sh` for Rust/manifests, and commit-msg executes `pnpm exec commitlint --edit <Git-supplied-message-file>`. No hook bypass, lowered lint, or global config edit. The explicit clean guard compiles minimal, server-full and server-full,postgres-backend library harnesses with --locked/--no-default-features/--no-run and records unchanged source/submodule/toolchain identity. Commit approval therefore necessarily authorizes these named hook commands; no frontend files changed and frontend hooks do not select this tree.

## Gate C — affected integration, required regressions and metadata

E8 coverage is the authored six-case actual-orchestrator regression listed explicitly below. It uses real registry/admission with synthetic captured streams and remains supplemental; no installed-provider behavior is certified.

Serial commands under the same environment, no broad all-target/full library run:

```bash
cargo test --locked --no-default-features --features server-full --test context_history_integrity
cargo test --locked --no-default-features --features server-full --lib llm::orchestrator::tests::fallback_stream_reserves_destination_output_before_fitting_history
cargo test --locked --no-default-features --features server-full --lib presentation_history_tests
cargo test --locked --no-default-features --features server-full --lib uar::api::openai::embeddings::tests
cargo test --locked --no-default-features --features server-full --lib spec_uses_package_version_and_documents_customer_routes
cargo test --locked --no-default-features --features server-full --lib settings_only_profile_refuses_liter_rewritten_wire_alias
cargo test --locked --no-default-features --features server-full --lib settings_only_profile_preserves_prepared_alias_controls_and_schema
cargo test --locked --no-default-features --features server-full --lib prepared_final_wire_budget_uses_liter_bytes_and_refuses_image_liability
cargo test --locked --no-default-features --features server-full --lib durable_contract_without_final_wire_evidence_refuses_before_inner_stream
cargo test --locked --no-default-features --features server-full --lib remote_admission_refusal_does_not_lease_root_budget
cargo test --locked --no-default-features --features server-full --lib remote_persisted_launch_refusal_releases_capacity_and_shutdown_settles_record
cargo test --locked --no-default-features --features server-full --lib remote_pending_cancellation_releases_capacity_before_any_dispatch
cargo test --locked --no-default-features --features server-full --lib compare_candidates_tests::disabled_runtime_catalog_model_is_not_reentered_or_preferred
cargo test --locked --no-default-features --features server-full --lib compare_candidates_tests::registered_context_limit_is_not_reentered_from_catalog_or_preference
python3 .kbd-orchestrator/phases/production-readiness-476-capability-dispatch/evidence/verify-controlled-peer-observation.py
node .kbd-orchestrator/phases/production-readiness-476-capability-dispatch/evidence/verify-fixture-lifetime.mjs
pnpm sdk-runtime-pin:validate
pnpm support-matrix:validate
openspec validate carry-inline-images-and-constrain-dispatch
```

The context integration target retains its two historical frozen diagnostics as ignored; no new skip. Required history/embeddings/OpenAPI assertions historically count4/11/1; observe actual current counts. Each new supplemental name must select its real authored case; zero selected is not a pass. Fixture probes are controlled loopback/component evidence, not product certification; the lifetime probe retains its actual100second bound. support-matrix:validate invokes cargo tree internally and is serialized under the same build owner. The SDK pin check covers imported rc.2 runtime/SDK alignment.

## Still closed and outside this immediate packet

Docker build/recovery preflights, customer artifact build/inspection, candidate service/peer/gateway provisioning, mounted Cucumber scenarios, compiler-owner/migration qualification, imported rc.2 fixture reruns, OpenSpec archive/sync, signing/publication/push/deployment and paid provider calls are NOT authorized by approval of A–C. After these native gates, present exact owned Linux artifact/provisioning/HTTP commands with actual committed source and private-host configuration references. Native compilation/fixtures cannot qualify the Linux artifact, real vision, PG parity, recovery or another platform. Historical rc.2 macOS receipts stay historical.
