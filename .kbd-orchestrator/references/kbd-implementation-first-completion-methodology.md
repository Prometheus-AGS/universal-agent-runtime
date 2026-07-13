# KBD implementation-first completion methodology

Use this mode for a KBD completion phase when the product is known to be incomplete and repeated partial validation is slowing delivery.

1. Audit all remaining requirements statically and classify each as implementation, integration, evidence, or time-bound.
2. Batch and parallelize every independent implementation/integration gap. Audit existing code before writing replacements.
3. During implementation, avoid tests, Clippy, release builds, pushes, tags, and CI. Use `cargo check` only at cohesive checkpoints.
4. Start consolidated validation only when no known product requirement remains incomplete.
5. Run the full validation suite once, repair defects as one batch, then execute one immutable certification/release sequence.
6. Evidence and time-bound requirements do not masquerade as missing product code and do not block useful implementation.
7. Estimates assume a capable coding-agent harness using a current frontier coding model; report active agent-hours and identify irreducible external waiting separately.
8. Preserve active build caches. Use disk-space-guardian dry-runs and reversible cleanup; never reflexively run `cargo clean`.

This methodology does not waive correctness or final testing. It moves verification to the point where it can validate a complete product rather than repeatedly validating known incompleteness.
