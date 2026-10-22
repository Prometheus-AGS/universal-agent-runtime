# Construction-bound generic chat adapter controls

Phase: Execute. This teammate performed source inspection and authorized writing
only. No tests, checks, builds, probes, network calls, or binary execution ran.

## Observed cause

Gate004 `candidate.log:219` refuses explicit `parallel_tool_calls` as unsupported
by the `gpt-4o-mini` Liter adapter. The configured registry identity was
`fixture/gpt-4o-mini` with an explicit local chat base URL. Registry resolution
deliberately preserves the wire model as bare `gpt-4o-mini` and attaches
`resolved_provider_id=fixture`. The orchestrator uses that provider metadata for
its selection/logging, but `LiterLlmDriver::new` received only the bare model.
The capability guard then mistook the bare wire model for an adapter name.

Pinned Liter source proves the actual adapter: `client/mod.rs:1209–1249`
constructs `OpenAiCompatibleProvider` for an explicit base URL unless one of
its native special cases matches. Its adapter name is `custom`. Its default
`Provider::transform_request` is a no-op (`provider/mod.rs:457–460`). The generic
adapter implementation does not override that transform (`provider/mod.rs:743–782`).
`DefaultClient::prepare_request` serializes the typed request, updates model and
stream, applies that transform, merges extras, then serializes the final body
(`client/mod.rs:1070–1136`). The typed `parallel_tool_calls`, `tool_choice`, and
strict tool schema therefore survive this actual adapter path.

This is adapter serialization support, not a claim that every remote custom
endpoint accepts every feature. Endpoint/provider errors remain errors.

## Source changes

Only `src/llm/liter_driver.rs` changed in this cohort:

- Capture `generic_chat_adapter` at client construction, independently of the
  wire model and arbitrary registry IDs.
- Centralize its pinned selection mirror in `uses_generic_chat_adapter`.
  It requires an explicit base URL, excludes the exact native Azure, Anthropic,
  Gemini/Google AI, Vertex, and Bedrock branches, and excludes DashScope's exact
  `tongyi-embedding-vision-plus` match after stripping only `dashscope/`.
- Supply that captured fact to request preparation. The three existing control
  guards accept the generic adapter whose no-op transform preserves their
  semantics. Native unsupported paths retain their prior rejection.
- Preserve the captured adapter choice in inherited drivers. Prebuilt clients
  with unknown construction remain conservative; validated endpoint profiles
  retain their existing path. Budget-only helpers lack this construction proof
  and pass false. Wire model, budget enforcement, and error behavior remain.
- Author a regression for explicit-endpoint bare-model typed control/schema
  preservation, native parallel-control refusal, and native branch exclusions.
  Adjust existing private helper call sites for the captured argument.

No orchestrator file changed. Passing the registry ID `fixture` through the
constructor alone would not prove adapter semantics, so no arbitrary provider
ID was promoted into a recognized capability.

## Remaining limits

The regression is authored but unexecuted. Compilation, final HTTP wire, real
custom endpoint compatibility, and the full client-tools matrix remain
unverified. Lead owns the next explicitly authorized real gate and independent
review. This source mirror must be reconciled if pinned Liter construction
branches change; it intentionally does not guess capabilities from model
families. No unrequested guard or unrelated production edit was added.
