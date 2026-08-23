# Evaluations

> **Current authority:** [Inference workflow guide](/docs/providers/inference).
> Evaluation results are local, source-bound evidence for the named suite,
> provider, model, and profile only.

This directory contains suites for the runtime evaluation CLI
(`eval run`, `eval list`, and `eval baseline`). Each YAML suite declares cases
and optional scorers. Results and accepted baselines live in `evals/results/`.

## Scorers and evidence

Deterministic scorers include `exact_match`, `contains`, `json_valid`,
`non_empty`, `pattern_match`, and `sycophancy`. `llm_judge` is advisory and must
name the judge provider, model, prompt version, and temperature in retained
evidence.

Recorded or stubbed providers may diagnose parsing and harness wiring, but they
do not count as inference integration or model-quality evidence. A certifying
inference run must traverse a supported packaged UAR boundary and reach a real
loaded model through the configured provider path.

## Run locally

List suites and run a selected suite with the runtime CLI:

```bash
cargo run --bin universal-agent-runtime -- eval list
cargo run --bin universal-agent-runtime -- \
  eval run evals/starter.yaml --require-baseline
```

Create or update a baseline only after reviewing genuine model output:

```bash
cargo run --bin universal-agent-runtime -- \
  eval run evals/starter.yaml --update-baseline
```

Commit the resulting baseline deliberately. A missing baseline or unavailable
real-model prerequisite leaves the corresponding quality claim unverified.
GitHub Actions are deployment-only and do not run these suites.
