## 1. Hot-path profiling

- [x] 1.1 `benches/hot_path.rs`: `PromptDialect::detect` across 7 model ids
- [x] 1.2 `strategy_for_model` across 5 context-window sizes
- [x] 1.3 `apply_strategy` sliding-window trim over a 500-message synthetic
      conversation
- [x] 1.4 `ModelRouter::route` (async, seeded provider registry, tools-required
      requirements)
- [x] 1.5 `criterion` dev-dependency + `[[bench]]` entry in `Cargo.toml`
      (`harness = false`, Criterion drives its own `main()`)
- [ ] 1.6 Run `cargo bench --bench hot_path` and record baseline numbers
      (not done this session — deliberately out of scope, see proposal.md)

## 2. Concurrent-agent load test

- [x] 2.1 `tests/integration/live/load_test.rs`: 50 concurrent chat-completion
      requests against a real booted server + stub LLM
- [x] 2.2 Reports p50/p95/max latency + throughput; hard-asserts zero
      request failures
- [x] 2.3 Registered in `tests/integration/live/mod.rs`
- [x] 2.4 `MATRIX.md` row added (CH-20)
- [x] 2.5 `cargo check --test integration` clean

## 3. Prompt-injection resistance review

- [x] 3.1 Found evasion: plain substring scan defeated by whitespace
      padding / line breaks / tabs
- [x] 3.2 `normalize_whitespace` helper + wired into `screen_input`
- [x] 3.3 Tests: 3 evasion-closes (whitespace, line-break, tab+mixed) + all
      known phrases with surrounding noise
- [x] 3.4 Tests: 3 disclosed known-gaps (synonym substitution, roleplay
      framing, base64-encoded payload) — NOT fixed, honestly documented as
      false negatives for a future classifier-based approach to close
- [x] 3.5 `normalize_whitespace` unit test (Unicode whitespace kinds)
- [x] 3.6 `cargo test --lib uar::guardrails::` 13/13 green
- [x] 3.7 Fixed incidental bug: test module's `use super::{...}` import
      list was missing 2 new symbols the new tests reference

## 4. `server.rs` split assessment

- [x] 4.1 Current-structure line-range breakdown (5,068 lines total)
- [x] 4.2 Recommended target module layout (`src/server/mod.rs` +
      siblings)
- [x] 4.3 4-step extraction sequence with risk/checkpoint notes per step
- [x] 4.4 Explicit "what this does NOT recommend" section (no
      over-splitting, no premature helper extraction)
- [x] 4.5 Deliberately NOT executing the split (Rule 31 / Rule 8) — this
      is an assessment + recommendation only

## 5. Incidental fixes (separate commit, not folded into this change)

- [x] 5.1 `tests/settings_persistence.rs`: `minimal_config()` was missing
      `AppConfig.guardrails` (added `c454431`, never backfilled) — blocked
      `cargo check --tests` entirely; fixed as its own commit

## 6. Not this change (disclosed, out of scope)

- [ ] `tests/uar_integration.rs`: `Skill` struct literal missing 8 fields
      (pre-existing, unrelated)
- [ ] `tests/bdd.rs`: broken nested `#[path]` resolution (pre-existing,
      unrelated — not part of any tracked G4/G5 change)
- [ ] `cargo check --benches` / `cargo bench` not run this session
