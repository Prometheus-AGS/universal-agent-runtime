# Design

## Context

Issue #379 items 1, 2 and 9 target the A2A boundary. The child architecture analysis locates production `A2AThreadService` construction in `src/server.rs` and actor dispatch in `src/uar/api/a2a/thread_service.rs`. Its task bindings are process-local; the current card route, wire fields and hardcoded version need adaptation. Source analysis is not runtime certification. The existing `a2a-grpc` spec still names a 0.3 gRPC binding, so conformance claims must identify the selected public 1.0 interface rather than silently relabel that binding.

## Goals / Non-Goals

Deliver a usable anonymous 1.0 card, a conformant authenticated 1.0 task surface with durable task correlation, and a card version equal to the actual published release version. Maintain complete verified owner scope, existing host admission and actor execution. A second executor, independent task authority, invented release tag and generic unauthenticated A2A access are outside scope.

## Decisions

1. The implementation task selects and records authoritative A2A 1.0 schema sources and exact method/part/role/state/error names before editing wire types. The schema fixture is authored now but not executed before the full Dot implementation boundary. Existing 0.3 clients receive an explicitly documented compatibility disposition; only a conformant interface is advertised in the 1.0 card.
2. Map public task IDs to the existing durable owner-bound thread. Define who owns each binding and which atomic transition admits competing submissions and post-restart lookups. A local mutex/map alone is not the durable source. Reconstruct a missing process-local handle through current host admission and protected durable records; preserve failure and cancellation semantics. A2A cannot grant its caller authority to resume an unrelated thread.
3. Serve `/.well-known/agent-card.json` without credentials and keep any `/agent.json` alias equally narrow. All task methods remain under current authentication and authorization. The card advertises a reachable interface URL and the exact selected 1.0 shapes. An auth-required task state or protocol error follows the pinned 1.0 contract and does not expose private task data.
4. Produce the top-level plain `x.y.z` from the package/release source associated with the published artifact. Build/package provenance must establish equality to the actual tag; this design selects no tag. Do not substitute a hardcoded literal or a branch version for installed-artifact proof.

## Risks / Trade-offs

The uncomfortable thing: a valid JSON card from a branch build is still insufficient for Kubo. Task correlation can appear correct in one process while a restart or competing runtime loses the mapping. Final acceptance therefore uses the published installed artifact in local and shared profiles. Current source does not establish a binding race; the implementation task must define ownership/atomic transitions before changing synchronization.

## Migration Plan

Preserve legacy task/card clients only through explicitly named aliases or versioned bindings. Keep the old gRPC contract until an authoritative 1.0 gRPC binding is selected and implemented; do not advertise it as 1.0 merely because HTTP changed. The `A2AThreadService` actor, durable thread and host admission remain authoritative. The lead schedules exclusive A2A ownership and a serial server route handoff after dot-03 and dot-06 production contracts; dot-19 uses this protocol contract. No migration may infer owner identity from an old task ID alone.

## Verification

Author schema and restart/concurrency scenarios during implementation. Execute none until every combined Dot production item, required document and runner is complete and the lead records the implementation inventory. Then exercise public card access, private method denial, 1.0 message/get/cancel and task states, cross-process restart/correlation, insufficient scope and error mapping through real host/actor/persistence paths. Final dot-17 checks the top-level version against the actual published tag and digest/checksum from installed artifacts under both profiles. Historical 01–06 receipts do not certify this combined revision.
