# Current Waypoint

- Phase: `gate-activation-and-security-cleanup` **(reflect_complete)**
- Previous phase: `eval-harness-hardening` (complete — 4/4 MET + HK1)
- Backend: OpenSpec
- Status: `complete`
- Progress: **1 / 1 change shipped** (PR #43 — 2 commits — merged + archived)
- Exact next command: `/kbd-new-phase`
- Reflection: [reflection.md](phases/gate-activation-and-security-cleanup/reflection.md)
- Updated at: 2026-06-04

## Phase arc outcome

**Both goals MET (code side), in one PR (#43, two commits).**

- **`feat(eval): --require-baseline strict gate` (GA1)** — the unseeded nightly now fails loudly instead of passing silently; pure `baseline_missing_under_strict` helper; **fail-fast exit 2 before any model call**; nightly opts in; operator runbook in `evals/README.md`.
- **`fix(config): redact secrets in config Debug output` (Rule 33)** — redacting `Debug` for `LlmConfig` (api_key + provider_keys), `SecurityConfig` (jwt_secret), `Persistence`/`Memory` passwords.

36 eval lib tests green.

## ⚠️ Correction recorded

I mis-assessed the redaction as "already on `main`." It was **uncommitted working-tree WIP** (`origin/main` had 0 `REDACTED`) — this PR actually lands it. Caught at commit time (diff showed it as additions), **split into two focused commits**, and disclosed in the PR body. Lesson banked: verify "already merged" against committed state (`git show origin/main:<file>`), never the working tree.

## Remainder (operator-only — by design)

The gate code is in place but enforces *green* only once a human:
1. sets the `UAR_LLM__API_KEY` secret (+ optional `vars.UAR_EVAL_MODEL`),
2. runs `eval-nightly` with `update_baseline=true`,
3. commits `evals/results/starter.baseline.json`,
4. confirms a deliberate regression fails.

Until then the scheduled job fails loudly ("blocked until seeded") — intended.

## Other follow-ups

- **Hygiene:** resolve the long-lived dirty working tree (`static/index.html`, untracked `.agents/`/`.firecrawl/`/`.zed/`) so future `git add` can't capture stray WIP.
- Spawn-task "redact secrets" chip → **resolved by PR #43** (dismiss it).
- Carried: refiner QA-gate automation (3 phases); per-case scorers; per-judge model; HTTP eval endpoint; SurrealDB storage.

## Next

`/kbd-new-phase` — likely candidates: artifact-refiner QA-gate automation, or finishing H8 metric recorders. (Gate activation itself is an operator action, documented above.)
