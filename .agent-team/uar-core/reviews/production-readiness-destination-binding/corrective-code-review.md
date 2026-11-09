# Corrective trusted destination binding source review

Scope: one corrective source-only review of the new destination-binding task, limited to the prior request-credential finding and its narrowly necessary private final-admission integration in `manager.rs`, `turn/bindings.rs`, and `llm/router.rs`. Current waypoint: Execute, `RUN_LOCAL_REPAIR_COMPLETE_CORRECTIVE_SOURCE_REVIEW_PENDING`. Producer handoffs, history, self-assessments and earlier whole-phase reviews were not read.

## Findings

None remaining in this scoped correction. The original MEDIUM request-credential fallback finding is resolved in source.

- `manager.rs:6817` now requires `run_credentials.is_none()` for registry-only fallback resolution. Request-backed fallback aliases reach `config_for` and the host credential decision instead of being pruned by global registry absence. Missing request-provider bindings retain the existing explicit refusal. The full resulting config/decision pair is retained and consumed by `RunModelBindings::capture` without reconstructing it from the primary.
- `turn/bindings.rs:466` derives the private plaintext exception set only from constructed primary/fallback bindings whose captured decision source is `ExplicitRequest`. The manager installs that set in both registry and without-registry admission constructors. No public grant/schema field or credential serialization was introduced.
- `router.rs:176` checks exact captured destination scope before considering any metadata exception. The exception matches only `RuntimeMetadataUnavailable`, exact private-set membership, and the absence of tools, reasoning, vision, structured-output, context and cost requirements. Provider/model disablement and capability failures remain errors. Registered-provider health is checked by ordinary evaluation; missing-provider aliases receive a fresh health check through `ensure_provider_not_in_cooldown` before the exception admits them.
- The existing without-registry admission branch is behaviorally unchanged. The correction does not authorize global credentials, reroute destinations, or re-resolve inherited credentials.

The uncomfortable limit: source inspection cannot establish that the corrected alias reaches the intended endpoint through the mounted production path. The original falsifier remains pending: a permitted pre-output primary failure with a valid request-backed fallback alias absent from the registry must reach that captured fallback; a missing request credential must still refuse. Final-send negative controls should retain zero calls for out-of-scope identities, known disablement/cooldown and unsupported hard demands. These are pending gate scenarios, not execution evidence from this review.

## Commands and limits

Commands were source inspection only: scoped `git diff`, `rg`, `cat`, and `sed`, all with explicit cwd `/Users/gqadonis/.claude/worktrees/production-readiness-476`. The only write was this corrective review artifact. No tests, Cargo/compiler/import probes, lint/format checks, builds, packaging, network, service or acceptance commands ran. Runtime behavior, compilation and selected-container delivery remain unverified; the lead retains the serialized gate.

This is an independently tasked source review without producer narrative. The reviewer remains the same model family as the team; it is not cross-family isolation.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE for this scoped source correction. This is not runtime or phase acceptance.
