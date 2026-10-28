# Corrective repair 001 — operator decision pending

Phase Execute: uar-post-lifecycle-verification-readiness. Canonical revision340 / plan109, current update-parking-lot-admission/5.1. The signed adopted contract remains d00c3c5bd3057ad70ce7324efb6d4476066271081f87833ed811f2929e521551. The initial serial packet stopped at its first failure. No repairs or retries have run.

## Concrete proposed source change

Review repair-001.proposed.patch, SHA256 a781dc5c8fa6fd62d12378a1ead0947cdc2a6e8a6ce874a6a3509ccb744bec8a. Before/proposed file hashes are retained in repair-001.proposal.json. It changes exactly:

- tests/support/post_lifecycle_store.rs: reuse existing parent hash helper for three digest strings; remove unused digest import; match public DurableStoreEligibility variants instead of calling the private method.
- tests/post_lifecycle_main.rs: match the same public variants at its durability check. Reject unsupported/unattested stores and retain cleanup/error behavior.
- tools/seed-minimax-verification/main.rs: clone record_id into two owned query bindings so its existing diagnostic/readback uses remain valid.

No acceptance scenario, production API, dependency, fixture effect, timeout or supported profile changes. The patch addresses the seven observed diagnostics; source inspection cannot establish compilation or runtime success. One fresh source-only teammate independently reached this repair scope; this is planning evidence, not phase acceptance review.

## Decision requested now

Approve applying this exact patch as repair1 of the maximum2, and authoring the resulting whole-phase inventory and refreshed authority packet. Approval here does not execute verification or authorize a public tag/image move, push, commit hooks or publication.

After application, the candidate must be identified honestly as published source774469746fee4de99be41dc1cd231857067e4487 plus the explicit repair, rather than byte-identical published source. The main runtime source is untouched; the seed utility and acceptance fixture are changed. Existing publication identities and all failed receipts remain preserved.

## Required before a corrective invocation

1. Record writer handoff, refreshed whole-phase implementation/input inventory and exact repaired source identity. Keep canonical5.1/5.2 pending. No product checks while authoring.
2. Preserve initial failed evidence/main-boundary-v1.3.0-001. A new append-only evidence destination ending002 is needed for the same main runner; do not overwrite or remove001. Updating the contract's exact launcher destination changes its digest and requires an actual new signed amendment/adoption, presented after the final inventory is frozen.
3. Reconcile complete call/time consumption including post-failure bookkeeping, source planning and teammates. tool-call-audit.initial-segment.json recovers57 emitted host calls plus72 source-counted nested call sites for the initial segment; it is not yet a complete closure accounting record. Preserve the original epoch and limits; do not reset UNKNOWN history or floors. Do not silently decide whether to double-count orchestration and nested invocations.
4. Present exact renewed fixture up/main/issue426/stop/down, unchanged profiles/reviews/spec validations and conditional completion commands against the repaired identity. Obtain explicit instruction for the failed/invalidated gate and subsequent pending packet. No extra preflight or automated review is implied. First failure stops the corrective invocation too.

The uncomfortable fact: the binary built, but the declared integration command also compiled a seed utility and failed before runtime. Repairing those compile errors does not establish any of the ten scenarios, ingestion behavior, six supported artifacts or independent phase review. None of those acceptance requirements is removed.
