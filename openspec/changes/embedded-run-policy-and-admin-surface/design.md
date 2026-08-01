# Design — Embedded run-policy resolution + embedded admin surface

## Context

`resolve_effective_run_policy` (`src/uar/api/discovery.rs`) already resolves the
full Global → Agent → Conversation → Turn precedence, reading the Global scope
from `state.settings_manager.get_typed::<RunPolicy>("run_policy.global")`. Its
signature is `(&AppState, conversation_id, &AgentArtifact, turn)` but it uses
**only** `state.settings_manager` (`Option<SettingsManager>`), `state.persistence`
(via `load_conversation_policy` / `policy_universe`), and
`state.config.context_strategy` — **no axum extractors**. So it is nearly
transport-free already.

The embedded runtime's `RunManager` resolves policy through
`resolve_legacy_run_policy` (`src/uar/runtime/manager.rs:685`, called ~:862),
which builds `PolicyResolutionInput` from only the agent artifact + conversation
and never reads `run_policy.global`. `RunManager::new` already receives
`persistence: Option<Arc<dyn PersistenceLayer>>` (manager.rs:~411,417), and
`SettingsManager::new(persistence)` takes exactly that. So the embedded path can
build a `SettingsManager` from the persistence it already has and reuse the same
resolution logic — no new subsystem.

The embedded SDK `Runtime` (`sdks/rust/src/runtime.rs`) exposes `provider_registry()`,
`run_manager()`, `start_agent_run()` — but no settings/agent admin. Host apps that
embed UAR therefore cannot read/write `run_policy.global` or manage agents without
standing up the HTTP service.

## Goals / Non-goals

- **Goal:** the embedded runtime honors `run_policy.global` (Global scope) during
  resolution, matching the service path exactly.
- **Goal:** the SDK `Runtime` exposes settings get/set + agent CRUD, backed by the
  runtime's own `SettingsManager` + persistence.
- **Non-goal:** any change to the HTTP/service resolution path's behavior or public
  contract. The service path is refactored to call a shared core but must produce
  identical results.
- **Non-goal:** a new persistence schema. Reuse the existing settings + agent stores.

## Decision 1 — Extract a transport-free policy-resolution core

Introduce a pure resolver that takes its inputs directly instead of `&AppState`:

```rust
// src/uar/domain/policy_resolution.rs  (new; or a free fn in policy.rs)
pub struct PolicyResolutionContext<'a> {
    pub settings_manager: Option<&'a SettingsManager>,
    pub persistence: Option<&'a Arc<dyn PersistenceLayer>>,
    pub default_context_strategy: ContextStrategy,
}

pub async fn resolve_effective_run_policy_core(
    ctx: PolicyResolutionContext<'_>,
    conversation_id: &str,
    agent: &AgentArtifact,
    turn: Option<RunPolicy>,
) -> EffectiveRunPolicy
```

Its body is the current `resolve_effective_run_policy` body verbatim, with
`state.settings_manager` → `ctx.settings_manager`, `state.persistence` →
`ctx.persistence`, `state.config.context_strategy` → `ctx.default_context_strategy`.
`load_conversation_policy` and `policy_universe` are made to take
`&Arc<dyn PersistenceLayer>` / the two option refs instead of `&AppState` (both
already use only `state.persistence`/`state.settings_manager` internally, so this
is mechanical).

- **Service path:** `resolve_effective_run_policy(&AppState, …)` becomes a thin
  wrapper that builds `PolicyResolutionContext` from `state` and calls the core.
  Behavior is identical (Base Rule 32 — preserve existing behavior).
- **Embedded path:** `RunManager` gains an `Option<SettingsManager>` built from its
  `persistence` at construction (or lazily). Its resolution site
  (`resolve_legacy_run_policy` caller, manager.rs:~862) calls
  `resolve_effective_run_policy_core` with the manager's settings + persistence.
  `resolve_legacy_run_policy` is retained as the fallback when no settings manager
  is available, so no behavior change for any caller that doesn't opt in.

Precedence, scope order, and the emitted `effective_run_policy` artifact are
unchanged — the embedded path simply now includes the Global scope.

## Decision 2 — Embedded admin surface on the SDK `Runtime`

Expose the runtime's existing `SettingsManager` + persistence-backed agent store
through typed SDK methods. `EmbeddedRuntime` gains accessors (e.g.
`settings_manager()`, and it already has `persistence()`); the SDK `Runtime`
wrapper adds:

```rust
// sdks/rust/src/runtime.rs
impl Runtime {
    // settings
    pub async fn get_setting(&self, key: &str) -> anyhow::Result<Option<serde_json::Value>>;
    pub async fn set_setting(&self, key: &str, value: serde_json::Value) -> anyhow::Result<()>;
    pub async fn settings_snapshot(&self) -> anyhow::Result<SettingsSnapshot>; // settings + setting_types

    // agents (delegating to the same store the service path uses)
    pub async fn list_agents(&self) -> anyhow::Result<Vec<AgentArtifact>>;
    pub async fn get_agent(&self, id: &str) -> anyhow::Result<Option<AgentArtifact>>;
    pub async fn upsert_agent(&self, agent: AgentArtifact) -> anyhow::Result<()>;
    pub async fn delete_agent(&self, id: &str) -> anyhow::Result<()>;
}
```

- Settings map to `SettingsManager::{get_typed/get, set_value}` (get returns the
  raw JSON value so a host can read/write `run_policy.global`).
- Agent CRUD delegates to the persistence-backed agent store that the service
  handlers (`discovery.rs` create/update/patch/delete_agent, ~:281-383) already
  use. If that logic is axum-bound, extract a transport-free helper (like Decision 1)
  and have both the service handlers and the SDK methods call it — the service
  behavior stays identical.

## Alternatives considered

- **Keep `resolve_legacy_run_policy` on embedded, add global-only patch.** Rejected:
  duplicates precedence logic; drifts from the service path. Reusing the one resolver
  is DRY and guarantees parity.
- **Store `run_policy.global` host-side (in KnowMe).** Rejected by the operator: the
  capability must live in the shared library so every embedding app inherits it.
- **New settings/agent trait abstraction.** Rejected (YAGNI): the concrete
  `SettingsManager` + persistence store already exist; expose them directly.

## Risks & mitigations

- **Service-path regression** from the extraction. Mitigation: the wrapper preserves
  the exact call; add a test asserting service and embedded resolvers produce the
  same `EffectiveRunPolicy` for identical inputs.
- **Double settings init.** Building a second `SettingsManager` in `RunManager` when
  the service `AppState` also has one. Mitigation: on the service path keep using
  `AppState.settings_manager`; the embedded `RunManager` builds its own from the same
  persistence (they read the same store, so results agree). Do not call
  `initialize()` twice destructively — `initialize` is idempotent (registers typed
  settings); confirm and, if needed, guard.

## Impact

Unchanged provider routing and service contract. Embedded hosts gain the Global tier
+ in-process settings/agent admin. Follows the crate's free-function/`SettingsManager`
conventions. ADR `0013` records the decision.
