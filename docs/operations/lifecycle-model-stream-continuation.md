# Model stream lifecycle continuation

## Scope and verification boundary

This source-only continuation belongs to KBD phase
`uar-outstanding-issues-v1-1-1::function-first-release-certification`, task
5.4. The verification gate is closed. No test, build, formatter, lint,
typecheck, compiler probe, review gate, or acceptance script ran for this work.

The pinned Liter checkout is at
`e446f5ef91003bbec5fce6b2f45fb94559dff335`, matching `versions.toml`.
Its source was inspected but not modified.

## Confirmed source trace

- `LlmDriver::stream` in `src/llm/mod.rs` returns an owned
  `ExternalDriverStream`. The trait carries neither a caller cancellation
  capability nor a provider completion acknowledgement.
- `open_driver_stream` in `src/llm/orchestrator.rs` applies the configured
  start budget to `driver.stream(request)` and the first-semantic idle budget
  to `stream.next()`. On either timeout, Rust drops the local future or stream
  while the retry/failover path may begin another attempt. Provider lifecycle
  and usage preludes remain inside that retry boundary.
- `LiterLlmDriver::stream` in `src/llm/liter_driver.rs` awaits
  `DefaultClient::chat_stream`, then retains the returned chunk stream inside
  its normalized `async_stream`. It creates no production Tokio task or join
  handle.
- The pinned Liter `DefaultClient::chat_stream` builds an HTTP request and
  returns a parser over `reqwest::Response::bytes_stream` in
  `vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs` and
  `http/chat_streaming.rs`. The inspected production path creates no detached
  task. Liter contains a cancellation-capable parser helper, but the selected
  `DefaultClient::chat_stream` path does not expose that helper's token through
  the UAR adapter.
- `AnthropicDriver::stream` in `src/llm/anthropic_driver.rs` likewise awaits
  its `reqwest` response and retains `response.bytes_stream` in the normalized
  stream. It creates no production Tokio task or join handle. The only
  `tokio::spawn` occurrences in either adapter are test fixtures.
- `BudgetedModelDriver::stream` in `src/uar/runtime/cost_budget.rs` admits a
  durable accounting call before provider dispatch. It records terminal usage
  only after a provider `Done` event with a confirmed usage report. Cancellation,
  a stream error, an idle timeout, or dropping the returned stream does not
  release that durable reservation.

## Resulting contract

There is no demonstrated unjoined local model-stream producer in the owned
adapter files. A dropped stream ends UAR's local read scope. The inspected
source provides no remote provider-completion acknowledgement, so that drop
cannot establish that the provider stopped, completed, or incurred no charge.
It also cannot justify a rollback or refund.

The existing durable accounting path preserves the conservative outcome: an
accepted call without terminal provider evidence remains pending. Retry and
fallback create another local attempt only after the timed-out local scope is
dropped; this source trace does not claim the prior remote attempt completed.

## Attempt receipt and budget limits

`Orchestrator::attempt_manifest` describes the selected destination and budget
profile, but `prepend_attempt_manifest` runs only after `open_driver_stream`
returns successfully. A start or first-semantic timeout therefore has no
emitted attempt-manifest event. The team handoff record is also written only
after that same successful return.

Durable accounting covers the safety property that matters for an unknown
effect: `CallAdmission` is persisted before dispatch and begins in `Pending`.
Its record contains a call identity, model string, scope bindings, and
reservations. It does not itself contain a resolved provider ID, a credential
binding identity, or `LiterLlmDriver::profiled_wire_receipt` evidence. The
`attempt` field is initialized to `1` for each newly admitted durable call, so
it is not a retry ordinal across opening attempts.

This is a provenance limit on the existing accounting record, not evidence
that a credential was resolved again or that a final-wire budget was violated.
The bound-driver construction and request preparation remain the authorities
for those contracts. A later shared receipt design must preserve their exact
provider and credential binding rather than deriving either from an alias.

## Required cross-owner decision if stronger semantics are needed

A host-owned attempt lifecycle contract would need to cross `LlmDriver`, the
orchestrator retry/failover owner, and durable accounting. It must identify an
attempt, distinguish local read abandonment from a confirmed provider terminal
outcome, retain finalization independently of a dropped waiter, and keep the
reservation pending when no provider acknowledgement exists. It must not
invent a remote cancellation acknowledgement from a dropped `reqwest` stream.

If that contract also needs a durable per-attempt receipt, it must capture the
already bound provider ID, a redacted credential-binding identity, and the
existing final-wire proof before dispatch. It must record an opening timeout as
an unknown-effect outcome, not a successful attempt or a refund.

Those shared interfaces are not owned by this provider lane. No adapter code
change is warranted from the inspected source alone.

## Uncomfortable limit

An upstream provider can receive a request and continue work after UAR stops
reading its response. The current source can retain liability for that unknown
effect, but cannot join or prove completion of work it neither spawned nor can
acknowledge remotely.
