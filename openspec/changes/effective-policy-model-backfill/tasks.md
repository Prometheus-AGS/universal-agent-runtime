# Tasks

## 1. Backfill the resolved model route

- [x] 1.1 In `start_run_with_policy`, after resolving `effective_policy`,
      backfill `effective_policy.model` from `resolve_default_model()` when the
      route is `None` or has an empty `provider_id`/`model_id`.
- [x] 1.2 Leave a fully-specified route untouched (precedence preserved).

## 2. Verify

- [x] 2.1 Add an embedded test asserting the `effective_run_policy` artifact
      carries the registry-default `(provider_id, model_id)` when the agent's
      provider default is empty
      (`effective_run_policy_artifact_backfills_the_registry_default_model`).
- [x] 2.2 `cargo test --no-default-features --features server-full,in-memory-backend
      --lib` for the new test — green.
- [x] 2.3 `cargo fmt --check` — clean.
- [x] 2.4 Device re-verify on iPhone (release build, detached): the embedded-UAR
      Orchestrator bubble reports the on-device provider/model.
