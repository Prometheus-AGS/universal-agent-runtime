# Tasks

## 1. Seed-history seam

- [x] 1.1 Add `SeedMessage { role, content, tool_call_id }`.
- [x] 1.2 `RunManager::start_run_with_policy_and_history` seeds an empty session
      (`message_count() == 0`) from `seed_history` before adding the current
      user message; `start_run_with_history` and the SDK
      `Runtime::start_run_with_history` delegate.
- [x] 1.3 Existing `start_run`/`start_run_with_policy` delegate with an empty
      seed (no behavior change).

## 2. Verify

- [x] 2.1 `cold_started_session_is_seeded_from_supplied_history`: empty-session
      run sees seeded turns + current input; warm session is not re-seeded.
- [x] 2.2 `cargo check --features server-full` + full embedded lib suite (9
      passed) green; `cargo fmt --check` clean.
