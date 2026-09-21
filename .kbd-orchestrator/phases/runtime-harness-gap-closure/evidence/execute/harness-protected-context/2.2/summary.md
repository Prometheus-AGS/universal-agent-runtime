# harness-protected-context task 2.2

Execute replaced destructive tool-history normalization with lossless validation. Missing, orphaned, duplicate, conflicting and misplaced records now return explicit invalid-history outcomes without modifying canonical history or dispatching a provider request. Reduction validates first and returns `protected_history_overflow` if any reducer removes, alters or reorders a completed tool group or a meaningful multimodal assistant record.

RunManager now persists an assistant turn whenever tool calls exist, including empty-text pending calls that end before a result. Pending call identities are retained in run context, and an unresolved call blocks resume dispatch. Host-proven terminal failures are serialized as typed error results with call identity and trusted-host provenance. When a persistence store is configured, the canonical terminal receipt is stored before the result enters provider history; a storage failure stops the stream without fabricating a result.

The production changes are in `src/uar/runtime/context/normalize.rs`, `src/uar/runtime/context/reduce.rs`, `src/uar/runtime/manager.rs`, `src/llm/orchestrator.rs`, and the required reduction call-site propagation in `src/uar/runtime/turn/builtin.rs`. `src/uar/context/strategy.rs` required no edit because the single reduction boundary now detects protected loss for every declared strategy. Regression coverage is in `tests/context_history_integrity.rs`; the existing generic strategy suite remained unchanged.

Final verification on the recorded source state:

- Tier 0: `RUSTC_WRAPPER= cargo check --locked --no-default-features --features server-full` finished warning-free in 13m 12s.
- Tier 1: `RUSTC_WRAPPER= cargo test --locked --no-default-features --features server-full --test context_history_integrity` passed 22 tests, failed 0, ignored 2 frozen baseline diagnostics.
- Tier 1: `RUSTC_WRAPPER= cargo test --locked --no-default-features --features server-full --test test_context_strategy` passed 4 tests, failed 0.
- `cargo fmt --all -- --check`, `git diff --check`, and `openspec validate harness-protected-context --strict` passed.

The first isolated artifact review found two critical gaps: pending calls were lost at final flush, and terminal failures used plain non-durable strings. Both were corrected and covered by production-path regressions. The isolated re-review returned PASS with zero findings.

No unrelated feature or guard was added. The invalid-history guard traces to the observed destructive normalizer and malformed provider dispatch. The protected-overflow guard traces to the observed whole-group deletion. Pending persistence and terminal settlement trace to the critic-reproduced loss and provenance gaps. No live external provider was contacted. Postgres receipt persistence and crash/restart behavior remain outside this unit and are not claimed by these local checks.
