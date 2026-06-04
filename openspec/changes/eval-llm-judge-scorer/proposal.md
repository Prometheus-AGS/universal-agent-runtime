# eval-llm-judge-scorer

## Why

The eval harness has only rule scorers; the deferred **LLM-as-judge** scorer (the one EH3 cut from v1) is still missing. EHH3 added the `ScorerSpec` factory seam; this change (EHH2) adds the judge so suites can grade open-ended answers against a rubric. Per phase decision **D-B**, judge scores are **advisory** — computed, persisted, and reported, but never part of the hard regression gate (which stays deterministic rule scorers).

## What Changes

- **`LlmJudge` scorer** (`src/uar/eval/judge.rs`) — implements the async `Scorer` trait over an `Arc<dyn CompletionProvider>` captured at construction. `score()` builds a rubric prompt (rubric + case input + candidate output, instructing JSON-only output), calls the provider, and parses a verdict.
- **Deterministic verdict parse (D-C):** expect JSON `{ "score": 0.0–1.0, "reason": "…" }`; tolerant extraction (first `{`…last `}`); clamp to 0–1; on any failure → `Score` value `0.0` + a detail string. Never panics, no `unwrap`.
- **`ScorerSpec::LlmJudge { rubric }`** variant + factory arm; `build_scorers` gains an `&Arc<dyn CompletionProvider>` parameter so the judge can be constructed with the run's provider.
- **CLI** builds its provider as `Arc<dyn CompletionProvider>` and passes it to `build_scorers`.

Out of scope: per-judge model override (v1 uses the run's provider/model); judge-in-hard-gate (D-B keeps it advisory); per-case scorers.

## Capabilities

### Modified Capabilities
- **`eval-harness`** — delta `specs/eval-harness/spec.md`. Adds an LLM-as-judge scorer (rubric-graded, deterministic parse, advisory).

## Impact

- **Affected code:** new `src/uar/eval/judge.rs` (`LlmJudge` + prompt + parse); `src/uar/eval/scorer_spec.rs` (`LlmJudge` variant; `build_scorers`/`ScorerSpec::build` take a provider); `src/uar/eval/mod.rs` (re-export `LlmJudge`); `src/uar/eval/cli.rs` (provider as `Arc<dyn CompletionProvider>`).
- **Behavior preservation (Rule 32):** suites that don't declare `llm_judge` are unaffected; the gate semantics (rule scorers via `compare`) are unchanged — the judge just adds a reported score.
- **No new dependency** (Rule 27): reuses `serde`/`serde_json`/`async_trait` + the existing `CompletionProvider`.
- **KBD workflow state:** YES — EHH2, round 2 of `eval-harness-hardening`.
