# Findings: pin-surreal-memory-to-sha

## Change summary

`surreal-memory`'s git dependency moved from `branch = "main"` to
`rev = "f9ab1c29944b86d44c23ea0e6192fa3d39acbde8"`. This SHA was resolved
via `git ls-remote https://github.com/Prometheus-AGS/surreal-memory-server.git HEAD`
at planning time and re-verified immediately before applying the change —
unchanged, no drift between planning and execute.

## Verification

- `Cargo.lock` diff is exactly 1 line changed: the `source` field's query
  string moved from `?branch=main#<sha>` to `?rev=<sha>#<sha>` (same
  `<sha>` in both — pinning to the branch's current HEAD, not a different
  commit). No crate version changes anywhere else in the lockfile,
  confirming this was a pure pin-mechanism change with zero behavioral
  impact.
- `cargo check --lib --tests`: clean (same pre-existing warnings as
  baseline).
- `cargo test --lib`: 387/388 pass (1 pre-existing ignore) — unchanged.
- `cargo clippy --lib`: 499 warnings — unchanged, zero new.
- `cargo audit`: 11 vulnerabilities + 9 allowed warnings — identical
  count to the phase's own `assessment.md` baseline (all pre-existing,
  disclosed items from the prior phase; nothing new introduced by this
  pin change, as expected since the resolved commit didn't change).

## Why this is safe

Pinning to a branch's *current* HEAD at the moment of the change cannot
introduce new behavior — by definition, it's the exact commit already in
use. The value of this change is entirely forward-looking: the next
`cargo update` (routine or accidental) can no longer silently pull in
whatever `surreal-memory`'s `main` branch has moved to since; any future
upgrade now requires an explicit `Cargo.toml` edit to a new `rev`, the
same deliberate-upgrade pattern already used for `rmcp` and
`prometheus_parking_lot`.
