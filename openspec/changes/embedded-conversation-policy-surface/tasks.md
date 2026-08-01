# Tasks

## 1. SDK conversation-policy surface

- [x] 1.1 Add `Runtime::save_conversation_policy(id, RunPolicy)`,
      `get_conversation_policy(id)`, `delete_conversation_policy(id)` delegating
      to the persistence layer (feature `embedded`).
- [x] 1.2 Add `Runtime::effective_config(id) -> EffectiveConfig` delegating to
      `RunManager::effective_config`.

## 2. RunManager resolver

- [x] 2.1 Add `RunManager::effective_config(id)`: resolve agent (stored policy's
      agent_id or default) → resolve effective policy → backfill model → return
      `EffectiveConfig { agent, requested_policy, effective_policy }`.
- [x] 2.2 Factor the ADR-0014 model backfill into
      `RunManager::backfill_effective_model` and call it from both
      `start_run_with_policy` and `effective_config`.

## 3. Verify

- [x] 3.1 Embedded test `conversation_policy_round_trips_and_effective_config_
      reflects_the_override`: no policy → registry-default model; save openai/gpt-4o
      override → effective model = openai/gpt-4o; delete → reverts to default.
- [x] 3.2 `cargo check --features server-full` (lib) + `cargo check -p
      universal-agent-runtime-sdk` (embedded-mobile) — green.
- [x] 3.3 `cargo fmt --check` — clean.
