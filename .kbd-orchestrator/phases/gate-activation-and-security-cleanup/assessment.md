# ASSESSMENT: gate-activation-and-security-cleanup

Project: universal-agent-runtime · Date: 2026-06-04 · Backend: OpenSpec
Assessing model: Opus 4.8 (frontier)
**Origin:** follow-up seeds from `eval-harness-hardening` reflection — make the nightly eval gate *enforce* (not just report), and close the carried secret-logging item.

---

## Goal

1. **Gate activation** — turn the Tier-2 nightly eval gate from "informational until seeded" into one that actually enforces a regression bar (or fails loudly when it can't).
2. **Security cleanup** — ensure no plaintext secrets are logged (the carried Rule 33 item).

---

## Current state (grounded) — two big surprises

### 🟢 Security cleanup is ALREADY DONE on `main`
The carried "secret-logging at `main.rs:46`" item is **resolved in the current tree** — verified by reading the code, not assuming:
- `config.rs` defines `REDACTED` + `redact_opt()` and **custom `Debug` impls** that mask every secret-bearing field:
  - `LlmConfig` (line 1362) → `api_key` and **all `provider_keys` values** redacted (provider *ids* kept, sorted).
  - `SecurityConfig` (264) → `jwt_secret` redacted.
  - `PersistenceConfig` (≈398) and `MemoryConfig` (≈811) → `surreal_pass`, `openai_api_key`, `cohere_api_key` redacted.
- `main.rs:48` (`tracing::info!("Configuration loaded: {:?}", config)`) is the only full-config dump, and it now routes through those redacting impls.
- **Audit for other leak sites:** the only `tracing` calls touching credentials are in `credentials.rs`/`server.rs`, and they log *error messages* (`error = %e`) or status strings — **no secret values**. No other `{config:?}` dump exists.

⇒ **Goal 2 is essentially MET already.** Remaining work is verification + dismissing the still-open spawn-task chip. *(The earlier EH5 smoke-test dump that showed a plaintext key predates this fix; current `main` is clean.)*

### 🟡 Gate activation is mostly operator-action, with one code gap
- The nightly workflow (`.github/workflows/eval-nightly.yml`) and `eval run` exist and gate on regression **only if a baseline file is present**. With no baseline, `run_suite` does `load_baseline(...).unwrap_or_default()` → empty baseline → `compare` finds no regression → **exit 0** (silently passes).
- No baseline ships (`evals/results/` has none). So today the gate is a **no-op smoke test**, and *nothing in the tooling signals that*. That's the real code gap: the unseeded state is invisible.
- Seeding the baseline + configuring the `UAR_LLM__API_KEY` GitHub secret + running `workflow_dispatch` are **operator actions** the agent cannot perform (no access to repo secrets or the Actions runner).

---

## Gaps

| # | Gap | Agent-deliverable? | Evidence |
| - | --- | ------------------ | -------- |
| G1 | **Unseeded gate passes silently** — `eval run` exits 0 with no baseline; no signal that the gate isn't really gating. | **YES (code)** | `cli.rs::run_suite` baseline fallback |
| G2 | **No baseline shipped** → Tier-2 can't enforce. | **PARTLY** — agent could generate one locally via a real model call (decision D-A); committing it is then trivial. Otherwise operator-only. | `evals/results/` empty |
| G3 | **GitHub secret + first workflow run** — `UAR_LLM__API_KEY` not configured; nightly never executed. | **NO (operator)** | no secret; workflow unproven |
| SC1 | Secret-redaction (Rule 33) — **already implemented**; needs verification + chip closure. | verify only | `config.rs` redacting Debug impls |
| QA1 | Artifact-refiner QA-gate automation (carried 3 phases) — process tooling, not really this phase's theme. | out of theme | no `.refiner` runs for recent phases |

---

## Reusable building blocks

- `EvalAction::Run` (config.rs:111) already has `threshold`/`results_dir`/`update_baseline` — adding a `require_baseline` flag is a one-field extension + a guard in `run_suite`.
- `load_baseline` already returns `Ok(None)` when absent — the guard just needs to act on `None`.
- The redaction infra (`REDACTED`, `redact_opt`, per-struct `Debug`) is the template for any future secret field.
- `.env` holds working provider keys (alibaba/openai/groq) — so local baseline generation (D-A) is *technically* possible.

---

## Proposed architecture / changes (small phase)

1. **`--require-baseline` gate-strictness flag (GA1)** — `EvalAction::Run` gains `require_baseline: bool`; `run_suite` returns non-zero (with a clear message) when `require_baseline` is set and no baseline exists. The nightly Tier-2 step adds `--require-baseline` so the unseeded state **fails loudly** instead of passing silently. Makes the gate's status honest. *(Small, pure-code, fully agent-deliverable.)*
2. **Operator runbook (GA-DOC)** — extend `evals/README.md` (or a short `docs/eval-gate.md`) with the exact operator steps: set the `UAR_LLM__API_KEY` secret + optional `vars.UAR_EVAL_MODEL`, run `workflow_dispatch --update_baseline`, commit `evals/results/starter.baseline.json`, verify a deliberate regression fails. *(Agent-deliverable doc; the execution is the operator's.)*
3. **(Decision D-A) Local baseline seed** — optionally have the agent run `eval run evals/starter.yaml --update-baseline` against a real model now and commit the baseline, so Tier-2 enforces immediately. *Trade-off:* a real LLM call (token cost; Rule 8) and the baseline encodes today's model output, which may not be the intended reference. **Needs a decision.**
4. **(SC1) Verify + close security** — a build/run check that the config dump is masked + dismiss the spawn-task chip. *(No code; verification only.)*

---

## Key product decisions (for `/kbd-plan`)

- **D-A — seed the baseline now, or leave to the operator?** (a) Agent runs a real eval locally and commits `starter.baseline.json` (gate enforces immediately, costs a few model calls, baseline = today's output). (b) Ship only `--require-baseline` + the runbook; the operator seeds deliberately (no agent model calls; baseline seeded under human control). **Recommend (b)** — seeding is a judgment call about the reference bar and belongs to the operator; `--require-baseline` makes the unseeded state safe meanwhile.
- **D-B — scope of this phase:** (a) minimal — GA1 + runbook + close security (recommended; honest given how little is left); (b) also fold in QA1 (refiner automation) — but that's process tooling unrelated to the gate/security theme and arguably its own phase. **Recommend (a)**; track QA1 separately.
- **D-C — is `--require-baseline` even wanted,** or should *every* `eval run` warn-but-pass on missing baseline? Recommend an opt-in flag (default off preserves EH5 behavior, Rule 32) used by the nightly.

---

## Complexity & risk

- **GA1** — trivial, additive, behavior-preserving (flag defaults off). Lowest risk.
- **D-A local seed** — only risk if chosen: a real model call (cost, possible auth failure, non-deterministic baseline). Mitigated by recommending the operator path.
- **Honest scope note:** this is a **small phase** — one real code change (GA1), one doc, and a verification. ~½ the original intent (security) is already done, and ~¼ (GitHub secret + first run) is operator-only. Surfacing this rather than inventing busywork.
- No new dependencies anticipated (Rule 27).

---

## Assessment status

**COMPLETE.** Security cleanup is already implemented on `main` (verified) — the phase reduces to: **GA1** (`--require-baseline`, agent-deliverable), a **runbook** for the operator-only steps, and **closing** the security item. Decisions D-A (local seed vs operator), D-B (scope), D-C (flag shape) are for `/kbd-plan`. This is honestly a small phase. Next: `/kbd-plan gate-activation-and-security-cleanup`.
