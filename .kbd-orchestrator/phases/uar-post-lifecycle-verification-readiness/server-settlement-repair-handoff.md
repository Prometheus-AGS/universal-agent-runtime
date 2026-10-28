# Server shared-ledger settlement repair handoff

Phase: `uar-post-lifecycle-verification-readiness`, task `2.9`.
Implementation source authoring only. Operator verification gate: **CLOSED**.
This handoff grants no execution, review, preparation, build, provider, network,
or acceptance authority. Source identity was not computed. Files are frozen for
lead integration after this handoff.

## Changed files

- `scripts/acceptance/packet_processes.py`: adds the actual packet owner's
  `_ledger_snapshot` and `settle` methods. Existing default `alive`/start/stop
  behavior remains. No call to settlement was added to the default first-failure
  runner.
- `scripts/acceptance/failure_sweep.py`: requires settlement before each attempt,
  including the first borrowed-owner attempt, and after confirmed exact-owned
  cleanup before correction/rerun or independent continuation. It appends a
  settlement-failure receipt referencing the original terminal receipt.
- `scripts/acceptance/failure_sweep.README.md`: documents settlement semantics,
  receipts, limits, known failures, saturation, and the unrun test boundary.
- `scripts/acceptance/tests/test_failure_sweep.py`: adds three regression cases
  for dispatch ordering, unknown-accounting safety stop preserving original
  failure/cleanup, and context-construction failure without false cleanup.
  The existing FakePacket now implements `settle(remaining)`; existing fake
  lifecycle tests otherwise retain their packet start/stop and shared-counter
  contract.
- `scripts/acceptance/tests/test_packet_settlement.py`: new source file with six
  tests for pending-to-settled draining, unknown refusal, known failed 401 with
  retained reservation, missing successful usage, exact attempt saturation, and
  bounded pending timeout. Approval parsing is mocked; ledger transaction and
  reservation checks use authored temporary SQLite fixtures.
- This handoff file.

## Semantics and limits

The observed blocker is the source-proven case where item/runtime cleanup
finishes while the shared forwarding proxy continues draining an already
dispatched upstream response. The new guard addresses that boundary alone.

`settle(remaining)` waits no longer than 90 seconds, further bounded by the
collection callback, packet Context wall/output checks, and the unchanged
absolute seed deadline. It polls at most every 0.25 seconds with remaining-time
clipping. SQLite lock waiting is clipped to the current remaining interval;
metadata and reservations are read inside one explicit read transaction on a
`mode=ro` connection, which is closed after each snapshot. Reservation reads
are limited to the approved cumulative model/embedding attempt cap plus one
sentinel row, with excess rejected.

Frozen proxy source and config hashes are checked using the existing fixture
references. Existing budget-proxy config/private-file validation rebinds approval,
seed, source references, and input identity. The original process-captured ready
receipt must remain identical on disk; config digest, epoch, input identity,
listeners, nonce, and PID must match. The packet-owned process must remain live.
The ledger's current owner must match that captured PID/nonce; root and ledger
must belong to the current user and remain private nonsymlink paths. The root
and ready path derive from the frozen configuration and captured nonce. This
uses the existing ownership semantics directly in the same snapshot rather than
adding a separate `owned_hop` connection with a different metadata snapshot.

Every retained reservation is counted from the original seed. Expected totals
and provider pools must equal stored metadata and remain within the existing
ceilings. Pending entries cause a bounded wait. Unknown or inconsistent entries,
blocked epochs, changed ownership/configuration, exceeded ceilings, exhausted
wall/output/request allowance, and unresolved timeout stop continuation. Known
terminal non-success HTTP responses, including controlled 401, are accepted
without a successful-usage envelope and retain their full reservation, matching
`budget_proxy`. Completed successes require actual model output usage within
the reservation, or actual embedding input/total usage. Only the proxy's already
settled successful response-byte accounting is reflected in reconstructed totals.
The settlement reader never refunds or writes accounting. Exactly reaching a
known ceiling still permits truthful settlement; existing `remaining()` and
proxy admission checks control later dispatch. No invented forecast of the next
binding was retained.

The original terminal receipt includes cleanup confirmation and is written
before post-cleanup settlement. Original failed-case receipts and summary failure
entries are preserved; settlement failure appends its own referencing receipt
and stops the branch. No correction slot is consumed after that failure. Context
construction failure retains `cleanup_confirmed: false` and stops conservatively.
Borrowed packet owners remain borrowed: no restart/stop, epoch change, counter
reset, refund, new provider request, model redispatch, or ledger deletion occurs.

## Verification and unresolved decisions

**All implementation and all nine newly authored tests remain UNVERIFIED.** No
test, import, compiler/syntax probe, lint, formatting, typecheck, build, packaging,
service/provider/network action, review, diff check, or acceptance command ran.
Only source-reading commands and text-writing patches were used. No checks were
delegated. No source identity was computed. No independent source review was
performed; the lead will request the replacement packet separately.

No unresolved implementation choice is claimed to require operator input. Runtime
behavior, syntax, SQLite contention/timing, actual provider usage, receipt I/O,
and full-path integration remain unverified. Tests using mocked approval parsing
cannot certify production approval/provenance behavior.

Lead retains ownership of whole-phase source inventory, canonical KBD/OpenSpec,
and repair accounting. Per task packet, bounded source repair17 is charged once
within cap28 and fixture repair18 is reserved for a future preparation entry;
this writer did not modify those counters or any configuration. Historical
consumption and the separate provider caps remain unchanged.
