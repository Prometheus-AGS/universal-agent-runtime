## Context

Greenfield: no eval module exists. Reusable: `crate::uar::quality::detect(&SycophancyConfig, &str) -> Option<SycophancyOutcome>` (rule-based, `outcome.score: f32` 0..1), `serde`/`serde_json` (already deps). No `regex` crate in deps. This change adds only the domain + scorers (foundation); the runner/persistence/CLI are EH2/EH4/EH5.

## Goals / Non-Goals
**Goals:** typed serializable domain; a `Scorer` trait that can host both rule and (future) LLM-judge scorers; pure deterministic rule scorers; full unit tests.
**Non-Goals:** runner, suite loading, persistence, CLI/HTTP, LLM-judge, full-agent-run — all later changes.

## Decisions
- **D1 — module layout:** `src/uar/eval/{mod.rs, domain.rs, scorers.rs}`. `mod.rs` re-exports the public types + scorers. Registered with `pub mod eval;` in `uar/mod.rs`.
- **D2 — `Scorer` is async** (`async fn score`) so LLM-judge scorers fit later without trait churn; rule scorers compute synchronously and return ready. Boxed as `Arc<dyn Scorer>` for suites.
- **D3 — `Score.value: f32` clamped to 0.0–1.0** in a constructor (`Score::new(scorer, value, detail)` clamps) so every scorer is normalized by construction.
- **D4 — Sycophancy adapter** calls `quality::detect(&SycophancyConfig::default(), output)`; `value = 1.0 - outcome.score`; `None` (empty/disabled) ⇒ 1.0 (treat as clean). Detail carries the flagged pattern ids when present.
- **D5 — Regex scorer without the `regex` crate:** v1 uses literal/substring + simple anchors (starts_with/ends_with/contains) configured per case, OR is named `PatternMatch` doing contains; avoids adding a dependency (Rule 27). (A true-regex scorer can arrive with EH3/later if `regex` is justified.)
- **D6 — serde:** all domain types `#[derive(Serialize, Deserialize)]` for EH4 file persistence; `run_at` stored as an RFC3339 string (set by the runner later, not in pure scoring).

## Risks / Trade-offs
- **[Async trait overhead]** `async fn` in trait (Rust 1.75+ native async-fn-in-trait or `async_trait`) → Mitigation: the repo already uses `async_trait` widely; reuse it for `Scorer`.
- **[Regex limitation]** substring-only "regex" is weaker → Mitigation: documented; true regex deferred to avoid a dep; covers the common contains/format checks.
- **[Sycophancy default config]** uses `SycophancyConfig::default()` (enabled, standard) regardless of app config → Mitigation: intentional — eval scoring should be independent of runtime gating; documented.

## Migration Plan
1. Create `src/uar/eval/` (domain + Scorer trait + scorers) + `pub mod eval;`.
2. Unit tests: domain serde round-trip; each scorer positive/negative; Score clamping; sycophancy clean vs flagged.
3. `cargo check`/`clippy`/tests.
- Rollback: pure additive module; revert deletes it. Nothing depends on it yet.

## Open Questions
- True-regex scorer: add the `regex` crate in EH3, or keep substring-only? (Defer; v1 substring is enough for format/contains checks.)
