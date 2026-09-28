# C07: Local scoped observers with durable delivery

## Why

Agent Fabric Convergence C07 requires an observer to watch selected agents and conversations without depending on a live UI stream. Today C06 instance events are committed with their instance record but bounded, while ordinary run token streams are process-local. Neither is an independent durable observer inbox.

## What changes

- Publish committed C06 instance transition occurrences through a source-transaction outbox with stable occurrence and provenance IDs. The initial durable source profile is a logical agent instance and its stable conversation identity; ordinary unversioned session snapshots and token deltas are not advertised as durable sources.
- Persist observer subscriptions, source/conversation intersection, projection grant, revision, delivery cursor, inbox admission, acknowledgement, pause, backlog and retention-gap state in SurrealDB 3.3.0. A subscription is owned by the verified principal and workspace and targets an authorized observer instance.
- Admit occurrences as bounded, independent observer turns through C06's command path. Recheck current binding and grant before exposing queued projection or starting a turn. Replay means observe again with the same identity; it never repeats a source effect.
- Expose authenticated administration, capability discovery and operator-visible recovery diagnostics. Unsupported persistence providers refuse this durable profile.

## Scope and dependencies

This is the UAR product slice of initiative C07 and depends on the C02 authority boundary, C03 bindings and C06 durable instances. The Boss UI, Fabric adapter and cross-host delivery are later changes. The existing ordinary-run and AG-UI paths remain supported without implying durable observer semantics.

The uncomfortable constraint is that UAR's ordinary `Session` has no durable per-message revision and its save is not an atomic semantic-event commit. C07 does not label that path as a durable source. Adding it later requires a transactional session/outbox write, not a wrapper around its in-memory stream.
