# Classification — second Ping boundary

The single new attempt crossed the first empty-Ping boundary: the collector
completed one masked empty-Pong write. The next observed header was another
empty Ping. The collector stopped as required, before reading any payload or
receiving a sign-in result. This is a bounded diagnostic outcome, not successful
authentication or a readiness diagnosis.

## Evidence

`evidence/signin-observation.json` records both headers as final, RSV-clear,
server-unmasked, minimally encoded 7-bit zero-length Ping (opcode 9).
It records one upgrade, one sign-in request, one Pong write attempted/completed,
two headers, zero sign-in responses and zero payload bytes intentionally read
or retained. No query, health/readiness request or retry was made by this script.
The source digest in the receipt matches the collection-time collector.
The receipt hash remains `969d730640fb8c8e1ddede26bdc3b1236b22b3719d9e138fa67ec80011a515b8`.

| Boundary | Recorded outcome |
|---|---|
| Continuity | All inherited source, executable, process, listener and configuration comparisons passed before connecting. |
| Handshake | HTTP 101, json subprotocol, no negotiated extensions. |
| First frame | Exact allowed zero-length Ping. |
| Pong | One complete local socket-write callback; remote receipt/processing is not proven. |
| Second frame | Another zero-length Ping; unsupported by the one-Ping plan. |
| Deadline | 6,845.883 ms exchange; 15,853.493 ms active total, below 15,000/30,000 ms caps. |
| Privacy | 3,906-byte receipt; no application content retained; no Text parsed. |

No RPC identifier or result could be validated because neither observed frame
was Text. That comparison is unavailable, not passed. The receipt does not prove
whether a response would arrive after another Pong, whether the peer processed
our Pong, or why authentication was delayed. It also cannot reconstruct the
discarded historical frame or establish the cause of persistent readiness delay.

## Next evidence

The observed blocker is now repeated empty Ping during sign-in. A subsequent
plan can consider a finite repeated-Ping allowance under the same non-resetting
deadline, with an explicit frame cap, before one new authorized attempt. This
change does not implement that extension or repeat the consumed attempt.
Avoid another unbounded sequence of one-frame exceptions: the next design should
state its entire finite control-frame budget and terminal behavior upfront.

## Goal disposition before acceptance

1. MET: the assessed exact-frame behavior is implemented; synthetic verification
   and independent acceptance remain Task 4.
2. PARTIAL: one Pong was written and a bounded following-frame wait occurred;
   a second Ping prevented a sign-in result.
3. PARTIAL: content-free evidence and the stop boundary are recorded;
   independent acceptance is pending. No readiness-recovery claim is supported.
