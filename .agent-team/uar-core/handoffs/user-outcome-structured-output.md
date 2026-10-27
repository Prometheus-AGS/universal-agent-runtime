# Structured output implementation handoff

Phase: `uar-post-lifecycle-verification-readiness`, user-outcome implementation, task 2.8. Verification gate **CLOSED**. No tests, builds, compiler probes, formatters, lint, validators, services, model calls, or other verification commands ran. Commands were source inspection and implementation/documentation writing only. Shared checkout; all prior edits preserved. Rust file ownership released to lead after this handoff.

## Minimal design and source APIs

Observed defect: mounted `src/server.rs::ChatCompletionRequest` and native `CreateRunRequest` did not accept `response_format`; the provider could answer unconstrained text while the product advertises OpenAI-compatible structured output. The minimal path adds a typed request-scoped restriction, never a JSON prompt instruction or independent provider client.

Pinned source read: `vendor/git/liter-llm/crates/liter-llm/src/types/common.rs::ResponseFormat` has `Text`, `JsonObject`, `JsonSchema { json_schema: JsonSchemaFormat }`; `JsonSchemaFormat` contains `name`, optional `description`, object `schema`, optional boolean `strict`. `types/chat.rs::ChatCompletionRequest.response_format` is the actual serialized provider wire field. Existing `jsonschema::validator_for` is already used by production modules and the dependency is pinned in Cargo.toml; no dependency added. Existing Liter final-wire request serialization and output-ceiling enforcement remain the governing budget path.

## Files changed

- `src/llm/response_format.rs` (new): typed public text/object/schema shapes; unknown-field rejection; schema name/shape validation; local-only references; final projected JSON/object/schema validation.
- `src/llm/mod.rs`: module declaration only.
- `src/server.rs`: mounted OpenAI-compatible DTO, ingress validation, forwarding to the admitted run request, including the caller-tool adapter path.
- `src/uar/api/routes.rs`: native DTO, ingress validation, admitted request forwarding.
- `src/uar/runtime/turn/request.rs`: optional typed request-scoped format initialized absent.
- `src/uar/runtime/turn/resolved.rs`: immutable turn capture/accessors.
- `src/uar/runtime/manager.rs`: explicit request capture/pass into normal and shadow resolved turns; checkpoint admission identity includes format; invalid/unsupported graph combination fails through existing observable admission failure; legacy initializer remains format-absent.
- `src/llm/orchestrator.rs`: public inference preparation attaches scrubbed format after exact destination settings preparation for primary/retry/fallback; private summary/title calls continue using original preparation. Structured deltas are withheld through tool rounds and until the final complete JSON passes post-scrub validation. Malformed/schema-mismatched output yields `structured_output_invalid`, never success or plain-text fallback.
- `src/llm/liter_driver.rs`: extract internal prepared format from extra parameters into Liter's actual typed `ChatCompletionRequest.response_format` before final-wire accounting; eliminate duplicate extra-body key. Refuse adapters without a confirmed preserving wire path.

## Supported combinations and uncomfortable limits

Direct model execution through the mounted OpenAI-compatible and native run ingress supports `text`, `json_object`, and `json_schema` with the preserving OpenAI/Azure/OpenAI-compatible adapter. Actual unsupported model/schema combinations remain endpoint errors; no guessed family allowlist pretends model support. Existing output cap/budget enforcement and secret projection still govern dispatch. JSON schema output is checked after projection; a schema requiring a redacted secret fails explicitly rather than leaking it or reporting success.

`orchestrator-agent` with an installed agent graph plus a structured format explicitly fails before provider I/O as `structured_graph_unsupported`. The graph shares its model driver with router classification and does not currently inherit an output contract into selected-agent turns. The lead explicitly accepted documenting this bound-combination limitation without expanding child authority/interfaces. `text` and absent format retain graph behavior and attribution. Keep the Tier 1 direct OpenAI-compatible structured path in the required support ledger; do not downgrade that provider family. Graph selection attribution remains its typed metadata. No structured payload can acquire the graph's `[agent]` prefix because this unsupported combination is refused.

Anthropic captured/native paths explicitly reject structured requests; Anthropic structured output is not advertised in the support matrix. Non-preserving Liter adapters reject explicit structured format. Private classifier/router/summarizer calls are not given the public format. The format is request-scoped: callers resubmitting caller-tool continuation requests must resend their standard format option. Checkpoint/child format persistence/inheritance was not invented; those contracts remain outside this bounded implementation.

Structured streaming currently emits one validated final content chunk, followed by the established completion event, rather than publishing partial JSON that could later prove invalid. Tool-round prose stays internal. No `[agent]` prefix is introduced on direct execution.

## Genuine acceptance fixture requirements for capabilities writer

Keep these required, not documentation-only or ledger unsupported fallbacks. Use the configured real supported model in the mounted runtime path and inspect the actual provider request evidence; a prompt merely asking for JSON or a mock returning JSON is insufficient.

1. Mounted `/v1/chat/completions`: send `response_format: {"type":"json_object"}` with a planted unique nonce and requested fact. Assert actual outbound provider `response_format`, successful public completion, exactly one parseable JSON object, correct fact/nonce, no attribution prefix, and output ceiling still present on the actual wire.
2. Same route `json_schema`: `{"type":"json_schema","json_schema":{"name":"OutcomeNonce","strict":true,"schema":{"type":"object","properties":{"nonce":{"type":"string"},"count":{"type":"integer"}},"required":["nonce","count"],"additionalProperties":false}}}`. Request a unique planted nonce/count; assert provider field retains schema/strict/name, response validates the schema and contains the planted values. Repeat streamed and nonstreamed SDK calls; concatenated content must parse as the same single object.
3. Native `/api/uar/runs` with equivalent `response_format` and a direct agent/model selector: consume actual run events through terminal completion and validate the final content/schema. Inspect actual provider field rather than accepting only request deserialization.
4. Invalid schema name, unknown format type/fields, nonobject schema, malformed schema and external `$ref`: reject at ingress; no provider calls or successful run. These are trust-boundary guards, not speculative retries/fallbacks.
5. Provider fixture deliberately emits invalid JSON or a valid object missing a required property: assert `structured_output_invalid`, failed terminal result, and no public invalid content delta. A length-truncated document must not become a successful structured completion.
6. A supported caller-tool round followed by a structured answer: provider sees `response_format` on each public inference attempt, tool-round prose is absent from final content, final JSON validates, and provider cap/budget remains applied. Resubmit the same format option on continuation.
7. Structured request to installed graph target: explicit `structured_graph_unsupported` and zero provider calls. Absent/text format graph control still emits normal typed route selection and answer.
8. Unsupported adapter/model: explicit error; no silent format stripping or fallback to unconstrained text. A fallback preserving the supported wire must keep the identical scrubbed format.
9. Summary/title/private classifier controls: public format absent from helper provider requests and present only on user-answer requests. Include a schema with a captured credential sentinel and assert provider/public/persisted evidence does not leak the secret; a schema requiring the redacted secret must not succeed.

## Verification and remaining risk

Only source inspection (`rg`, `sed`, `cat`, `git diff` without check flags) and edits ran; no command is acceptance evidence. Rust compilation, serde behavior, real endpoint model support, schema conformance, streaming completion, SDK/native journeys, and exact final-wire budget behavior remain unverified until the operator opens the gate and names exact commands. The lead owns source identity inventory, independent review scheduling, support-ledger integration and the eventual approved production gate.
