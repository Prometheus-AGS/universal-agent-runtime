# Fail-Closed Rollback Plan

## Purpose

The rollback build retains the governance status endpoint and persisted settings
schema while forcing effective governance On. Governance mutation is unavailable
in that build, so an older operator surface cannot present a writable Off state.
The rollback build never interprets `governance.enabled=false` as permission to
bypass tool governance.

## Reproducible source

- Forward implementation commit: `9c251e16161e778d99bd8c30da2e69c15b7070cb`
- Rollback branch: `codex/governance-rollback`
- Rollback implementation commit: `ec21b0aba68a048c0ac51a5ecf56eb0d5730e870`
- Supported downgrade target: the fail-closed rollback commit above
- Forward release-candidate commit and digest: `PENDING AFTER REVIEW CORRECTION`
- Rollback release-candidate commit and digest: `PENDING AFTER REVIEW CORRECTION`

The rollback commit is derived from the forward source commit and changes only
governance bootstrap finalization: it initializes the runtime to On and marks
governance mutation unavailable after settings schema initialization.

The final candidate commits may add only evidence or release metadata after
these implementation commits. Any production-code delta invalidates the listed
candidate and requires both candidates to be rebuilt and reverified.

## Required release order

1. Finish all forward and rollback source changes.
2. Commit immutable forward and rollback candidates.
3. Build and verify the rollback candidate before installing the forward
   candidate.
4. Complete the forward Tier 2 and Tier 3 gates and record both binary digests.
5. Export the persisted row and retain the prior installed binary before the
   forward replacement.
6. Install only the verified forward candidate and prove the live matrix.

No earlier temporary artifact or already-installed binary substitutes for this
order. A source change after verification restarts the sequence.

## Compatibility matrix

| Backend | Operator UI | Expected result |
| --- | --- | --- |
| Forward | Forward | Authoritative On/Off status and serialized mutation |
| Rollback | Forward | Truthful On status; mutation unavailable; Off cannot be saved |
| Forward | Prior | Unknown governance fields remain additive; backend stays authoritative |
| Rollback | Prior | API-owned persisted Off is preserved while effective runtime stays fail-closed On; a seed-owned Off default can be normalized to On |

Observed receipts will be appended only after all implementation and artifacts
are complete and the operator reauthorizes the end-of-work verification phase.

## Reversible persisted-row procedure

Before installing the rollback binary:

1. Export the complete `governance.enabled` settings row, including its record
   identifier, typed value, and metadata, to the release evidence directory.
2. Stop the launch agent and retain the export alongside the rollback binary
   digest and source commit.
3. Remove only the exported `governance.enabled` row if the selected older
   downgrade target cannot tolerate unknown settings rows. Do not remove the
   governance namespace or any policy rows.
4. Install and start the rollback build, then confirm its authoritative status
   reports effective On and mutation unavailable.

The export receipt must identify the storage backend, source row identity,
export checksum, forward binary digest, rollback binary digest, and the exact
directory containing the recoverable artifacts. Secrets must be redacted from
human-readable evidence without altering the recoverable export.

To restore the forward build:

1. Stop the rollback process.
2. Reinstall the verified forward binary.
3. Compare the current row with the export. The supported rollback candidate
   preserves API-owned rows (identified by their durable `updated_at` marker)
   but can normalize a seed-owned Off default to On. Restore the exported typed
   value through the forward settings API only when the export was seed-owned
   and the current row is the known normalized On value. If the row has another
   unexpected concurrent change, preserve it and stop for operator resolution.
4. Start the forward process and confirm the restored value through the
   authoritative governance status endpoint before accepting tool traffic.

## Verification receipts

- Verification state: `COMPLETE — OPERATOR AUTHORIZED TIER 3`
- Rollback build and focused behavior: `PASS` — effective On,
  `mutation_available=false`, reason `persistence_unavailable`, and an Off
  mutation returned `validation_rejected` without changing status.
- Forward UI against rollback backend: `PASS` — the Governance route loaded,
  the truthful locked projection remained readable, and mutation was rejected.
- Prior UI/backend compatibility: `PASS` — the status API addition is additive;
  the new UI's missing-status test and Unknown/Refresh browser scenario cover a
  prior backend, while the rollback build retains the route and schema.
- Persisted-row downgrade behavior: `PASS WITH OWNERSHIP DISTINCTION` — the
  isolated live round trip began from a seed-owned default Off row, which the
  rollback candidate normalized to On. The focused API-owned-row regression
  preserves durable false while rollback enforces effective On, and forward
  restart then recovers effective Off without a restore write.
- Export/remove/restore exercise: `PASS` — the complete pre-install row was
  exported to
  `/Users/gqadonis/.prometheus/backups/uar/governance-release-20260828T.HtRDLE/governance.enabled.json`
  (SHA-256 `5154022f9c6b7a1891b742b51952d3167c6a98b6164320fadbcd6ff716873a3f`),
  and the isolated seed-owned round trip restored Off through the forward API
  at revision 11 after observing the known normalization.
- Rollback candidate built before forward installation: `PASS` — rollback
  `server-full` digest `f725a77f...5189` was built and retained in the backup
  directory before forward digest `0030737d...4bff` was installed.
