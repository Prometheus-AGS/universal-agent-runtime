# fix-uar-integration-test

## Why

`cargo check --test uar_integration` failed with `E0063: missing
fields authors, compatibility, language and 5 other fields in
initializer of Skill` at `tests/uar_integration.rs:430` — pre-existing,
unrelated to any tracked change in the prior phase (`uar-spec-v2-and-polish`),
first discovered while verifying that phase's CH-20/CH-14.

## What changed

`Skill` derives `Default`, so the fix is a one-line addition:
`..Default::default()` after the explicitly-set fields in the test's
`Skill { ... }` literal, filling `authors`, `compatibility`, `language`,
and the other newer fields with their defaults instead of listing all
8 explicitly.

## Verification

- `cargo check --test uar_integration`: clean (`Finished` profile, no
  errors). One residual IDE diagnostic still showed the old error at
  time of writing — confirmed stale via a second `cargo check` re-run
  (0 `error` lines in output), matching the same stale-diagnostic
  pattern already seen once this session with `parser.rs`.
