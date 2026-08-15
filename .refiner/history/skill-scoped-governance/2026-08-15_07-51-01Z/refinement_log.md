# Refinement log — `skill-scoped-governance`

## Iteration 1 — 2026-08-15T07:31:55Z

### Actions Taken

- Evaluated the completed B4 implementation against its OpenSpec delta and amended phase execution contract.
- Replayed the scoped precedence group, filesystem/API proofs, and real-run embedded assertion under `server-full`.
- Evaluated the observed-failing registration-merge and live-disable controls and their exact source-restoration hashes.
- Ran the final Tier 0 commands, strict OpenSpec validation, and scoped diff inspection.

### Constraint Status

- `b4-scoped-precedence-live-run`: satisfied — both service directions and the gated real-run widening/binding sequence passed.
- `b4-durable-merge-controls`: satisfied — Surreal restart preserved separate global and per-agent disables, and both inversions failed before exact restoration.
- `b4-origin-delete-storage`: satisfied — origin serialization, origin-based deletion behavior, and filesystem round-trip passed.
- `b4-tier-scope-spec`: satisfied — Tier 0 and strict validation passed; no B4-owned Clippy warning appeared; Tier 2 was not run.

### Reflection Summary

- Convergence: terminate.
- Reason: all four blocking constraints have observed evidence and no B4 stop condition fired.
- Uncomfortable result: the original contract amendment allowed only passing the conversation ID, but the existing policy universe discarded globally disabled skills before a narrower enable could be considered. The operator approved the additional one-line universe correction; without it the service tests would have overstated real-run behavior.

### Files Modified

- `.refiner/artifacts/skill-scoped-governance/artifact_manifest.json`
- `.refiner/artifacts/skill-scoped-governance/constraints.json`
- `.refiner/artifacts/skill-scoped-governance/decisions.md`
- `.refiner/artifacts/skill-scoped-governance/dist/verification-summary.md`
- `.refiner/artifacts/skill-scoped-governance/state.json`

### Content Type

- Type: `direct:content`
- Evaluation: `output_inspection`

## Iteration 2 — 2026-08-15T07:49:56Z

### Adversarial findings

- The first restart test reused one live `SurrealDbProvider`; it did not cross a cold-restart boundary.
- The durable agent model dropped unknown IDs from the existing binding API while returning success.
- User deletion was only proven in memory.
- Positive commands and literal output were missing from the replayable artifact.

### Corrections and observed results

- Replaced the acceptance proof with seed, reopen-delete, and verify-deleted child processes against one SurrealKV and filesystem location.
- Restored the legacy binding view for unloaded IDs while persisting agent overrides on loaded skills; both existing replacement tests passed.
- Observed API-created user removal from SurrealKV and filesystem, builtin refusal, and absence after a second reopen.
- Re-ran the merge inversion against the cold-restart test: exit 101; restored registry SHA-256 `cd81693b96bb3c1f1dfdfa6362aedbacafaa748359dc2c276d261a1b6d65547c`; positive exit 0.
- Added literal focused positive receipts and reran Tier 0 and strict OpenSpec validation.

### Constraint status

- `b4-scoped-precedence-live-run`: satisfied.
- `b4-durable-merge-controls`: satisfied on a cold-process boundary.
- `b4-origin-delete-storage`: satisfied with durable deletion and compatibility proof.
- `b4-tier-scope-spec`: satisfied; Tier 2 remains deferred.

### Content Type

- Type: `direct:content`
- Evaluation: `output_inspection`
