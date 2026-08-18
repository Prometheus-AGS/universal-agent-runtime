# Convergence Decisions

### Iteration 1 Decision

All A0 blocking constraints pass with deterministic evidence. The first
provider-conflict control exposed an incorrect interpretation of
`OnceLock::set`: its error returns the attempted provider, not the installed
provider. The corrected wrapper records only a successful UAR-owned RustCrypto
installation and compares that recorded pointer on repeated calls; a prior
foreign or panic-provider initialization returns `ProviderConflict`.

The Android check initially exposed missing NDK/OpenSSL environment setup. A
temporary vendored Android OpenSSL sysroot allowed the unchanged full
`embedded-mobile` graph to pass, so no unrelated dependency repair entered A0.
Proceed to the KBD/OpenSpec completion handoff without running Tier 2.

### Iteration 1 Supersession

Independent review invalidated iteration 1's pointer-comparison rationale:
`install_default()` returns the attempted provider, not the installed one, and
the installed provider getter is private. The operator selected the smaller
safe contract: UAR owns first installation. Any process provider installed
before UAR—including RustCrypto—returns a structured conflict. Iteration 2
implements that boundary at the shared server-startup funnel and guarded JWT
operations.

### Iteration 2 Decision

The independent artifact-validator confirmed all four blocking constraints,
and the isolated artifact-critic returned PASS with no findings. Persist
iteration 2 and terminate refinement as converged. Tier 2 remains deferred to
completion of all six phase changes.

### Iteration 3 Decision

- **Decision**: terminate
- **Iteration**: 3 of 5
- **Blocking violations remaining**: 0
- **Rationale**: The later pointer-identity attempt was executed against an AWS-LC-first process and failed its conflict assertion. The plan, spec, code, and evidence now restore the already-proven first-owner contract; both prior-provider positives pass and the false AWS-LC-acceptance control fails.
- **Next focus**: Separate A0 follow-up commit. The final history-free critic and judge reported no concrete A0 blocker.

### Iteration 4 Decision

- **Decision**: terminate
- **Iteration**: 4 of 5
- **Blocking violations remaining**: 0
- **Rationale**: Deterministic validation passed 4/4 and the fresh critic and judge both cleared A0 after the verification, mobile-source-identity, and task-ledger corrections.
- **Next focus**: Commit the A0 follow-up separately from A1.
