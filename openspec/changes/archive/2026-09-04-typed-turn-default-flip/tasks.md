# Tasks — typed-turn-default-flip

scope: src/config.rs (harness mode default), settings schema and tests, docs release notes, .prometheus/decisions.md

## 0. Evidence gate

- [x] 0.1 Attach `parity-report.json` from typed-turn-assembly showing zero unexpected differences across the corpus; record corpus size
- [x] 0.2 Run the live smoke set in `shadow` mode; record command, output, and zero unexpected differences

## 1. Failing test first

- [x] 1.1 Settings-default test asserts `HarnessConfig::default().mode == Typed` and that `legacy` still deserializes

## 2. Flip

- [x] 2.1 Change the default to `typed`; add the deprecation note for `legacy` to the settings schema
- [x] 2.2 Release notes entry naming the rollback setting

## 3. Verification

- [x] 3.1 Tier 2: fmt check and full `cargo test --locked --no-default-features --features server-full` with the new default
- [x] 3.2 Decision log entry with the evidence from 0.1 and 0.2
- [x] 3.3 `openspec validate typed-turn-default-flip --strict`

## Evidence receipt — 2026-09-04 (0.1)

Attached `parity-report.json` byte-for-byte from typed-turn-assembly; `cmp`
exited 0. Corpus size: 3 (basic user turn, host instructions, memory
contribution). Unexpected differences: 0; allowlisted differences: 0; every
case dispatched only the legacy request. The corpus-driven test validated this
report during the completed phase-end focused and full local test runs.
This is a small corpus, not evidence that all MCP, skill and child-thread
combinations are covered. At this receipt's creation, task 0.2 still required a
nonempty live shadow record; the following receipt satisfies that gate.

## Evidence receipt — 2026-09-04 (0.2)

The command and output in `evidence/README.md` record two real k3 cases, both
completed with text and one shadow comparison each; unexpected differences and
allowlisted differences are zero. The command exited 0. Both project and phase
decision logs record the three-case corpus and two-case live set and their
coverage limitations. An independent artifact critic found no concrete blocker.

The default/rollback test was written alongside the default change; execution
is deferred to the phase-end suite, following the operator's full-code-before-
tests instruction rather than the original failing-test-first heading.

## Phase verification — 2026-09-04 (initial failure)

`cargo check --locked --no-default-features --features server-full` exited 0
in 53.34s after the default/schema/test edit. Formatting and strict OpenSpec
validation passed. The full suite compiled in 6m15s and the new default/rollback
test passed, but the command exited 101 at BDD: 8/9 scenarios passed; the
multi-turn case failed before its request because server readiness exceeded
30 seconds. No default-flip completion is claimed from this run.

The shared live-test helper now waits up to 120 seconds for startup (standard
skill discovery/reconciliation previously exceeded 30 seconds with 1,044 user
skills). Its enclosing child-process wait is 180 seconds so it covers startup
plus the existing 30-second health probe. Request/health assertions and
production timeouts are unchanged. Independent artifact review found no scope
or nesting blocker. A fresh Tier 0, formatting and full phase run is underway.

## Final phase verification — 2026-09-04

The rerun exited0 after Tier0, formatting, all executed test targets and
doctests. Library:710 passed,1 ignored. BDD:9 scenarios and49 steps passed.
Broad integration:93 passed,1 ignored in921.95s. Doctests:26 passed,17 ignored.
The new default/rollback test passed, as did the parity corpus. Exact command,
selected output and limitations are in `evidence/phase-test-report.md`.
