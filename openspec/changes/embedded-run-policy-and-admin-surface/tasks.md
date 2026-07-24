# Tasks — Embedded run-policy resolution + embedded admin surface

## 1. Transport-free policy-resolution core

- [x] 1.1 Add `PolicyResolutionContext<'a>` and `resolve_effective_run_policy_core(ctx, conversation_id, agent, turn) -> EffectiveRunPolicy` (new `src/uar/domain/policy_resolution.rs` or a free fn near `policy.rs`), moving the current body of `discovery.rs::resolve_effective_run_policy` and swapping `state.settings_manager`/`state.persistence`/`state.config.context_strategy` for context fields.
- [x] 1.2 Change `load_conversation_policy` and `policy_universe` to take `Option<&Arc<dyn PersistenceLayer>>` / `Option<&SettingsManager>` instead of `&AppState` (mechanical; they already use only those). Keep thin `&AppState` shims if other callers need them.
- [x] 1.3 Rewrite `discovery.rs::resolve_effective_run_policy(&AppState, …)` as a thin wrapper that builds `PolicyResolutionContext` from `state` and calls the core. Verify service behavior is byte-identical.

## 2. Embedded runtime honors the global scope

- [x] 2.1 Give `RunManager` an `Option<SettingsManager>` built from its `persistence` (`SettingsManager::new(persistence)`), constructed in `RunManager::new` (or lazily) when persistence is present.
- [x] 2.2 At the embedded resolution site (the `resolve_legacy_run_policy` caller, manager.rs ~:862), call `resolve_effective_run_policy_core` with the manager's settings + persistence. Keep `resolve_legacy_run_policy` as the fallback when no settings manager is available.
- [x] 2.3 Confirm `SettingsManager::initialize` is idempotent (or guard it) so building a second manager on the embedded path does not corrupt the shared store.

## 3. Embedded admin surface on the SDK Runtime

- [x] 3.1 Add `EmbeddedRuntime` accessor(s) needed by the SDK wrapper (`settings_manager()`; `persistence()` already exists).
- [x] 3.2 SDK `Runtime`: add `get_setting`/`set_setting`/`settings_snapshot` delegating to `SettingsManager::{get_typed|get, set_value}` + the registered setting types.
- [x] 3.3 If agent CRUD in `discovery.rs` (create/update/patch/delete_agent ~:281-383) is axum-bound, extract a transport-free agent-store helper; keep the service handlers calling it (behavior unchanged).
- [x] 3.4 SDK `Runtime`: add `list_agents`/`get_agent`/`upsert_agent`/`delete_agent` delegating to that helper.

## 4. Tests

- [x] 4.1 Unit: embedded resolution applies `run_policy.global` model when agent+conversation set none (seed a `SettingsManager` with a global `RunPolicy.model`).
- [x] 4.2 Unit: agent + conversation still override the global default (precedence conv > agent > global).
- [x] 4.3 Unit: no-settings-manager fallback resolves via legacy path without error.
- [x] 4.4 Parity test: service and embedded resolvers return an equal `EffectiveRunPolicy` for identical inputs.
- [x] 4.5 Unit: SDK `set_setting`/`get_setting` round-trip for `run_policy.global`; `settings_snapshot` returns values + types.
- [x] 4.6 Unit: agent CRUD round-trip (upsert→list→get→delete) against in-process persistence.

## 5. Verification, docs, and pointer

- [x] 5.1 `cargo check --locked --no-default-features --features server-full` green.
- [x] 5.2 Default-feature `cargo check` green; `cargo test` for touched modules green.
- [x] 5.3 `cargo fmt --all -- --check` clean; no new clippy warnings (`#[expect(reason=…)]` only if unavoidable).
- [x] 5.4 ADR `docs/adr/0013-embedded-run-policy-and-admin-surface.md` (follow 0012) + index in `docs/adr/index.md`.
- [x] 5.5 `openspec validate embedded-run-policy-and-admin-surface --strict` clean; commit on `desktop-stable-port-opsx`.
- [ ] 5.6 (parent repo) bump the KnowMe submodule pointer to the new UAR commit.
