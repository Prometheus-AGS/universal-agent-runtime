# CH-17 eval-targeted-suites

## Why

The eval harness (built across `uar-eval-harness`/`eval-harness-hardening`)
only had one suite (`starter.yaml`), which grades LLM *completion* quality.
Nothing evaluated the runtime's *decision* code directly: does the skill
matcher (CH-08 instruments it) activate the right skill for a query? Does
the model router (CH-03/CH-09) pick the right model under a capability
filter, and correctly exclude an unhealthy provider? Does the context
strategy (CH-05) resolve to the right tier for a given model's context
window? These are deterministic code paths, but had zero eval coverage.

## What changed

- **`src/uar/eval/targeted.rs`** (new): three `CompletionProvider` impls
  that call real runtime code, not an LLM:
  - `SkillActivationProvider` — seeds 5 fixture skills into a real
    `SkillService`, calls the actual `match_skills` keyword matcher.
  - `RoutingProvider` — seeds a real `ProviderRegistry` with 2 stable
    catalog entries (`openai/gpt-4o`, `anthropic/claude-3-5-haiku`), calls
    the actual `ModelRouter::route`. Supports a `trip_health_for` case
    field to exercise CH-03's cooldown-exclusion path deterministically.
  - `ContextEfficiencyProvider` — calls `strategy_for_model` (CH-05)
    directly; no fixture needed, it's a pure function.
- **`RouteRequirements` gained `Deserialize`** (`src/llm/router.rs`) so
  eval case inputs can be authored as plain JSON.
- **`src/uar/eval/cli.rs`**: `run_suite` recognizes the 3 new suite names
  (`skill-activation`, `routing-accuracy`, `context-efficiency`) and
  dispatches to the matching fixture provider instead of building a real
  `Orchestrator` — every other suite name is unaffected.
- **3 new suite files** under `evals/`: `skill-activation.yaml` (6 cases),
  `routing-accuracy.yaml` (3 cases), `context-efficiency.yaml` (3 cases).
- **Committed baselines**: unlike `starter.yaml` (whose baseline requires
  a real paid model call to seed — an operator-only action, still open),
  these 3 suites are fully keyless and deterministic, so their baselines
  were seeded and committed in this same change:
  `evals/results/{skill-activation,routing-accuracy,context-efficiency}.baseline.json`
  (all `contains: 1.0`).
- **`.gitignore`**: `evals/results/*.json` ignored except
  `*.baseline.json` — per-invocation timestamped run results are local
  artifacts; only the committed baseline is shared (this pattern existed
  in the docs but had no enforcing `.gitignore` rule before).
- **New Tier-1 CI guard**: `targeted_suites_are_valid_and_score_perfectly`
  in `integration_tests.rs` runs all 3 suites through their real fixture
  providers and asserts every score is exactly 1.0 — catches suite-file
  and fixture drift, not just parse errors.

## Why these are simpler than the two-tier gate implies

Because all 3 are keyless, they don't need a Tier-2 (`eval-nightly.yml`,
API-key-gated) counterpart the way `starter.yaml` does — the Tier-1 Rust
test (`cargo test`, runs on every PR) already exercises them against their
real fixture providers with a committed baseline. No CI YAML changes were
needed; they ride the existing `cargo test` job that already gates every PR.

## Verification

- `cargo test --lib eval::` — 44/44 green (was 31 before this change: 9
  new unit tests in `targeted.rs` + 1 new Tier-1 integration test).
- Live CLI smoke test (no `UAR_LLM__API_KEY` set anywhere in the shell):
  `eval run skill-activation --update-baseline`,
  `eval run routing-accuracy --update-baseline`,
  `eval run context-efficiency --update-baseline`, then a non-`--update`
  re-run of all 3 — all exit 0, all score `contains: 1.000`, all show
  `Δ +0.000` against the freshly committed baseline.
- Full suite: `cargo test --lib` 341/341 green.
