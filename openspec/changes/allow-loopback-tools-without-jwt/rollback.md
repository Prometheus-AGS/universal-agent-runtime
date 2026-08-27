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
- Forward release-candidate commit: `44fc519c7d65e0f125b812caf992121cf51c38ad`
- Rollback release-candidate commit: `ce712ee4a969d15d9c73533ae5be4266abdaea1f`
- Forward release-candidate digest: `PENDING`
- Rollback release-candidate digest: `PENDING`

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
| Rollback | Prior | Unknown persisted row is tolerated; runtime remains fail-closed On |

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
3. Restore the exported row only when its current database value is absent;
   otherwise preserve the current row and record the conflict for operator
   resolution.
4. Start the forward process and confirm the restored value through the
   authoritative governance status endpoint before accepting tool traffic.

## Verification receipts

- Verification state: `PAUSED BY OPERATOR — IMPLEMENTATION/ARTIFACTS FIRST`
- Rollback build and focused behavior: `PENDING`
- Forward UI against rollback backend: `PENDING`
- Prior UI/backend compatibility: `PENDING`
- Unknown-row downgrade behavior: `PENDING`
- Export/remove/restore exercise: `PENDING`
- Rollback candidate built before forward installation: `PENDING`
