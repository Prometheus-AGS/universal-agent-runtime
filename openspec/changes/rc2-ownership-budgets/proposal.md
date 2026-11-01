## Why

Two open issues block 1.4.0-rc.2:

- **#387:** compiler REST handlers took no verified owner, and the stores listed specs, reports and authoring sessions globally. Global authentication does not supply tenant authorization, so on a shared deployment one owner could read, compile or delete another's specs.
- **#388:** ordinary runtime construction left destination preparations unset. A fallback model with a smaller context window could receive a request sized for the primary.

The release also needs its version, changelog and release note for 1.4.0-rc.2.

## What Changes

- **#387:** `CompilerOwner` (`src/uar/compiler/owner.rs`) is captured from the verified request context. Spec, report and session storage on SurrealDB and PostgreSQL filter by it. Migration `20261031010000_compiler_owner_scope.sql` adds a nullable `owner` column and index to `uar_specs`, `uar_reports` and `uar_compiler_sessions`. Another owner's record answers 404; no verified owner answers 401. Legacy ownerless records are readable only by the local-host owner and are never rewritten. SurrealDB `delete_spec` now reports whether it deleted a record instead of always succeeding.
- **#388 (partial):** on failover, each fallback's context window is resolved per run (run credential, provider registry, catalog), and `fit_fallback_request` reduces the request to that window with the existing history reducer before the fallback is prepared. A request that cannot fit is refused locally without counting against provider health; if no fallback fits, the run ends with `context_overflow` before any fallback dispatch. The attempt manifest carries a `context_fit` report. Exact destination preparations are not installed at construction: they fail closed for any model missing from the map, and every model is chosen per run.
- **Concurrent first turns (found by the rc.2 catalog):** `SessionManager::get_or_create_for_user` re-checks and inserts under the write lock, so two concurrent first turns of a new conversation share one session instead of the later save overwriting the earlier turn.
- **Release:** version 1.4.0-rc.2 in `Cargo.toml`, `sdks/rust/Cargo.toml` and both lockfiles; CHANGELOG entry; `docs/releases/v1.4.0-rc.2.md` with the 1.4.0 final checklist.
- **BDD:** `compiler-owner-isolation.feature` (#387, both stores) and `fallback-destination-budget.feature` (#388).

## Impact

- Code: `src/session/thread.rs`, `src/uar/api/compiler.rs`, `src/uar/compiler/**`, `src/uar/runtime/manager.rs`, `src/uar/runtime/turn/bindings.rs`, `src/llm/orchestrator.rs`.
- Fixtures: `dedicated-runtime.mjs` gains `stubFixtures` and async `config`/`env`; `provider-fault-proxy.mjs` gains `arm?mode=unavailable`.
- Not covered by #388 here: exact final-wire budgets, the fallback's own output maximum, per-step primary recompilation, tokenizer change and summary failure.
- Storage: one additive PostgreSQL migration; SurrealDB records gain an optional `owner` field.
- API: compiler spec, report and session endpoints now require a verified owner. `POST /compile`, `POST /verify` and `POST /compile-and-register` are unchanged: the first two store nothing, and the third writes the agent catalog, not compiler records.
