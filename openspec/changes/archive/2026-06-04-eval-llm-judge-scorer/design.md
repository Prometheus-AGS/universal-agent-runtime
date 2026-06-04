## Context

EHH3 added `ScorerSpec` + `build_scorers(suite)` + the async, object-safe `Scorer` trait. `CompletionProvider::complete(&self, input) -> Result<String>` is the model seam; the CLI's `OrchestratorCompletionProvider` wraps `chat_non_streaming`. A judge must call completions from inside `Scorer::score`, which only receives `(case, output)` — so it must hold its provider.

## Goals / Non-Goals
**Goals:** an `llm_judge` scorer, rubric-prompted, deterministic-parse, advisory (D-B).
**Non-Goals:** per-judge model override; judge in the hard gate; per-case scorers.

## Decisions
- **D1 — `LlmJudge` holds `Arc<dyn CompletionProvider>`** captured at construction (cloned from the run's provider). In `src/uar/eval/judge.rs`.
- **D2 — factory threads the provider:** `build_scorers(suite, provider: &Arc<dyn CompletionProvider>)` and `ScorerSpec::build(&self, provider)`. Rule arms ignore it; the `LlmJudge` arm clones it. `default_scorers` is unchanged (no provider).
- **D3 — `ScorerSpec::LlmJudge { rubric: String }`** (no `model` field in v1 — the judge uses the run's provider/model; a per-judge model override waits until `CompletionProvider` supports model selection).
- **D4 — prompt:** instruct JSON-only: `{"score": <float 0..1>, "reason": "<short>"}`, with the rubric, the case input, and the candidate output.
- **D5 — parse (deterministic, tolerant):** extract the substring from the first `{` to the last `}`; `serde_json` into `Verdict { score: f32, #[serde(default)] reason: String }`; clamp `score` to 0–1; success → `Score::new("llm_judge", score, reason?)`; any failure (no JSON, parse error, provider error) → `Score::new("llm_judge", 0.0, Some(detail))`. No `unwrap`/panic. `score: f32` (not f64) avoids a truncating cast.
- **D6 — advisory (D-B):** the judge contributes a `Score` like any scorer; the hard gate is `compare` over rule-scorer means. No gate code changes — `compare` already treats all scorers uniformly and the gate threshold is applied per scorer; judge variance is accepted because the CI gate (EHH1) runs only rule scorers in its deterministic tier. Documented; no special-casing in `compare` this change.
- **D7 — CLI:** `let provider: Arc<dyn CompletionProvider> = Arc::new(OrchestratorCompletionProvider { orchestrator });` then `build_scorers(&suite, &provider)` and `Runner.run(.., provider.as_ref(), ..)`.

## Risks / Trade-offs
- **[verdict format drift]** models wander off JSON → Mitigation: tolerant first-`{`…last-`}` extraction + contained failure (0.0 + detail); unit-tested against clean / prose-wrapped / malformed / out-of-range.
- **[judge non-determinism in a gate]** → Mitigation (D-B/D6): advisory only; EHH1's hard tier uses rule scorers. Documented.
- **[cost]** the judge calls the model once per case → only runs when a suite declares it / in the nightly tier; the structural CI tier uses a stub.
- **[provider signature ripple]** `build_scorers` gains a param → localized to the factory + its one CLI caller + EHH3's tests (add a stub provider).

## Migration Plan
1. `judge.rs`: `LlmJudge`, `judge_prompt`, `parse_verdict`, `extract_json_object` + tests (stub provider).
2. `scorer_spec.rs`: `LlmJudge` variant; `build`/`build_scorers` take `&Arc<dyn CompletionProvider>`; update tests (stub provider).
3. `mod.rs`: `pub use judge::LlmJudge;`.
4. `cli.rs`: provider as `Arc<dyn CompletionProvider>`; pass to factory + Runner.
5. Verify: check/clippy/test `eval::`; `openspec validate --strict`.
- Rollback: additive (new file + new variant + a param); revert restores EHH3.

## Open Questions
- Per-judge model override + judge-in-gate posture revisited only if advisory proves insufficient.
