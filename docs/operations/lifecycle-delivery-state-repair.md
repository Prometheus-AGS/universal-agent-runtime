# Lifecycle delivery state repair

## Scope

This source-only repair responds to the authorized ARM64 Docker delivery
receipt in `lifecycle-delivery-build-failure-39523808909.md`. It covers only
the reported memory live-stream, realtime bus, PostgreSQL context import, and
retained scoped-persistence ownership failures.

## Repairs

- `MemoryLiveStream` now owns the pinned SDK's actual
  `Stream<Vec<surrealdb::types::Value>>` selected-query handle. Its stream item
  remains the SDK's single-value notification, while its explicit and retained
  close paths still hold the scope lease until close acknowledgement.
- The memory expiry call reaches the existing private controls backend through
  a narrow `memory::controls` sibling façade. Builds without `surreal-backend`
  retain the established explicit unsupported result; no consent or expiry
  fallback was added.
- `LiveQueryBus` uses an internal subscription enum. Raw persistence streams
  close through the SDK acknowledgement; memory streams close through
  `MemoryLiveStream::close`, preserving its lease and retained cancellation
  cleanup. Both variants yield the same pinned SDK notification item.
- `PostgresNotifyBus` imports `anyhow::Context` for its existing shutdown
  error context.
- Scoped persistence makes owned copies before retained dispatch. An
  `IngestionPublication` is a prepared, validated payload and can be cloned
  for an admitted worker. `VerifiedSupervisorExclusion` remains sealed: its
  fields stay private, it has no deserialization or public constructor, and a
  copy contains the same immutable verified challenge and observation only.
  The provider still validates the target and atomically consumes the durable
  challenge generation, so a retained worker or caller retry cannot create or
  replay authority by copying the proof.

## Interface effects

`MemoryLiveStream` and the private realtime stream adapter now use the SDK's
selected-query stream type. `IngestionPublication` and
`VerifiedSupervisorExclusion` implement `Clone` solely so the existing
`ScopedPersistence::dispatch` retained-task boundary can own inputs that were
already accepted by reference. No public construction, field visibility, or
validation interface changed.

## Verification status

The verification gate remains closed. No Cargo command, compiler probe, test,
formatter, lint, typecheck, policy validator, or review gate ran after these
edits.

The repair is source evidence only. The next authorized delivery build can
establish whether the corrected SDK item types and retained-input boundaries
compile. Actual stream closure and scoped worker completion require separately
authorized runtime acceptance. A successful image build cannot certify those
contracts or the separate process-restart, settings or embedding requirements.
