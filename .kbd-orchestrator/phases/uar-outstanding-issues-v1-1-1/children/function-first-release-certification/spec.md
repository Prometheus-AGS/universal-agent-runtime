# Spec — function-first v1.1.1 release certification

Status: specified, unimplemented. Parent phase:
`uar-outstanding-issues-v1-1-1`. Existing OpenSpec change:
`openspec/changes/uar-outstanding-issues-v1-1-1`.

This child amends the existing release verification contract. It does not add a
second runtime feature change or a competing acceptance runner. The operator
verification lock remains **CLOSED** until all parent and child implementation
is complete, inventoried with exact identity and separately authorized by the
operator for named commands.

## Required public function

1. A user creates an enabled skill through the shipped skills API and submits
   a unique prompt with that skill explicitly attached through the shipped run
   API. The provider peer receives both markers. Its exact expected response is
   a deterministic function of both the received prompt and skill marker; a
   second distinct prompt produces a different response, and changing the skill
   marker would also change it. Public stream, terminal event, persisted run
   state and output agree with that exact response. A disabled but eligible skill reports
   the actual activation failure, contributes no overlay or unique dependency,
   and does not widen tool policy; ordinary prompt execution may continue.
2. A qualified budgeted run succeeds and settles durable accounting. An
   unattested store and the confirmed 16,000-token catalog reservation refuse
   safely before provider/tool effects, with terminal-only safe history. These
   observations continue to certify the #385 product repair, not customer data
   recovery.
3. Provider stream failures retain truthful terminal status. Text without
   terminal, pending tool EOF, malformed known event, overflow, invalid UTF-8
   and post-terminal data fail safely; a valid unknown future event is tolerated.
   A valid tool loop, cancellation cleanup and approved graph delegation remain
   observable.
4. MCP application failure, explicit success, omitted status, protocol failure,
   timeout, uncertain effect and a failed-tool/incomplete-stream combination
   retain typed immutable receipts and never imply unobserved success or replay.
5. Verified credential identity, source grants, direct/fallback/team/graph/
   summary/SDK capture, three-backend migration authority, immutable receipts,
   recovery after lost response and owner isolation remain observable. Preview
   and status assertions may be folded into the same backend journey but not
   removed. #387 compiler authoring remains denied for two admitted remote
   principals and local shared profile on both aliases and all operations,
   with spoof/body controls, unchanged storage and local single-owner control.
6. Unattested upload refuses before acceptance. Qualified upload, owner-isolated
   retrieval, legacy isolation, joined reclaim and qualified supervisor reclaim
   survive real restart on the advertised stores. Missing qualified supervisor
   proof blocks certification.

## Required release program

- Local candidate: all six reduced groups, plus the existing four native host
  supplements. Each required scenario has a real public/host entry point,
  negative control, durable/public observation and retained sanitized receipt.
- Published arm64 child: `streams` and `recovery` on the immutable pulled child.
- Published amd64 child: the same two groups on its distinct immutable child.
- Expected labels: local 65 (binding 8, streams 6, MCP 4, common and migration
  credentials 39, compiler 3, recovery 4, combined 1); each published child 10
  (streams 6 and recovery 4). No omitted label is represented as passed.
- Runner rejects any other non-repair group selection, wrong source/image/
  platform identity, missing scenario, false pass, changed fixture or unusable
  receipt. One named failed-group repair requires a valid failed parent receipt.

The published reduction withdraws direct per-child evidence for budget refusal,
compiler profile, credential migration and MCP failure handling. Those remain
local candidate release blockers; the release notes and receipts must state
this limit. Source-only tests do not become image evidence. Existing production
protocol/security/durability requirements remain unchanged; only the release
sampling program changes.

## Timing and claim boundary

The installed prompt/skill journey records client timestamps before POST, at
admission, first parsed content delta and terminal-status observation, and peer
timestamps at receive, first write and last write. Durations are compared only
within a clock domain. Client/network/poll uncertainty is disclosed. Timing is
diagnostic until a measured baseline or explicit operator threshold exists; a
material slow observation needs an operator disposition before release. Without
an approved numeric threshold, the evidence reports timings and uncertainty but
does not automatically pass or fail speed. This
certificate proves UAR propagation of an explicit user-selected skill and
controlled response. It does not certify model-selected skill quality,
external-model reasoning quality or external-provider latency.

## Implementation and verification boundary

The existing runner, adapters, fixture, manifest schema/example, OpenSpec
verification/proposal/design/deltas, parent plan/contracts and release docs
must be revised together. Every old scenario ID is mapped to a retained ID,
folded assertion or explicitly narrowed published claim in child
`analysis.md`. The exact expected candidate inventory is 65 labels. The
new source and fixture identity invalidates L11. A new whole-phase inventory
must reconcile the complete parent and child ledgers and writer handoffs before
any product review, check, test or build. The operator then decides the exact
commands. No current behavior, timing or release acceptance is claimed by
writing this spec.
