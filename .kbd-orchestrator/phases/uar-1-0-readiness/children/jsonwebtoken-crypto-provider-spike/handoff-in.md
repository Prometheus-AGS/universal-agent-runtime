# Handoff in — uar-1-0-readiness / jsonwebtoken-crypto-provider-spike

**Spawned by:** `uar-1-0-readiness`

## Why this child was spawned

The A0 proposal selected RustCrypto from lockfile presence alone. Current execution evidence showed that `jsonwebtoken` 11 has no selected provider and panics at runtime. The operator requested a contained research spike so the phase standardizes on RustCrypto or AWS-LC from comparative evidence rather than preserving the proposal's preliminary choice.

## Inputs

- `.kbd-orchestrator/phases/uar-1-0-readiness/assessment.md`
- `.kbd-orchestrator/phases/uar-1-0-readiness/plan.md`
- `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`
- `openspec/changes/fix-jwt-crypto-provider/`
- `Cargo.toml` and `Cargo.lock`

## Success criteria

- Both built-in providers are compared against the `server-full` build and supported release targets.
- The recommendation distinguishes lockfile presence from activation in the build graph.
- One provider is selected, the alternative is explicitly rejected, and re-evaluation triggers are recorded.
- The parent receives an exact manifest change and executable verification sequence.

## Expected deliverables

- `assessment.md`
- `analysis.md`
- `plan.md`
- `execution.md`
- `decision.md`
- `reflection.md`
- `handoff-out.md`
