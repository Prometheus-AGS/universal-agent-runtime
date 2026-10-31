# Runtime content and destination-admission contract for #463/#464

## Scope and boundary

Phase: Plan. This is source-only interface preparation for the next bounded
#463/#464 work after the frozen #462 worktree is released. The current
authoritative waypoint is `identity-pass-through`, status `ready`; it does not
authorize this next implementation. The verification lock is CLOSED.

This note proposes only runtime-owned contract changes. It does not qualify an
image path, provider adapter, route, budget, or checkpoint behavior. It does
not change the checkout. The sole output is this file.

## Proposed admitted-current-content contract

`RunExecutionRequest.input: Option<String>` remains the text-only compatibility
and matching projection. It must not become a carrier for a JSON multipart
string or a data URI. Add this adjacent host-only field in
`src/uar/runtime/turn/request.rs`:

```rust
/// Current user content admitted by a trusted host. This is never deserialized
/// from an agent, tool, or model payload. `input` is its text-only projection.
pub(crate) current_user_content: Option<crate::llm::MessageContent>,
```

Add this crate-private constructor-style builder in the same implementation:

```rust
#[must_use]
pub(crate) fn with_admitted_current_user_content(
    mut self,
    content: crate::llm::MessageContent,
) -> Self {
    self.input = Some(current_user_text_projection(&content));
    self.current_user_content = Some(content);
    self
}
```

`current_user_text_projection(&MessageContent) -> String` is runtime-owned and
must return the complete text projection:

- `MessageContent::Text { content }` returns that string.
- `MessageContent::Parts { content }` concatenates every `ContentPart::Text`
  in original order, with one stable `\n` delimiter between retained text
  parts; it omits every `ImageUrl` body and metadata.
- Image-only content therefore projects to the empty string but is still a
  present `input` (`Some("")`) and remains an appendable current turn.

This helper must not call `MessageContent::as_text()`: the existing method
returns only the first text part (`src/llm/mod.rs:118-130`). It must not inspect
or stringify image URLs. Ingress owns validation and may call the builder only
after it has produced ordered typed `MessageContent::Parts`; the builder does
not validate caller JSON or grant authority.

The manager constructs the canonical user `Message` once by value from
`current_user_content.clone().unwrap_or_else(|| MessageContent::text(input.clone()))`.
Its role is `User`, and its tool fields are `None`. Each append needs a clone of
that same canonical message; it must not reconstruct from `input`. Existing
text-only callers keep `current_user_content: None` and therefore retain exact
current constructor behavior.

## Constructor, literal, and move inventory

The added field defaults to `None` in `RunExecutionRequest::new` at
`src/uar/runtime/turn/request.rs:170-204`. `from_bound_agent` calls `new`, so
it needs no second literal initialization. The only direct struct literal found
in either the current checkout or the frozen #462 worktree is
`src/uar/runtime/manager.rs:3475-3512`
(`start_run_from_checkpoint`); it must set `current_user_content: None`.

The only whole-struct destructure is
`src/uar/runtime/manager.rs:4236-4269`; it must bind
`current_user_content` with the request's other host-owned values. The existing
`TurnAssemblyPlan` may continue to derive `append_input` from `input.is_some()`
because the builder always sets it to `Some`, including image-only input.

The source constructor call inventory is:

- Production: `src/server.rs`, `src/uar/api/acp/handler.rs`,
  `src/uar/api/full_harness/host_context.rs`, `src/uar/api/openai/routes.rs`,
  `src/uar/api/routes.rs`, `src/uar/mcp_server.rs`,
  `src/uar/runtime/manager.rs`, `src/uar/runtime/team_execution/execution.rs`,
  `src/uar/runtime/thread/actor_host.rs`, `src/uar/runtime/thread/kernel.rs`,
  and `src/uar/runtime/turn/plan.rs`.
- Tests/support: `tests/context_history_integrity.rs`,
  `tests/credentials_api_integration_test.rs`, `tests/mcp_projection.rs`,
  `tests/skill_activation_runtime.rs`,
  `tests/support/uar_1_1_1_runtime_fixture.rs`,
  `tests/test_chat_completion.rs`, and `tests/typed_turn_assembly.rs`.

The old constructor callers do not need to supply the field. Their behavior is
protected by `new`'s default. The ordinary OpenAI aliases must use the builder;
excluded ACP, A2A, native-run, client-tool history, and attachment paths must
not obtain multimodal convenience behavior merely because the field exists.

`checkpoint_request_identity` at `src/uar/runtime/manager.rs:640-655` currently
binds only `input`, response format, checkpoint state and history. If a resumed
request can carry admitted current content, it must bind a SHA-256 digest of
canonical `current_user_content` JSON (or a `None` marker), not raw data URI
bytes. Otherwise a post-admission swap of the image remains outside its
continuation identity. This is an additional required update, not covered by
the blueprint's constructor inventory.

## Canonical append and safe-projection seams

The current manager has three text reconstruction sites. All must use the one
canonical current user message described above when `append_input` is true:

| Seam | Existing source behavior | Required replacement |
| --- | --- | --- |
| Initial session | `manager.rs:5133-5136` calls `session.add_user_message(&input)` | `session.add_message(admitted_current_message.clone())`. |
| Retained warm session | `manager.rs:6949-6958` calls `retained_session.add_user_message(scrub(&input))` | Add the projected canonical message with `retained_session.add_message(...)`; do not reduce it to text first. |
| Restored/inherited provider history | `manager.rs:7304-7317` pushes `MessageContent::text(input.clone())` | Push the same canonical message into `messages`; do not append both a typed and a text copy. |

This covers ordinary warm history, cold checkpoint history and inherited child
history. `Session::add_message(Message)` already preserves typed content
(`src/session/thread.rs:271-277`), and `Message` plus `MessageContent` already
serialize image parts (`src/llm/mod.rs:73-103,185-239`). No parallel image
store or checkpoint schema is needed.

The derived `input` continues to feed existing text-only operations. In
particular, the manager creates `routing_input` from it at
`manager.rs:5515-5519`, then sends it to retrieval at `5600-5607` and to both
skill-matching paths at `5908` and `5925/5939`; the safe public run-context
projection is written at `6963-6967`. Any guardrail/matching/memory path that
already consumes `input` consequently receives every text part in order and no
base64. New code must keep that property; it must not receive the canonical
image message as an alternate matching input.

### Scrubber requirement

The generic scrubbers are not safe for canonical inline image bytes today:

- `RunSecretScrubber::project_message` rewrites every `ImageUrl.url`
  (`src/uar/runtime/turn/host/event_projection.rs:144-157`). Manager uses it
  while projecting history at `6925-6928`, staged seed history at `6950-6954`,
  and final messages at `7321-7323`.
- `project_message_json` rewrites all JSON `content` at
  `event_projection.rs:122-142`; `Orchestrator::prepare_attempt` calls it for
  request and canonical messages at `src/llm/orchestrator.rs:1059-1065`.
- `CheckpointHost::persist` serializes an entire `GraphState` through generic
  `project_value` before `Checkpoint::new`
  (`src/uar/runtime/checkpoint_host.rs:83-89`). Checkpoint protection then
  hashes and stores those projected `state.messages` values
  (`src/uar/runtime/checkpoint.rs:75-97`) and restores them through
  `history_from_checkpoint`.

Add a named canonical-history projector rather than weakening generic secret
projection. It must scrub text content and tool arguments as today, but preserve
the exact `ImageUrl.url` only when it revalidates as the bounded, admitted
`data:image/png;base64,...` or `data:image/jpeg;base64,...` grammar. It must
reject rather than repair an invalid image value. Use that projector at the
three manager sites, at the provider request/canonical JSON projection seam,
and for the `GraphState.messages` portion of checkpoint persistence. Generic
`GraphState.data`, prompt fragments, tool arguments, ordinary URLs, errors,
logs and run context continue through the normal scrubber.

This revalidation is necessary because `Message` currently has no provenance
field that says an `ImageUrl` passed trusted-host inline-image admission. A
syntax-only exception without the same strict MIME/base64/signature validation
would convert arbitrary image URLs into unsanitized canonical content. Adding a
serialized provenance bit instead is a format-contract expansion and must be
explicitly allocated; it is not required for the proposed bounded slice.

The current checkpoint raw history preserves JSON values and hashes, but it does
not prove that a data URI survives the generic scrubber. That is an unimplemented
requirement, not a behavior to claim from source inspection.

## Destination authority and final-send admission

The routing/provider owner proposed one provider-owned reusable contract in
`src/llm/router.rs`:

```rust
#[derive(Debug, Clone)]
pub struct DestinationAdmission {
    // router/registry access plus immutable authority only
    authority: DestinationAuthority,
}

#[derive(Debug, Clone)]
pub enum DestinationAuthority {
    Exact { model: String },
    Bounded { primary: String, fallbacks: std::collections::BTreeSet<String> },
    Unbound,
}

impl DestinationAdmission {
    pub async fn admit(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<AdmittedDestination, DestinationAdmissionError>;
}
```

`RouteRequirements` is not captured in `DestinationAdmission`. It is derived
fresh from the final public `LlmRequest` immediately before each send, after
destination preparation and response-format controls. The evaluator must first
enforce authority, then current runtime provider/model enabled state and
cooldown, then required capability evidence. A required capability with unknown
runtime metadata is a refusal. Catalog data may fill metadata for an enabled,
unregistered model only where the provider design explicitly allows it; it may
not resurrect a registered disabled model.

The `RouteRequirements` derivation must inspect the final request, not the
initial user string:

- vision: any final provider message deserializes to a `Message` with image
  parts;
- tools: nonempty final `LlmRequest.tools`, or `ToolChoice::Required`/`Named`;
- structured output: the trusted resolved response-format contract is
  structured after `prepare_user_attempt` has inserted it;
- reasoning: only an explicit admitted reasoning requirement, not an optional
  configured thinking budget;
- context/cost: preserve existing semantics, while final-destination request
  preparation remains the authority for actual capacity and #388 accounting.

`LlmRequest` already supplies final messages, tools and tool choice
(`src/llm/mod.rs:336-367`). `prepare_user_attempt` applies per-destination
preparation and response-format controls at `orchestrator.rs:1098-1124`, so
preparation must occur before this derivation. Text-only helper inference stays
on `prepare_attempt` and must not inherit the public run's requirements.

### Runtime composition

After `RunModelBindings` has captured the final primary and actual fallback
clients, runtime creates one `Arc<DestinationAdmission>` from immutable captured
authority. `RunModelBindings` owns the private primary/fallback identities
(`src/uar/runtime/turn/bindings.rs:231-240,301-383`), so add a runtime-private
accessor there rather than reconstructing authority from configuration strings.
It must report the exact captured model identity set, not all credential grants.

- `target_selection.is_some()` becomes `Exact { model: primary }`; its existing
  source rule already clears failover (`manager.rs:6600-6605`).
- A captured bound/team/credential route becomes `Bounded { primary, fallbacks
  actually captured }`; it may never escape through global fallback settings.
- Ordinary unbound routing uses `Unbound`, but only with the already captured
  driver set; it does not authorize a new driver or credential resolution.

The manager supplies this same `Arc` to both public execution mechanisms:

1. Add it to `Orchestrator` with a provider-owned
   `with_destination_admission(Arc<DestinationAdmission>)` builder. In the
   primary retry closure at `orchestrator.rs:2489-2513`, and for each fallback
   at `2619-2633`, call `admit` after `prepare_user_attempt` and before
   `open_driver_stream`. A destination-admission refusal must be distinguished
   from a provider failure: do not cool down the provider, do not turn it into
   an unrestricted fallback, and do not send it.
2. Extend `DestinationBudgetDriver` at `manager.rs:66-93` with
   `admission: Arc<DestinationAdmission>` and `destination: String`. Its
   `stream` first attaches/checks the existing destination budget contract and
   output cap, then derives requirements from that final graph `LlmRequest`,
   calls `admit(&destination, &requirements)`, and only then calls
   `inner.stream`. The construction seam is `manager.rs:8018-8022`.

The graph wrapper is admission-only: it does not rank, select, replace or
fallback a destination. This preserves graph cancellation/join ownership and
the already captured driver/budget. The orchestrator remains responsible for
each retry/fallback's own preparation; the wrapper preserves the #388 final
budget contract on the graph path. No request should be rerouted after captured
model, credential, owner, billing, continuation, policy, or budget bindings
exist.

### Current gaps the provider implementation must close

`ModelRouter::route_with_preferred_model` filters enabled runtime models, then
supplements missing candidates from the catalog when only the provider is
configured (`src/llm/router.rs:62-136`). A disabled runtime model can therefore
be absent from the first pass and reappear via catalog supplementation. The
new evaluator must make an explicit runtime registration's disabled state a
veto. The existing `route` answer is advisory and cannot be reused as actual
send authority.

The manager also currently falls back to `preferred_llm_config` when routing or
resolution has no match (`manager.rs:6525-6540`). For a hard capability or
authority requirement, this must become an explicit no-dispatch refusal. The
no-registry path lacks runtime capability metadata; it must refuse a required
capability rather than assume support.

`RunModelBindings::for_policy` can bind an exact child model or fall back to an
eligible captured binding and a model-bound driver (`bindings.rs:439-473`). The
new authority accessor must describe the resulting captured identities, and the
final evaluator must still reject any actual destination outside them. Do not
infer that a credential grant authorizes arbitrary model identities.

## Final budget and checkpoint invariants retained

The next phase must retain `DestinationRequestPreparations`, per-destination
settings/profile, response/reasoning reserves, protected continuity, and the
leaf final-wire contract. Existing budgeting in `DestinationBudgetDriver` stays
in front of graph dispatch; the new admission hook adds no token allowance and
does not replace destination preparation. A real image data URI does not prove
an image-token count. A strict guaranteed-fit image policy therefore requires a
validated image-aware counting contract or a pre-dispatch refusal; it cannot
reinterpret base64 JSON length as provider-exact accounting.

Checkpoint raw history remains one protected JSON history, not a text projection
or parallel blob store. `Checkpoint::new` hashes raw graph state/history and
`history_from_checkpoint` deserializes complete `Message` values
(`src/uar/runtime/checkpoint.rs:75-97,247-257`). The canonical-image projector
must preserve those bytes across warm history, checkpoint creation, checkpoint
restore and final provider history. It must not change checkpoint authorization,
claim, cancellation, join or durable mutation ownership in the trusted host.

## Required source ownership handoff

- Runtime owns `src/uar/runtime/turn/request.rs`,
  `src/uar/runtime/manager.rs`, and the necessary runtime binding/checkpoint
  helper changes. It must retain trusted-host policy intersection, reservations,
  captured authority, cancellation/join and durable-mutation boundaries.
- Routing/provider owns `src/llm/router.rs` and `src/llm/orchestrator.rs`,
  including the shared evaluator, final-send guards and JSON canonical-message
  projection contract. Runtime consumes its public types only.
- Ingress owns `src/server.rs` typed parse/admission and calls the new builder
  only for the two selected ordinary OpenAI aliases. It rejects excluded forms
  before any RunManager execution.

No change to `src/llm/mod.rs`, session persistence schema, vendor source,
dependency pins, or the pinned skill-system submodule is justified by this
runtime preparation alone.

## Unresolved implementation limits

1. The existing `Message` type permits arbitrary URL strings and has no
   validated-inline-image provenance. The canonical-image scrubber must share
   strict ingress validation or a later phase must deliberately introduce a
   format-contract marker.
2. Existing source establishes serialization and checkpoint integrity hashing,
   not successful provider image transport or image understanding.
3. The provider-owned evaluator needs a concrete diagnostic/reason vocabulary;
   no current `RouteRequirements` API returns a reusable admission reason.
4. A behavior-level evidence plan and exact commands remain for the future
   phase. No verification command was run for this note.

## Superseding provenance and contract reconciliation — 2026-10-31

This section supersedes the prior note's repository-position claims. Initial
source reads used the default checkout at
`/Users/gqadonis/Projects/prometheus/universal-agent-runtime`; its waypoint
reported the unrelated `identity-pass-through` phase and `144/170` completion.
That evidence is not authoritative for this #462-frozen planning packet.

The reconciliation source is exclusively:

```text
path: /Users/gqadonis/.claude/worktrees/production-readiness-476
HEAD: 8e0602eecbd37504e8cdf8945f2f1c5537ff6701
waypoint: production-readiness-476-baseline
status: IMPLEMENTED_ACCEPTANCE_PENDING
stage: verify
testedSource: 8e0602eecbd37504e8cdf8945f2f1c5537ff6701
```

Every source claim below was re-read at that explicit worktree path only:

- `RunExecutionRequest` still has legacy `input`, typed host/checkpoint and
  inherited histories, `new`, and `from_bound_agent` in
  `src/uar/runtime/turn/request.rs:76-340`; it does not yet have admitted
  current content.
- The sole direct request literal remains
  `src/uar/runtime/manager.rs:3475-3512`, and the sole whole-request
  destructure remains `4236-4269`; `TurnAssemblyPlan` still derives append from
  `input.is_some()` in `src/uar/runtime/turn/plan.rs:12-26`.
- The three text reconstruction seams remain at `manager.rs:5133-5136`,
  `6949-6958`, and `7304-7317`. Existing `Session::add_message(Message)` is
  confirmed at `src/session/thread.rs:271-277`.
- The aggregated text projection feeds retrieval and skill matching through
  `routing_input` at `manager.rs:5515-5519` and `5908/5925/5939`; run context
  still writes the text-only value at `6963-6967`.
- Generic content projection still rewrites `ImageUrl.url` at
  `src/uar/runtime/turn/host/event_projection.rs:144-157`; it is invoked for
  manager histories, JSON provider request/canonical messages in
  `src/llm/orchestrator.rs:1052-1124`, and the graph-state checkpoint projection
  at `src/uar/runtime/checkpoint_host.rs:83-89`. Checkpoint raw history still
  records graph message JSON at `src/uar/runtime/checkpoint.rs:75-97`.
- `DestinationBudgetDriver` still contains only `inner`, `contract`, and
  `output_token_cap` at `manager.rs:66-93`, and is constructed at `8018-8022`.
  Primary retry and fallback attempt preparation still immediately precede
  `open_driver_stream` at `src/llm/orchestrator.rs:2489-2513` and `2619-2633`.
- `RunModelBindings` still privately owns the captured primary/fallback models
  at `src/uar/runtime/turn/bindings.rs:228-385`; child rebinding remains at
  `425-550`. `ModelRouter` still performs runtime metadata filtering before
  catalog supplementation at `src/llm/router.rs:35-160`.

The code-seam conclusions in the earlier sections stand after this re-read. The
only changed conclusion is provenance/phase position: this note must not cite
the original checkout's `identity-pass-through` state as context for the frozen
#462 worktree.

### Reconciled shared provider/runtime contract

The routing/provider and runtime packets now use exactly these names; this
replaces the earlier `DestinationAuthority`/`Unbound` sketch:

```rust
#[derive(Debug, Clone)]
pub enum DestinationScope {
    Exact { primary: String },
    PrimaryWithFallbacks {
        primary: String,
        fallbacks: std::collections::BTreeSet<String>,
    },
}

#[derive(Debug, Clone)]
pub struct DestinationAdmission {
    scope: DestinationScope,
    // provider-owned router/registry access; no credentials or new authority
}

impl DestinationAdmission {
    pub async fn admit(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<AdmittedDestination, DestinationAdmissionError>;
}
```

The provider/routing owner constructs `DestinationAdmission` and owns its
metadata/cooldown/capability evaluation. Runtime owns the private
`RunModelBindings` accessor that constructs the `DestinationScope` only after
the actual primary and actual captured fallback clients exist. For an explicit
`target_selection`, it constructs `Exact { primary }`. For every other path it
constructs `PrimaryWithFallbacks` from the captured primary plus the actual
captured fallback identities. It must not derive scope from configuration text,
the full credential catalog, a generic provider grant, or a registry query.

There is deliberately no `Unbound` variant. Ordinary routing has already
captured a concrete client set before execution; a scope with no immutable
destination membership would turn admission into a new authority grant. The
`BTreeSet` answers membership only. Orchestrator retains its current ordered
fallback `Vec`, so this contract neither reorders failover nor selects a new
destination.

Runtime passes `Arc<DestinationAdmission>` to the primary/fallback
orchestrator path and to `DestinationBudgetDriver` with its fixed destination
identity. Each send derives requirements from the final prepared request and
calls `admit`; no route query, failed primary, cooldown, catalog entry, or
credential grant can expand the captured scope.

Provider metadata remains a declaration used by this admission evaluation, not
certification that a remote endpoint accepts, counts, or understands images.
The current Liter receipt's `fit_guarantee` treatment for requests carrying
images is provider-owned follow-up: it must not label real image input as a
guaranteed fit merely because a budget contract exists. This is not a reason to
add global provenance fields, public model allow/deny APIs, or a generic
metadata schema redesign to this bounded phase.

### Reconciliation limits

- This audit reconfirmed source shapes and paths only. It did not establish
  compile success, runtime behavior, image transmission, model understanding,
  checkpoint preservation, or provider capability certification.
- The exact `DestinationAdmissionError` reason vocabulary and
  `AdmittedDestination` fields remain routing/provider-owned decisions.
- The canonical-image projection still needs a shared strict validation rule or
  a deliberately allocated format marker before it can safely preserve an image
  while generic secret projection runs. No global provenance redesign is
  proposed here.
