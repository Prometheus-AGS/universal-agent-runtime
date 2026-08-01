# Embedded conversation-policy admin surface + effective-config resolver

## Why

The embedded runtime already honors the Conversation scope at resolution time —
`RunManager` reads `load_conversation_policy` when building the resolution
universe — and the persistence layer already exposes
`save/load/delete_conversation_policy`. But the SDK `Runtime` gave embedding
hosts no way to write or read a conversation policy, or to compute a
conversation's effective configuration, without standing up the HTTP service.

So first-party embedded hosts could offer the Global and per-Agent model tiers
(from the embedded run-policy + admin surface change) but not the
**per-conversation** tier — the third scope of the Cherry-Studio-style model
selection. KnowMe's `control_plane` conversation-policy and effective-config
operations returned "not available on the embedded runtime yet".

## What Changes

- Add SDK `Runtime` methods (feature `embedded`), mirroring the settings/agent
  admin surface: `save_conversation_policy`, `get_conversation_policy`,
  `delete_conversation_policy` — thin delegations to the persistence layer the
  runtime already owns.
- Add `Runtime::effective_config(conversation_id)` returning an `EffectiveConfig`
  (resolved agent + stored requested policy + effective policy), backed by a new
  `RunManager::effective_config` that resolves the agent named by the stored
  conversation policy (or the default agent), resolves the effective run policy
  through the shared transport-free core, and applies the model backfill.
- Factor the effective-config computation and the ADR-0014 model backfill into
  `RunManager` methods (`effective_config`, `resolve_agent_or_default`,
  `backfill_effective_model`) so `start_run_with_policy` and the new resolver
  share one implementation.

No new persistence methods: the surface reuses the existing
`save/load/delete_conversation_policy`.

## Impact

- Affected specs: `embedded-admin-surface` (extended)
- Affected code: `sdks/rust/src/runtime.rs`, `src/uar/runtime/manager.rs`
- Behavior: additive. Enables per-conversation model overrides on embedded hosts,
  completing global → agent → conversation model selection at parity with the
  service path. Precedence unchanged.
