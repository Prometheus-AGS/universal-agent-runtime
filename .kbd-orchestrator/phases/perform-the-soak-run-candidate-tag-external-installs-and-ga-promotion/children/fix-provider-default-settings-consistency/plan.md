PLAN: fix-provider-default-settings-consistency
Project: universal-agent-runtime
Date: 2026-08-19
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. fix-provider-default-settings-consistency: Accept the supported local memory provider during settings bootstrap and prevent rejected default-provider writes from changing live routing state.
   - Scope: host configuration schema | provider API | focused Rust tests | OpenSpec evidence
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: S
   - Customer value: HIGH
   - Details: Add `local` to the existing memory settings schema because `MemoryConfig`, the memory service, and the certified local profile already support it. Change the default-provider handler to validate the provider, persist the selection, and only then publish it to the in-memory registry; prove the failure and success orderings with focused tests. Do not alter frontend behavior, general startup-failure policy, or the inactive config-authority proposals.

EXECUTION ROUND ORDER
Round 1 (serial): `fix-provider-default-settings-consistency`

IMPLEMENTATION ORDER
1. Create the OpenSpec change and add a delta to `provider-model-settings-certification` for supported settings values and persistence-before-publication semantics.
2. Add `mgr_initialize_accepts_local_memory_embedding_provider` and `mgr_initialize_rejects_unknown_memory_embedding_provider` to `tests/settings_persistence.rs`. Run both focused tests to capture the positive failure and retained rejection control, then run Tier 0 before the next edit.
3. Add `local` to the memory settings schema in `src/uar/settings/manager.rs`. Rerun both focused settings tests and Tier 0 before touching the provider handler.
4. Add three private handler tests in `src/uar/api/providers.rs`: persistence failure preserves the live default, a missing provider preserves durable/live defaults and returns not-found, and success agrees with a fresh `SettingsManager` reopened over the same persistence layer. Run the focused tests to observe the pre-fix ordering failure, then run Tier 0 before the next edit.
5. Change the handler to validate the target, persist the selection, and only then publish it to the registry. Rerun all three provider tests and Tier 0.
6. Run only the child completion checks listed below, produce row-form verification and artifact-refiner evidence, and obtain history-free critic and judge approval while the live OpenSpec change remains available for review.
7. Archive/sync with `openspec archive fix-provider-default-settings-consistency --yes`, then strict-validate the synced `provider-model-settings-certification` capability.
8. Complete child reflection, run `/kbd-child-exit complete`, replace the generated `handoff-out.md` placeholders with exact deliverables and parent resume commands, restore the parent runtime’s `exactNextWork` to `/opsx:apply screen-by-screen-validation`, and create the single final child commit. Full screen recertification remains parent work.

ACCEPTANCE CRITERIA
- `SettingsManager::initialize` accepts `memory.embedding_provider=local` and seeds `llm.default_provider` instead of stopping early.
- An unsupported memory embedding provider still fails schema validation; the repair does not replace the enum with unrestricted strings.
- A missing target provider still returns the existing not-found response and changes neither durable nor live default state.
- A default-provider persistence failure returns HTTP 500 and leaves the prior registry default unchanged.
- A successful selection leaves `ProviderRegistry::default_id` equal to the requested provider, and a fresh `SettingsManager` initialized over the same persistence layer rereads that same provider id.
- `openspec validate fix-provider-default-settings-consistency --strict --no-interactive` passes with a real spec delta.
- Focused tests, Tier 0 check, package-scoped Clippy, formatting/diff checks, and artifact-refiner validation are observed and recorded with commands and output.
- No parent BDD, frontend, general startup policy, dependency, migration, or inactive OpenSpec proposal is modified by the child.
- The OpenSpec delta is reviewed before archive, synced into the canonical capability, and strict-valid after archive.
- Reflection, child exit, completed handoff, implementation, evidence, archive, and canonical KBD projections are included in one final child commit; nothing is pushed and no PR is opened.

VERIFICATION ORDER
1. Focused settings bootstrap positive and invalid-value controls, using the exact selectors below.
2. Focused provider persistence-failure, missing-provider, and fresh-manager durable round-trip controls, using the exact selectors below.
3. `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full`.
4. `cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps`.
5. `cargo fmt --all -- --check` and `git diff --check` scoped to the child candidate.
6. `openspec validate fix-provider-default-settings-consistency --strict --no-interactive`.
7. Artifact-refiner schema/reference/integrity gate and history-free critic/judge review.
8. `openspec archive fix-provider-default-settings-consistency --yes`, followed by strict canonical spec validation.

TIER BOUNDARY
- Every source/test edit is followed by its applicable focused observation and the Tier 0 command before another edit begins. The plan deliberately separates test introduction from the corresponding implementation edit.
- It does not rerun the 32-scenario browser certification, the full server test suite, release builds, or publication checks. Those remain at the outer phase’s existing completion points.

PERMITTED WRITE SURFACE
- `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-provider-default-settings-consistency/**`
- `src/uar/settings/manager.rs`
- `src/uar/api/providers.rs`
- `tests/settings_persistence.rs`
- `openspec/changes/fix-provider-default-settings-consistency/**`
- `openspec/changes/archive/2026-08-19-fix-provider-default-settings-consistency/**`
- `openspec/specs/provider-model-settings-certification/spec.md`
- `.refiner/artifacts/fix-provider-default-settings-consistency/**`
- `.refiner/history/fix-provider-default-settings-consistency/**`
- `.refiner/registry.json` as generated local state only; exclude it from the commit
- `.prometheus/**` append-only history

STOP CONDITIONS
- The fix requires a new crate dependency, schema migration, or a file outside the permitted surface.
- `local` is not actually supported by the current memory runtime under `server-full`.
- Persist-first ordering changes the missing-provider HTTP contract or cannot be tested without widening public API.
- The focused test exposes a broader partial-bootstrap policy decision that must change `src/server.rs`.
- An inactive overlapping OpenSpec change already implemented the same source behavior or requires a conflicting invariant.
- Any parent-owned BDD/evidence file must be altered to make the child pass.

TRADE-OFFS AND SCOPE CUTS
- This fixes the observed provider mutation ordering; it does not claim a distributed transaction across persistence and registry under arbitrary concurrent provider deletion.
- General behavior after any unrelated settings bootstrap failure remains unchanged. Redesigning startup to fail, disable all settings mutations, or discard the manager is a separate architecture decision.
- The broader `make-config-authoritative-on-boot` decision and frontend optimistic rollback changes remain deferred and untouched.
- The uncomfortable scenario is retained as the negative control: durable write failure must not silently change the provider used for live routing.

COMMANDS TO RUN
- `openspec new change "fix-provider-default-settings-consistency"`
- `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test settings_persistence mgr_initialize_accepts_local_memory_embedding_provider -- --exact --nocapture`
- `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test settings_persistence mgr_initialize_rejects_unknown_memory_embedding_provider -- --exact --nocapture`
- `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::api::providers::tests::set_default_persistence_failure_preserves_live_default -- --exact --nocapture`
- `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::api::providers::tests::set_default_missing_provider_preserves_defaults -- --exact --nocapture`
- `cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::api::providers::tests::set_default_success_survives_fresh_manager -- --exact --nocapture`
- After every edit: `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full`
- Completion Clippy: `cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps`
- Completion formatting: `cargo fmt --all -- --check`
- Change validation: `openspec validate fix-provider-default-settings-consistency --strict --no-interactive`
- Archive/sync: `openspec archive fix-provider-default-settings-consistency --yes`
- Synced spec validation: `openspec validate provider-model-settings-certification --type spec --strict --no-interactive`
- Child exit: `/kbd-child-exit complete`
- Canonical parent next work: `prometheus kbd --path . phase activate --command-id 'phase-resume:screen-by-screen-validation:after-provider-settings-child' --id 'perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion' --exact-next-work '/opsx:apply screen-by-screen-validation'`
- Parent lifecycle resume: `/opsx:apply screen-by-screen-validation`
- First parent focused check: `CI=1 pnpm exec playwright test -c tests/bdd/playwright.config.ts tests/bdd/.features-gen/features/product-screen-validation.feature.spec.js --grep 'Providers changes|Auth mints|MCP health'`

SYCOPHANCY REVIEW
- The optional `sycophancy-correction` tool is unavailable. Manual review cut frontend work, generalized bootstrap redesign, dependencies, migrations, and overlapping inactive proposals; it retained the cross-store transaction limit explicitly.

PLAN COMPLETE
