# Model fleet

Profile in AGENTS.md: `mixed` (set `2026-08-09`)

AGENTS.md is per repository, not per model. Every model listed below reads the
same file, so the weakest one governs its content. Record the fleet here — a
profile choice nobody wrote down gets re-argued every time someone new arrives.

## Models that work this repo

All four reach the repo through liter-llm (`provider/model` addressing); the
harness column is where each one actually drives edits.

| Model | Harness | Scaffold needed | Measured |
|---|---|---|---|
| Claude Opus 5 | Claude Code (primary); liter-llm for review/judge calls | unknown | no |
| Kimi K3 | liter-llm gateway — adversarial judge | unknown | no |
| MiniMax M3 | liter-llm gateway — adversarial critic | unknown | no |
| GPT-5.6-x | Codex; liter-llm gateway | unknown | no |

**Nothing here has been measured.** "unknown" is the honest value for scaffold
need — no task set has been run per model under either profile. The `mixed`
profile is a default chosen because the fleet spans more than one model family,
not because a measurement supports it.

## Before switching to lean

Do not adopt `lean` because a model is reported to be capable. Adopt it when
this repo's task set says so.

1. Fix ~10 representative tasks for this repo.
2. Run them under `mixed`, per model. Record pass rate and token cost.
3. Run them under `lean`, per model. Record the same.
4. Adopt `lean` only if no model regressed.

Pass rate is the gate. Token cost is the tiebreaker. A configuration that costs
less and passes less is a regression carrying an efficiency argument.

## Results

<!-- date | model | profile | pass rate | tokens | decision -->
