# Evaluations

Golden suites for the runtime's eval harness (`eval run|list|baseline`).

## Layout

- `evals/<suite>.yaml` — a suite: `name`, `cases` (`id`, `input`, optional `expected`),
  and optional suite-level `scorers`. With no `scorers`, a default set is used
  (non-empty + sycophancy, plus exact-match + contains when every case has `expected`).
- `evals/results/` — run results (`<suite>-<ts>.json`) and the committed baseline
  (`<suite>.baseline.json`).

## Scorers

Declared per suite via `scorers:` (snake_case `type`): `exact_match`, `contains`,
`json_valid`, `non_empty`, `pattern_match` (`pattern` + `mode`), `sycophancy`,
and `llm_judge` (`rubric`). `llm_judge` is **advisory** — it is reported and
persisted but does not fail the regression gate; the hard gate uses the
deterministic rule scorers.

## Two-tier CI gate

- **Tier 1 — every PR (no key, no cost):** a deterministic structural test
  (`src/uar/eval/integration_tests.rs`) loads `evals/starter.yaml`, builds its
  scorers, and runs it through a recorded provider — proving the suite parses and
  the harness wiring is intact. No model is called.
- **Tier 2 — scheduled (`.github/workflows/eval-nightly.yml`):** runs
  `eval run evals/starter.yaml` against the real model using the `UAR_LLM__API_KEY`
  secret and **exits non-zero on regression** vs the committed baseline. If the
  secret is absent (e.g. forks), the job skips the real-model step without failing.

## Establishing / updating the baseline

No baseline is shipped (it needs real model outputs). To seed or update it:

```bash
cargo run --bin universal-agent-runtime -- \
  eval run evals/starter.yaml --update-baseline
git add evals/results/starter.baseline.json && git commit -m "chore(eval): update starter baseline"
```

Until a baseline is committed, the scheduled run reports scores and is clean
(a run can establish expectations). Baselines are updated by a deliberate commit,
never auto-committed from CI.
