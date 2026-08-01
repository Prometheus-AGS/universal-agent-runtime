# 13. Embedded run-policy resolution + embedded admin surface

Date: 2026-07-24

## Status

Accepted

## Context

The embedded, in-process runtime (`EmbeddedRuntime`, linked by host apps that
have no HTTP service — the mobile and in-process desktop shells) resolved run
policy through `RunManager::resolve_legacy_run_policy`, which built a
`PolicyResolutionInput` from only the agent artifact and the per-conversation
policy. It never read the global `run_policy.global` setting, so the **Global
default model/policy tier was silently ignored on every embedded host**, even
though the full precedence resolver (`resolve_run_policy`, Global → Agent →
Conversation → Turn) and a `SettingsManager` already existed and were used by the
HTTP service path (`api/discovery.rs::resolve_effective_run_policy`).

Separately, the embedded SDK `Runtime` exposed no settings/agent administration
surface (only `provider_registry()` / `run_manager()`), so an embedding host
could not read or write `run_policy.global` or manage agent definitions without
standing up the HTTP service — defeating the point of embedding. First-party
hosts consequently could not offer the same three-tier model selection (global →
per-agent → per-conversation) that the service path already supports.

The capability belongs in the runtime library so every embedding app inherits it,
rather than each app re-implementing policy storage and resolution against its
own seam.

## Decision

- **Extract a transport-free resolution core.** `resolve_effective_run_policy_core`
  (`src/uar/domain/policy.rs`) takes a `PolicyResolutionContext { settings_manager,
  universe, default_context_strategy }` plus the agent, the caller-resolved
  conversation scope, and the turn — instead of `&AppState`. It reads the Global
  scope from `run_policy.global` via the supplied `SettingsManager` and delegates
  to `resolve_run_policy`, so identical inputs always yield an identical
  `EffectiveRunPolicy` regardless of caller. The service path
  (`resolve_effective_run_policy(&AppState, …)`) becomes a thin wrapper that builds
  the context from `AppState` — behavior unchanged.
- **Wire the embedded RunManager to the Global scope.** `RunManager` builds a
  `SettingsManager` from the persistence it already receives and resolves policy
  via the shared core (`resolve_effective_policy` +
  `build_universe_and_conversation`), so `run_policy.global` is honored on embedded
  hosts exactly as on the service path. `resolve_legacy_run_policy` remains the
  fallback when no settings manager is available, preserving prior behavior for
  callers that do not opt in.
- **Expose an embedded admin surface on the SDK `Runtime`** (behind
  `#[cfg(feature = "embedded")]`): `get_setting`/`set_setting`/`settings_snapshot`
  delegate to the runtime's `SettingsManager`; `list_agents`/`get_agent`/
  `upsert_agent`/`delete_agent` delegate to a new transport-free
  `src/uar/domain/agent_store.rs` used by both the SDK and (where applicable) the
  service handlers. `SettingsManager::ensure_run_policy_seed()` registers the
  `run_policy.global` type/default without an `AppConfig`, so embedded hosts have
  the row to read/write.

## Deviation from the original task note

The task's ground-truth note assumed `load_conversation_policy` and
`policy_universe` read only `settings_manager`. In fact they read **five**
`AppState` services (skill_service, mcp, native_skill_registry, persistence,
settings_manager), and the service path also consults an in-memory
`agent_sessions` fallback for the conversation scope. Rather than force those into
the core (which would either widen the context struct with service-only
dependencies or drop the in-memory fallback), they stay **service-side**: the core
accepts an already-built `PolicyUniverse` and a caller-resolved conversation
scope. This preserves the service's exact behavior (Base Rule 32) while still
giving one shared precedence core for both paths.

## Consequences

- Embedded hosts (mobile, in-process desktop) gain a working global-default tier
  and in-process settings + agent administration — three-tier model selection and
  admin UIs with no HTTP service.
- No change for service-mode consumers; provider/model routing is unchanged (only
  which *scope* supplies the `ModelRoute` changes, not how a model is called).
- The per-run `effective_run_policy` artifact now reflects the resolved Global
  scope on embedded hosts too, keeping provenance/telemetry consistent across
  surfaces.
- Test-only: the SDK enables the runtime's `in-memory-backend` under
  `[dev-dependencies]` so embedded admin/policy tests build a Runtime without a
  real database; the shipped `embedded` feature set is unchanged.
