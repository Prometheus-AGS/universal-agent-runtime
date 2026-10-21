# Ordered transport and subscription recovery

Date: 2026-10-20. Execute phase `uar-outstanding-issues-v1-1-1::function-first-release-certification`; authoritative waypoint read at revision 3144, plan revision 68. Verification is **CLOSED**. This is source implementation evidence, not a passing runtime result or a phase-complete inventory.

## Observed source failure

The server independently registered persistence transport and realtime subscription reconnect handlers, and independently registered memory transport and memory-realtime handlers. Concurrent authenticated control commands could run these handlers against one another. A transport reconnect acknowledgement could also leave every subscription unavailable: the SDK's Available event is observation, not permission for the bus to recreate registrations.

The static lifecycle resource generation in the old snapshot did not expose the SDK transport generation required by the transport command. Topic snapshots also cannot substitute for the bus command generation while individual subscription workers are changing generations.

## Implemented contract

`ReconnectPair` owns one async mutex for each transport/subscription pair. Both resource aliases enter the same controller through the existing registered `RuntimeHandle` operation, which survives cancellation of its response waiter.

For one explicit command, the controller:

1. Acquires the pair mutex and checks the caller's generation against the authoritative target: SDK generation for a transport alias, bus command generation for a subscription alias.
2. Reads the retained transport state. Available transports are reused. Unavailable transports receive exactly one explicit reconnect request. Released or unobservable transports fail without replacing the client.
3. Confirms that the SDK retained the acknowledged available generation.
4. Requests one new subscription generation and awaits `ready_generation` for every enrolled topic.
5. Confirms the transport generation is still available before acknowledging the pair.

There is no retry, backoff, polling, or elapsed-time success rule. A failure after transport recovery leaves that real partial state observable; the controller does not invent rollback or silently issue another recovery attempt. A later explicit command uses a fresh snapshot.

| Resource command | Expected and returned generation | Pair |
| --- | --- | --- |
| `persistence` | SDK transport generation | Surreal persistence and `realtime` |
| `realtime` | Bus command/subscription generation | Same persistence pair; PostgreSQL uses its listener bus without a separate transport callback |
| `memory` | SDK transport generation | Memory and `memory-realtime` |
| `memory-realtime` | Bus command/subscription generation | Same memory pair |

Surreal persistence transport observations also cover embedded clients. When an embedded transport is already available, an explicit persistence command only recreates the owned subscriptions. This does not introduce engine reopening after release. PostgreSQL retains its acknowledged listener/pool cleanup and bus reconnect path.

## Private control protocol delta

Protocol version and successful reconnect result remain unchanged:

```json
{"version":1,"request_id":"example","result":{"resource_id":"memory","generation":1,"outcome":"acknowledged"}}
```

Snapshot `result` retains `resources` and `subscriptions`, and adds:

- `transports`: a map keyed by transport resource ID. Each value has `generation: number | null` and `state: "available" | "unavailable" | "released" | "observation_unavailable"`. Numbers come from the SDK, not the host resource registration counter.
- `subscription_generations`: a map keyed by bus resource ID with its actual nonnegative command generation. Consumers use this field for subscription reconnect CAS even when topic snapshots are mixed or not yet updated.

A failed reconnect produces a correlated error response containing safe partial state:

```json
{"version":1,"request_id":"example","error":"lifecycle_reconnect_failed","details":{"resource_id":"memory","expected_generation":0,"transports":{"memory":{"generation":1,"state":"available"}},"subscription_generations":{"memory-realtime":1},"subscriptions":{}}}
```

The example's empty subscription map is illustrative only; production responses include the currently registered bus/topic snapshots. No endpoint, password, raw driver error, or database-backed authorization lookup is added to this launch-authenticated channel.

Snapshots are observations, not a globally atomic snapshot of independently transitioning resources. A command validates the authoritative generation again after obtaining its pair lock. A stale observation cannot bypass that comparison.

## Changed files and coordinated interfaces

- `src/server.rs`: replaces independent reconnect registrations with shared pair controllers; registers actual transport observations; preserves PostgreSQL bus-only recovery.
- `src/server/lifecycle_reconnect.rs`: new paired controller, transport observation adapter and serialized recovery contract.
- `src/server/lifecycle_control.rs`: adds transport/bus generation snapshots and structured partial-state error responses.
- `docs/operations/lifecycle-reconnect-continuation.md`: this handoff.

The state writer supplies `BusLifecycle::generation()` and memory transport observation accessors in its owned files. Those APIs are source dependencies, not independent verification evidence.

The lead owns the matching strict schema change in The Boss `UarSidecarService.ts`. This writer is restricted to UAR and did not edit the other repository. The schema must accept both new snapshot maps and the optional structured reconnect-error details before the consumer change is complete.

## Limits and required evidence

The uncomfortable case remains a permanently pending SDK reconnect or subscription-close operation. It retains the registered command and pair lock, exposes the outstanding host obligation, and can prevent finite shutdown. It must not become success because a deadline elapsed.

Remote side effects remain uncertain independently of local recovery. Successful reconnect does not resolve an earlier unknown transaction or replay an operation.

No tests, checks, builds, compiler probes, formatting, linting, policy validators, review gates, or acceptance scripts were executed. Only source inspection and authorized authoring occurred. Compilation, feature combinations, concurrent command ordering, partial transport recovery, subscription failure, canceled control waiter, and consumer compatibility remain unverified. The lead must include these cases and the strict consumer schema in the whole-phase completion inventory and exact proposed commands before requesting operator authorization.
