# E1 context final-wire handoff

Status: source implementation complete; verification is CLOSED. No command that
tests, builds, formats, typechecks, lints, or otherwise verifies source was run
in this handoff.

## Observed source cause

The observed `context_history_integrity` failure at
`exact_budget_contract_dispatches_intact_protected_wire_body` originated in
`src/llm/liter_driver.rs::LiterLlmDriver::stream`. UAR compared its own
preflight JSON, whose `model` was the trusted qualified destination
`openai/uar-exact-cl100k-json-v1`, with bytes from retained Liter commit
`0d74c56659943e3fa44a3c8e6d51a27514f3b35c`.

That comparison was invalid. Liter
`crates/liter-llm/src/client/mod.rs::DefaultClient::prepare_request` resolves
the provider and calls `strip_model_prefix` before it serializes a streaming
body. Its explicit-base-URL generic adapter uses the model hint provider;
`crates/liter-llm/src/provider/mod.rs::OpenAiCompatibleProvider::strip_model_prefix`
therefore sends the literal model `uar-exact-cl100k-json-v1`. The routing
identity is not a literal provider-body model value.

## Changed paths and interface effect

- `src/llm/liter_driver.rs`
  - Renamed the private hand-built `PreparedChatRequest` bytes to
    `preflight_wire`; they are no longer represented as final validated bytes.
  - `LiterLlmDriver::stream` now obtains Liter's prepared streaming bytes,
    validates their final `stream` flag, selected output-ceiling field, exact
    count, and budget fit immediately before `chat_stream`, then emits its
    existing receipt from those same bytes.
  - `profiled_wire_receipt` now certifies Liter's prepared bytes rather than
    UAR's preflight serialization.
  - No existing provider ID, alias, capability certification, history,
    cancellation, tool identity, or vendored Liter source changed. E1b adds
    only the doc-hidden return type described below for the public driver trait.
- `tests/context_history_integrity.rs` is owned by the fixture writer, not this
  writer. Its E1 assertion now expects the retained adapter's literal
  `uar-exact-cl100k-json-v1` model while retaining the qualified synthetic
  destination contract and all tool/result/metadata/stream/count assertions.

Production defect versus fixture: the early equality guard was a production
defect because it compared different representations. The fixture's assertion
of the qualified model in the actual body was stale after retaining Liter's
documented prefix transformation. Neither correction adds a model-name
exception, compatibility allowance, counting-contract injection, or test skip.

## E1b durable reservation integration

`LlmDriver::prepared_final_wire_budget` is the exact named, object-safe seam.
Its doc-hidden public return type is nameable by the public trait but has
crate-private fields, so external callers cannot mint the evidence. Its default
rejects a contracted request. `LiterLlmDriver` alone implements it by using the
captured `DefaultClient::chat_stream_request_body` path and the same final-body
validation as dispatch.

`BudgetedModelDriver` calls that method only when both a durable budget and a
request contract exist, after output-cap narrowing and before durable call
admission. `ModelCallBudget::begin_call` uses the returned actual serialized
input count and effective output ceiling, then adds only the existing contract
uncertainty and reasoning reserves with checked arithmetic. Durable provenance
is now `liter_prepared_final_wire_contract`; the old static
`request_budget_bound` helper was removed after source inventory confirmed it
had no callers. `preflight_budgeted_request` remains only as the existing
auxiliary admission gate; it is explicitly not durable final-wire evidence.
No-contract and non-durable paths retain their existing behavior.

The capability rejects image-bearing requests for durable final-wire
reservation: JSON serialization proves neither real-image billing nor finite
image capacity. Existing finite-image refusal remains the governing path; no
synthetic image capacity or model-name exception was introduced.

Post-transform validation now requires the final literal model to be present,
the complete actual JSON controls to equal the captured typed request after its
literal-model transform, the selected output field and stream flag to remain
exact, and the final counted bytes to fit. For endpoint profiles it derives the
complete expected body through `profiled_wire_body`, which also preserves the
captured literal wire model and allowed schema. The pinned Liter helper and
`chat_stream` both prepare from the same unchanged typed request, but Liter does
not expose a dispatch operation accepting the prepared object; runtime equality
is therefore unverified until the phase-exit packet runs.

Authored, unrun assertion:
`src/llm/liter_driver.rs::prompt_caching_tests::prepared_final_wire_budget_uses_liter_bytes_and_refuses_image_liability`.
It independently prepares Liter's stream body, compares the returned bound with
the actual serialized count and effective output ceiling, asserts the literal
wire model, and refuses image-bearing durable-bound input. The existing context
fixture continues to cover captured protected tool/result body preservation.

## Source state and unverified work

The provider writer changed `src/llm/liter_driver.rs`, `src/llm/mod.rs`, and
this handoff. The runtime writer changed `src/uar/runtime/cost_budget.rs` for
the E1b consumer and `src/uar/runtime/thread/service_tests.rs` so its
HoldingDriver fixture obtains an explicit bound from captured Liter
preparation, rather than retiring or skipping that coverage. Fixture-writer
changes visible in the shared checkout remain their responsibility. E1/E1b
source work is unverified: the closed gate prohibits the affected context
target, Cargo checks, formatting, lint, typechecking, compiler probes, builds,
and review-driven verification.

The CLOSED state is the unfinished whole-phase implementation boundary, not a
new routine-approval requirement. The lead must first renew the complete
whole-phase inventory with E2--E5 and every writer handoff, then run the
already-authorized bounded phase-exit packet. Publishing, signing, deployment,
paid, and other external-effect boundaries remain separate.
