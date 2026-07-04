# CH-20 perf-security-load

## Why

G5 (Polish & Release) requires evidence, not assumptions, about how the
router/dialect/context hot path performs under load, whether the existing
prompt-injection heuristic actually holds up under adversarial input, and
whether `src/server.rs` (now 5,068 lines, up from 4,922 at the start of
this phase) still belongs as a single file. None of this evidence existed
in the repo before this change: no `benches/` directory, no load-test
harness, no injection-resistance test suite beyond the heuristic's own
happy-path tests, and no structural assessment of `server.rs`.

## What changed

Four independent, read-mostly workstreams (per plan.md, this change runs
in Tranche B alongside CH-13/CH-17 — fully independent of the CH-12→13→
{14,15} spec-v2 chain):

1. **Hot-path profiling** (`benches/hot_path.rs`, new): Criterion
   benchmarks for `PromptDialect::detect`, `strategy_for_model`,
   `apply_strategy` (sliding-window trim over a realistic 500-message
   conversation), and `ModelRouter::route` (async, seeded registry).
   `harness = false` in `Cargo.toml` since Criterion drives its own
   `main()`; `criterion` added as a dev-dependency (`html_reports`,
   `async_tokio` features).
2. **Concurrent-agent load test**
   (`tests/integration/live/load_test.rs`, new): boots one real server
   against the in-process stub LLM and fires 50 concurrent chat-completion
   requests at it, reporting p50/p95/max latency and throughput. Registered
   in `tests/integration/live/mod.rs` and given a `MATRIX.md` row. Deliberately
   a soak/smoke test (asserts zero failures under load), not a strict
   perf-regression gate — CI hardware varies too much for a hard SLA to be
   meaningful at this level; that's what the Criterion benchmarks are for.
3. **Prompt-injection resistance review** (`src/uar/guardrails.rs`):
   found and fixed a real evasion — the existing substring scan didn't
   normalize whitespace, so padding a phrase with extra spaces or a line
   break defeated it. Added `normalize_whitespace` (collapses any run of
   Unicode whitespace to one ASCII space) ahead of the match. Added an
   honest test inventory: cases the heuristic now closes (whitespace/
   line-break/tab evasion, all known phrases with surrounding noise) and
   cases it does **not** catch and is not claimed to catch (synonym/
   paraphrase substitution, indirect roleplay framing, base64-encoded
   payloads) — disclosed as known gaps for a future classifier-based
   approach to use as its own regression baseline.
4. **`server.rs` split assessment** (`docs/server-rs-split-assessment.md`,
   new): current-structure breakdown by line range, a recommended target
   module layout, and a 4-step extraction sequence (Anthropic shim →
   OpenAI shim → admin API → bootstrap-only `mod.rs`). This is an
   assessment and recommendation, not an executed split — per Rule 31
   ("Prefer Small, Reviewable Changes") and Rule 8 ("Minimize Irreversible
   Actions"), moving ~5,000 lines of handler code is its own
   dedicated, narrowly-scoped effort, not something to rush inside a
   7-change phase.

## Verification

- `cargo check` (lib+bin): clean.
- `cargo check --test integration` (includes `load_test.rs` via
  `tests/integration/live/mod.rs`): clean.
- `cargo test --lib uar::guardrails::`: 13/13 green (6 new CH-20 tests:
  3 evasion-closes, 3 disclosed known-gaps, 1 `normalize_whitespace` unit
  test).
- Found and fixed one incidental bug in the diff itself (test module's
  `use super::{...}` import list was missing `INJECTION_PHRASES` and
  `normalize_whitespace`, both referenced by the new tests).
- Found and fixed one unrelated pre-existing compile error blocking
  `cargo check --tests` entirely: `tests/settings_persistence.rs`'s
  `minimal_config()` was missing the `guardrails` field on `AppConfig`
  (added in `c454431`, mount-governance-guardrails, never backfilled into
  this test). Landed as its own separate commit, not folded into this
  change's diff.
- `benches/hot_path.rs` was not run under `cargo bench` this session
  (deliberately out of scope for this pass — see carried-over follow-up
  below); it does compile as part of `cargo check --tests`... actually
  benches are checked separately via `cargo check --benches`, which was
  not run this session per explicit user direction to skip `--benches`.

## Known follow-up (not blocking, disclosed)

- `cargo check --benches` / `cargo bench --bench hot_path` have not been
  run in this session — the benchmark code has not been compiled or
  executed, only reviewed. A future pass should run it to confirm it
  actually compiles and produces sane numbers.
- Two pre-existing, unrelated compile failures were discovered while
  verifying this change and are **not** fixed here (out of scope):
  `tests/uar_integration.rs` (`Skill` struct literal missing 8 fields) and
  `tests/bdd.rs` (broken nested `#[path]` resolution — resolves to
  `tests/live/integration/live/harness.rs`, which doesn't exist). Neither
  file is part of any tracked G4/G5 change in this phase.
