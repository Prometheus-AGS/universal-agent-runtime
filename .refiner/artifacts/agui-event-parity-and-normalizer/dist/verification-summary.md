# C-06 Deterministic Verification Summary

Passed:

- `pnpm -C frontend typecheck`
- `pnpm -C frontend lint`
- `node scripts/check-frontend-boundaries.mjs`
- Focused Vitest: 3 files, 22 tests
- Full Vitest: 36 files, 171 tests
- `pnpm -C frontend build`
- `cargo check --locked --no-default-features --features server-full`
- Focused Rust replay filter: 5 tests passed, including 3 C-06 tests; the corrected real-name tool projection test also passes independently
- Adversarial-review regression: eight buffered tool-argument chunks retain
  unique lifecycle event ids and monotonic source-event ordering
- Final isolated review: PASS, 0 critical / 2 warnings / 0 suggestions;
  verified-distinct judge and anti-sycophancy score 0.0
- C-06 Rust files pass direct `rustfmt --check`
- `openspec validate agui-event-parity-and-normalizer --strict`
- `git diff --check`

Known external condition:

- The live integration target does not compile because
  `tests/integration/live/harness.rs` initializes `Cli` without the required
  `strict_config` field and passes the stale config type to `start_server`.
  The C-06 live seam cases therefore did not execute.
- Repository-wide `cargo fmt --all -- --check` remains red on unrelated dirty
  Rust files outside C-06.
