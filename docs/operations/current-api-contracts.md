# Current additive API contracts

This index describes source-authored changes for #468–475. The selected lane is Linux amd64 container APIs behind a trusted host/gateway, SurrealDB default, PostgreSQL 18 within its explicit restrictions. It is not a current-artifact acceptance receipt. All #462–475 implementation must precede the one final operator-approved validation batch; generation, schema validation and runtime qualification remain closed.

The runtime's existing OpenAPI source is `src/uar/api/openapi.rs`, served through its existing API documentation integration. It retains the original customer route inventory/assertions and adds source-backed shapes below. This index does not introduce a product route. Authentication, owner/tenant/workspace context and host authorization remain enforced by their existing middleware/runtime boundary; functional scripts use external identity provisioning and do not manage credentials.

## Embeddings

`POST /v1/embeddings` keeps `usage.prompt_tokens` and `usage.total_tokens`. A zero can be a compatibility placeholder: consult the corresponding `uar_usage.prompt_tokens` / `uar_usage.total_tokens` `{available,source}`. A measured zero is available/provider_reported; missing measurement is unavailable/unavailable. Each counter is independent, and multi-chunk counts require complete reporting and valid checked sums. Native billed units have their own `uar_usage.billed_input_tokens` `{value,available,source}`; its source is provider_billed_units or unavailable. They do not become invented prompt/total counts or a zero-cost claim.

The response remains input ordered for float/base64 output on the same pinned configured backend/model/dimension. Invalid, missing, duplicate or out-of-range remote indices and malformed/nonfinite vectors refuse the batch. See [embedding usage accounting](embedding-usage-accounting.md). The OpenAPI `EmbeddingResponse` preserves the compatible numeric fields and describes the additive provenance/billing objects.

## Anthropic Messages

`POST /v1/messages` preserves the Anthropic agent-mode route and owner requirement. `uar_usage` is a sibling of `usage` on the nonstream message and in streamed `message_start.message` and `message_delta`. Its independent input/output availability means initial or unavailable numeric zero cannot be called measured usage. Available counters have source provider_reported; unavailable counters have source unavailable. Optional `cache_read_input_tokens` and `cache_creation_input_tokens` are omitted when unavailable/invalid; measured zero remains meaningful. Anthropic input excludes a valid reported cache breakdown rather than double counting it.

Errors and cancellation remain native terminal failures projected into the protocol; a partial/failed stream cannot become successful zero-usage completion. See the provider owner's [Messages contract](../ANTHROPIC_MESSAGES.md). Nonzero/cache/cancellation paths still require the final mounted source/artifact evidence; the historic keyless parity fixture is not that evidence.

## Native chat intent and history

The source-mounted read is `GET /api/uar/chat-turns/{client_turn_id}?session_id=...`. An owned durable reservation returns its original client turn, original user message, session and run IDs, a state, recovery=available and retry_allowed=false. States are pending/running/paused/complete/error/cancelled/interrupted/admission_unconfirmed. A durable nonterminal run with no resident lane is interrupted. No reservation returns unknown with retry_allowed=false; unsupported/unavailable persistence returns503, and missing verified principal returns401. A lookup never executes work.

The ordinary native POST now accepts client_turn_id and user_message_id together and requires a stable resolved session. Before dispatch it reserves the verified owner/session/workspace-bound intent using the existing durable store. Its semantic digest covers original rich messages, attachments, model/agent, request policy/settings, tools, output/response options and stream format; it stores the digest rather than those private bodies. Only a new reservation executes its original reserved native run ID, including bound-agent admission. An existing reservation returns the original JSON receipt; changed semantic input conflicts, and uncertain storage/admission never permits another dispatch. Caller-owned tool-batch continuations retain their separate durable batch identity and refuse native turn IDs; ordinary host-governed tool runs remain supported. Requests without IDs retain legacy behavior. This integration is source-authored; current-tree runtime acceptance and complete end-to-end recovery remain unverified. Local browser recovery retains uncertain intent rather than silently generating a new ID and resubmitting effects.

Session history adds stable message/run/client-turn IDs and explicit provenance/terminal availability. Restored display history is not a rewrite of canonical model/tool/checkpoint history. Missing/legacy provenance is unavailable; a disconnected stream or missing terminal cannot synthesize completed status. See [chat turn recovery](chat-turn-recovery.md) for bounded history/run evidence and migration limits.

## Memory selection and final submission

Automatic context selection requires finite operator-configured `memory.injection_min_similarity` in [-1,1]. It has no guessed numeric default. The runtime threads the typed selection provenance and effective configured cutoff; clearing the existing setting to null removes the cutoff instead of retaining a previous value. Absent configuration refuses automatic injection; per-turn opt-out and disabled memory remain authoritative. Memory is untrusted retrieved data, never policy, credentials or approval.

The content-free `/memory_context` state patch distinguishes disabled, opt_out, no_match, bounded_out, unavailable, refused and selected_prepared. Nonempty hits or a prepared block do not prove submission. A later submitted observation requires the selected fragment to survive the actual final driver request, including relevant native-system overrides, plus an observed provider response. Its evidence_basis is final_driver_request_and_provider_response, with original memory IDs, retrieval method, request_content_sha256 and model. That digest attests the observed request projection; it is not a fact body and does not prove model understanding.

The OpenAPI `MemoryContextObservation` documents these outcomes as an event/state shape, without inventing a memory-injection endpoint. Lead/runtime ownership retains final-fit/refusal/team/representation exclusions. See [memory injection provenance](memory-injection-provenance.md). Fresh-fact input observation and real-model recall must be independent of history and memory recall tools.

## Team attempt reconciliation

`GET /api/v1/collaboration/team-instances/{id}/attempts/{attempt}/reconciliation` reads owner/workspace-qualified original attempt/run/root identity, epochs, usage revision, team reservation/usage and execution/accounting/effect disposition. It requires the Surreal team execution capability; PostgreSQL parity is not promised. Missing or foreign scope does not expose tool arguments, output, prompt, credentials or fabricated provider call IDs.

The view points to owner-scoped accounting records for original provider-call identity. Only authoritative confirmed provider evidence through the existing trusted-host reconciliation boundary may settle billing. Generic provider-call settlement does not project team usage or release the separate team reservation: automaticTeamProjection=false and effectReplayAllowed=false remain explicit. teamCapacityRetained=true may remain after provider settlement. There is no new team-release mutation, forced-success or automatic uncertain-effect replay. See [team control reconciliation](team-control-reconciliation.md) for the manual continued-reservation procedure and limits.

## A2UI protected intent and surface authority

REST `POST /api/uar/runs/{run_id}/a2ui/actions` keeps surfaceId/name/sourceComponentId and accepts interactionId/surfaceGeneration with snake_case aliases. A protected action requires the stable interaction ID and string surface generation; a new deliberate click gets a new ID, while uncertain intent retains its original ID/generation/payload. Exact retry returns the original reserved continuation; changed semantics conflict. Legacy no-ID requests explicitly return retry_protected=false. Current declaration, owner, source-run policy, approval and host-resource checks still apply.

Read-only `GET /api/uar/runs/{run_id}/a2ui/actions/{interaction_id}` returns202 with original continuation receipt, not a dispatch grant. Outcomes running/completed/failed/cancelled/uncertain describe native run truth, not universal effect settlement. A missing404 receipt is not permission to repeat unconfirmed effects.

`GET /api/uar/runs/{run_id}/a2ui/surface-authority` returns complete/truncated/expired/unavailable, authoritative, bounds (64 prior runs/48 surfaces), recovery_hint and surface/source-run/string-generation tuples. Only complete plus authoritative=true supports a new protected click. A nonempty partial surface list does not grant authority. The supported single owned host does not claim distributed multiwriter ordering.

AG-UI carries the same intent/generation in forwardedProps.a2uiAction and exposes `GET /api/uar/ag-ui/agents/{agent_id}/threads/{thread_id}/actions/{interaction_id}` with the original wire/native identity. CUSTOM uar.a2ui.surface_authority and uar.a2ui.retry_protection expose the bounded authority/protection contract; ACTIVITY_SNAPSHOT content carries surface_generation. They do not add a new execution identity or replace AG-UI/native admission. See [A2UI interaction recovery](a2ui-interaction-recovery.md).

## Telemetry and measurement limits

`GET /metrics` serves only UAR's owned recorder. Host ownership returns503 with telemetry_scrape_unavailable and metrics_status, rather than a successful empty scrape; compiled-out telemetry returns404. Request series use matched route templates or unmatched, not raw resource paths. Subscriber/export ownership is separately inspectable in the host API; attached export does not attest collector delivery. See [telemetry ownership](telemetry-ownership.md) for five-second SDK acknowledgement versus unsupported hard flush/thread-release guarantees.

Capacity/SLO/comparative promises remain unselected and current evidence unverified. Comparison scope is NONE, including the explicit Rig disposition and future equivalent durability/glue methodology in [runtime measurement scope](runtime-measurement-scope.md). Historical helper benchmarks and stub-provider receipts cannot become current model-quality or capacity results.

## Trusted Rust host composition

`server::start_server_with_host_routes` accepts a trusted Rust host's additional
`Router<AppState>` along with the existing owned listeners and shutdown signals.
The routes pass through the ordinary server's production identity, governance
and tracing layers. Existing entrypoints install no additional routes. This is
not network/configuration route installation, an identity grant or a sidecar
launch-token guard. It supplies no new release-qualified Rust embedding lane.
Transferred functional fixtures use this composition seam behind externally
provisioned identity ingress; genuine verified context stays in process and
tests neither obtain nor send authentication credentials. Runtime compatibility
and lifecycle behavior of the new entrypoint remain unverified.
