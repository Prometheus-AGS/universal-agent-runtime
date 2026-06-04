## Context

EH5 gave `eval run` a non-zero exit on regression; EHH3 gave suites declared scorers; EHH4 gave a recorded-fixture provider + pipeline coverage. `ci.yml` runs fmt/clippy/check/test. The real `run` path needs a live model (cost + secret) so it cannot run per-PR (D-A).

## Goals / Non-Goals
**Goals:** ship a suite; guard it on every PR cheaply; gate on real-model regression on a schedule.
**Non-Goals:** auto-committing baselines from CI; per-PR real-model runs; HTTP endpoint.

## Decisions
- **D1 — starter suite:** `evals/starter.yaml`, model-agnostic cases with `expected` substrings; declared scorers `non_empty` + `contains` (deterministic, hard) + advisory `llm_judge` (D-B).
- **D2 — Tier 1 (PR):** an in-crate `#[cfg(test)]` test loads the shipped file via `env!("CARGO_MANIFEST_DIR")/evals/starter.yaml`, asserts it parses with cases + scorers, builds scorers (stub provider), runs, and asserts every case produced scores. Runs under the existing `test` job — no key, no model. Guards the suite from rotting + the wiring from breaking.
- **D3 — Tier 2 (scheduled):** `.github/workflows/eval-nightly.yml`, `schedule` (daily) + `workflow_dispatch` (with an `update_baseline` input). A guard step sets `run=true` only if `secrets.UAR_LLM__API_KEY` is non-empty; all model steps are `if: run == 'true'` so keyless/fork runs skip cleanly. Build `--release`, then `eval run evals/starter.yaml --results-dir evals/results` with `UAR_LLM__{MODEL,API_KEY}` env. Regression → non-zero exit (the binary already does this).
- **D4 — baseline persistence (git-friendly, D1 storage):** the baseline lives at `evals/results/starter.baseline.json`, committed deliberately. Updating it = run `eval run evals/starter.yaml --update-baseline` locally and commit. We do NOT ship a baseline (no real outputs yet) and do NOT auto-commit from CI; documented in `evals/README.md`. Until a baseline is committed, the scheduled run reports + is clean (a run can establish expectations).
- **D5 — security (Rule 33):** key is a secret, referenced only via `env:`, never echoed; guard avoids hard-failing forks.

## Risks / Trade-offs
- **[no baseline shipped ⇒ gate is informational until seeded]** → accepted + documented; seeding needs a real run. The PR tier still guards structure every PR.
- **[scheduled job cost]** → once daily, smallest sensible model via `vars.UAR_EVAL_MODEL` (default a mini model).
- **[run needs no DB]** `run_suite` builds only an `Orchestrator` (LLM client) — no persistence/server — so the job needs only the key. Verified by EH5's design.
- **[workflow YAML can't be unit-tested]** → validate syntax locally; first real exec via manual `workflow_dispatch` post-merge.

## Migration Plan
1. `evals/starter.yaml` + `evals/README.md`.
2. `integration_tests.rs`: Tier-1 `starter_suite_is_valid_and_runs` test.
3. `.github/workflows/eval-nightly.yml`.
4. Verify: `cargo test --lib eval::` (incl. the starter test); check/clippy; YAML sanity; `openspec validate --strict`.
- Rollback: additive (data files + a workflow + one test); revert removes the gate, harness unaffected.

## Open Questions
- Auto-baseline-commit + an HTTP eval endpoint remain follow-ups.
