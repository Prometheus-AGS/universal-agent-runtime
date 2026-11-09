# #463/#464 provider admission: source-only interface design

## Scope and status

This is a design handoff, not an implementation or acceptance record. The
authoritative waypoint currently names `identity-pass-through` as the active
phase and reports 144/170 implementation items. The operator verification lock
is closed. I inspected source only; no checkout file, vendored Liter source,
dependency pin, test, formatter, build, compiler probe, or external state was
changed.

This note covers the requested provider-owned seams only:

- `src/llm/router.rs`
- `src/llm/orchestrator.rs`
- `src/llm/liter_driver.rs`
- `src/llm/mod.rs` only if it must export a shared provider type

The runtime writer owns creation of immutable authority and injection into the
orchestrator and graph wrapper. This note does not add public allow/deny policy,
change target verification, or alter existing authority/credential records.

## Source facts that constrain the interface

1. `ModelRouter::route_with_preferred_model` currently constructs runtime
   candidates from `ProviderRegistry::list`, skipping disabled providers/models
   and provider cooldown. It then supplements candidates from `ModelCatalog`
   when only the provider is configured. Thus a catalog entry can currently
   reappear for a registered-but-disabled model.
2. The registry's `ProviderConfig.id` and `ModelConfig.id` are the literal
   runtime destination identity. `detect_provider(model)` is adapter-family
   inference and must not decide authority or capability certification.
   `ModelCatalog` is discovery/ranking metadata, not an authorization grant.
3. `DestinationRequestPreparations::prepare` resolves the exact per-destination
   template, settings and budget contract. `prepare_user_attempt` then adds the
   effective `response_format`. The public-attempt guard must therefore follow
   both calls and precede `open_driver_stream`.
4. The retry closure and each fallback independently call
   `prepare_user_attempt`; an admission result from the first primary attempt
   cannot be reused. `DestinationBudgetDriver::stream` is a separate graph
   path that sets the exact budget/cap then calls `inner.stream`.
5. `LlmRequest` already retains the final messages, tool declarations,
   `tool_choice`, `thinking_config`, `extra_params`, response cap and budget.
   It is sufficient to derive request requirements. Do not add routing,
   authority, health, catalog or certification fields to it.
6. Liter's `generic_chat_adapter` is an adapter-construction fact used by
   `build_chat_request`, distinct from provider/model spelling. Its current
   `from_client` constructor initializes that fact to `false`; an arbitrary
   pre-built `DefaultClient` therefore cannot certify generic-compatible wire
   semantics. Tool-call delta assembly and streaming are already downstream of
   `build_chat_request` and must remain untouched.

## Proposed shared router API

Keep the evaluator in `router.rs`. It owns live registry/health inspection and
the separation between runtime facts, catalog hints and the immutable run
destination scope. Names below are proposed new APIs, not claims that they
already exist.

```rust
// src/llm/router.rs
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationScope {
    /// A verified explicit target or team-selected route. It has no substitute.
    Exact { primary: String },
    /// The run host captured the exact primary and its already-authorized,
    /// ordered fallback identities after credential/binding resolution.
    PrimaryWithFallbacks {
        primary: String,
        fallbacks: Vec<String>,
    },
}

impl DestinationScope {
    pub(crate) fn permits(&self, destination: &str) -> bool;
    pub(crate) fn permits_fallback(&self, destination: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationAdmissionRejection {
    MalformedDestination,
    OutsideCapturedScope,
    RuntimeMetadataUnavailable,
    ProviderDisabled,
    ModelDisabled,
    ProviderInCooldown,
    RequiredCapabilityUnknown { capability: &'static str },
    MissingRequiredCapability { capability: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdmittedDestination {
    /// Exact literal `provider/model` that passed this one evaluation.
    pub(crate) destination: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DestinationAdmission {
    router: ModelRouter,
    scope: DestinationScope,
}

impl DestinationAdmission {
    pub(crate) fn new(registry: Arc<ProviderRegistry>, scope: DestinationScope) -> Self;

    /// Re-evaluates an already chosen actual destination at send time.
    /// It does not select a model, acquire credentials, or change the scope.
    pub(crate) async fn admit(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<AdmittedDestination, DestinationAdmissionRejection>;
}

impl ModelRouter {
    /// Uses live runtime registration/health first. Catalog values may rank a
    /// candidate only after this function supplies a runtime-capability result.
    async fn evaluate_destination(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<(), DestinationAdmissionRejection>;
}
```

`DestinationScope` is deliberately private-to-crate plumbing, not a new
operator policy. The runtime creates it only from the route already admitted by
the existing target/team/binding/credential flow and the resolved configured
fallback list. `Exact` is used for explicit target and team selections. An
ordinary run receives its resolved primary plus only its existing authorized
fallback identities. The evaluator never turns a default, a preferred model,
or a catalog candidate into another permitted destination.

`ModelRouter::route*` must call the same `evaluate_destination` logic while
forming its candidates. For a registered `(provider, model)`, disabled runtime
metadata wins over catalog data. A catalog-only candidate may contribute cost,
context and benchmark ordering only; it is not capability certification. If a
hard requirement needs a fact that has no runtime certification, reject it as
`RequiredCapabilityUnknown` rather than treating catalog or an alias as proof.
Health is re-read immediately before every send, so cooldown after initial
routing is still observed.

The evaluator must split the literal `provider/model` with
`registry::split_model_string_pub`; it must not use `detect_provider`. A wire
model alias in `EndpointRequestProfile.wire_model` remains a transform detail
and is not a second destination identity.

### Capability derivation after final preparation

Add a private router helper, invoked by both send paths, rather than a field on
`LlmRequest`:

```rust
// src/llm/router.rs; private helper
fn requirements_for_final_request(request: &LlmRequest) -> Result<RouteRequirements, anyhow::Error>;
```

It starts from the existing context/cost requirements captured for the run, then
sets hard booleans from the final request as follows.

| Requirement | Final-request evidence |
| --- | --- |
| `needs_vision` | Any final `messages[*].content` contains a canonical `image_url` part, including retained history. |
| `needs_tools` | `tools` is nonempty, or `tool_choice` is `Required`/`Named`. An installed global registry alone does not count. |
| `needs_structured_output` | `extra_params.response_format` decodes as `ResponseFormat` and `is_structured()` is true. This happens after `prepare_user_attempt` projects and inserts the turn format. |
| `needs_reasoning` | A final explicit reasoning control is present: `thinking_config`, or known final dialect controls emitted by `PromptDialectEngine` (`thinking`, `reasoning_effort`, `enable_thinking`). Optional provider reasoning output is not a requirement. |

Malformed final `response_format` must be an invalid prepared request, not a
silent `false`. The image scan must inspect all final message JSON, not
`MessageContent::as_text()` or only the new user turn. Existing
`validate_tool_controls` remains the source of selection/schema consistency;
this helper only derives capability demand.

### Runtime metadata uncertainty that must be resolved before implementation

`ModelConfig.supports_tools` presently defaults to `true` when omitted, while
the other capability booleans default to `false`. After deserialization the
current `bool` fields do not distinguish an operator's explicit assertion from
the default. Consequently the requested rule "unknown is not positive evidence"
cannot be proved for tools using only the current shape. The router writer must
either preserve field-presence/provenance during registration or obtain an
explicit capability certificate from the host registration path. Do not label a
defaulted `true` as certified merely to make capability routing succeed.

## Send-boundary wiring

### `src/llm/orchestrator.rs`

Add a field and builder; neither changes `LlmRequest`:

```rust
destination_admission: Option<Arc<router::DestinationAdmission>>,

pub(crate) fn with_destination_admission(
    mut self,
    admission: Arc<router::DestinationAdmission>,
) -> Self;

async fn admit_prepared_user_attempt(
    &self,
    destination: &str,
    request: &LlmRequest,
) -> anyhow::Result<()>;
```

`admit_prepared_user_attempt` derives requirements through
`requirements_for_final_request` and calls `DestinationAdmission::admit`.
When the host installed a registry/admission object, rejection is terminal for
that attempt and must be reported as an admission/preparation refusal, not fed
to retry-health accounting. It must not mutate messages, tools, tool IDs,
results, cancellation state, or a budget contract.

Call it in exactly these existing public inference locations:

1. The retry closure, after `prepare_user_attempt(&model, ...)` and after its
   response-format addition, immediately before `attempt_manifest`/
   `open_driver_stream`.
2. Each fallback loop iteration, after its own `prepare_user_attempt` and
   immediately before `open_driver_stream`.

The initial prepared request is not a substitute for the retry-closure guard.
Each retry must re-read health and rederive requirements. Fallback admission
uses the same canonical request, but its own destination preparation/settings,
wire model and budget profile; it does not reuse primary certification.

No public model selection happens here. A scope failure ends that candidate;
an explicit target has an empty fallback scope, so there is no substitute send.
Existing stream-start/idle cancellation and incremental normalized stream flow
remain after the guard, unchanged. Since the guard is before
`open_driver_stream`, it cannot consume or rewrite tool-call/result identity.

### `src/uar/runtime/manager.rs` handoff to the runtime writer

After the current final route, credential/binding and fallback resolution, build
one `Arc<DestinationAdmission>` from the existing `ProviderRegistry` and a
`DestinationScope` representing only that resolved primary/fallback set.
Inject that exact `Arc` into:

```rust
// Existing orchestrator construction chain
.with_destination_admission(Arc::clone(&destination_admission))

// Existing graph wrapper
DestinationBudgetDriver {
    inner: model_bindings.primary(),
    contract: destination_budget_contract.clone(),
    output_token_cap: output_token_cap.unwrap_or(0),
    destination: run_llm_config.model.clone(),
    admission: Arc::clone(&destination_admission),
}
```

The wrapper applies its existing contract and output cap first, then derives
requirements from that final graph `LlmRequest`, calls
`admission.admit(&self.destination, &requirements).await`, and only then calls
`inner.stream(request).await`. It does not route, retry, bind another model or
invent a graph fallback. This closes the graph-only bypass while preserving the
existing final-wire budget injection.

If a host supplies a driver without a registry, the runtime must not omit the
guard for a request that has any hard capability demand. It must return an
explicit unavailable/unknown-capability refusal before send. A text-only legacy
external-driver request may retain existing behavior only if no hard capability
claim is made; it is not a certified vision/tool/structured/reasoning path.

### `src/llm/liter_driver.rs`

Keep Liter's adapter safety separate from router capability data. The driver
should make its construction evidence explicit internally:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdapterBinding {
    GenericOpenAiCompatibleChat,
    ProviderAdapterOrOpaqueClient,
}
```

`from_endpoint_profile` owns a complete explicit-endpoint configuration and
may set `GenericOpenAiCompatibleChat` only on the successful supported generic
constructor branch. `from_client` remains
`ProviderAdapterOrOpaqueClient`; a borrowed `DefaultClient` has no source API
that proves which Liter adapter it constructed. `with_bound_model` preserves,
never promotes, this evidence.

Use this binding in the current `build_chat_request` checks in place of a
provider-family alias. It lets an explicitly constructed compatible endpoint
preserve tool/parallel controls while a native/opaque adapter retains its
existing refusal. It is adapter/wire compatibility evidence, not catalog
vision/reasoning/tool certification and not destination authority.

The literal registry provider ID, `EndpointRequestProfile.endpoint_fingerprint`,
profile `wire_model`, and `AdapterBinding` must remain separate values. Do not
infer generic-compatible capability from a model alias. Do not edit vendored
Liter.

## Budget and streaming invariants

`DestinationRequestPreparations` and `LiterLlmDriver::enforce_request_budget`
already bind a destination model, endpoint fingerprint, settings/output ceiling
and final serialized wire body. Preserve all of that per primary/retry/fallback.
The current `SyntheticOpenAiCompatibleChatV1` counter can prove the serialized
wire representation it counts; it does **not** establish how a real destination
accounts for image content. A successful final JSON count must therefore never
be reported as a guaranteed real image fit. Existing attempt manifests already
publish `fit_guarantee: false`; retain that posture.

This design does not invent a new image-cost allowance or relabel base64 JSON
tokens as provider image tokens. A strict real-image-fit lane needs a separate,
destination-specific accounting contract. Until then, preserve final-wire
overflow protection and explicitly refuse any caller/run that genuinely requires
that guarantee.

The admission guard happens before a provider stream is opened. It therefore
does not collect a stream, alter stream establishment/idle cancellation, or
transform tool call/result deltas. Existing `ToolAccum` identity accumulation,
lossless `convert_messages(..., require_lossless)` behavior and normalized
partial-stream semantics remain the sole downstream behavior.

## Concrete blueprint corrections

1. Do not capture `RouteRequirements` once in a run-level admission object.
   History, tool declarations, response format and dialect controls change the
   final request. Capture only immutable authority; derive requirements at every
   final send.
2. Do not guard before `prepare_user_attempt`. That would miss the final
   response-format control and admits a different request than the one sent.
3. Do not let catalog supplementation bypass a disabled runtime model, and do
   not treat catalog capabilities as certification for hard requirements.
4. Do not use `detect_provider` or a model alias for destination authority.
   They identify an adapter family; the registry/profile carries the exact
   endpoint identity.
5. Do not silently allow a pre-built Liter client as generic-compatible.
   `from_client` currently starts `generic_chat_adapter` false, so only a
   constructor-controlled path can establish generic adapter evidence.
6. Do not call the current exact serialized-wire budget an exact real-image
   fit. The source offers no image-token accounting evidence.
7. Do not add `DestinationScope`, health or capability flags to `LlmRequest`.
   The immutable run boundary plus its final request supplies the needed data
   without widening every request constructor.

## Ownership and verification state

Proposed implementation ownership remains: routing/provider writer owns
`src/llm/router.rs`, `src/llm/orchestrator.rs`, `src/llm/liter_driver.rs` and,
only if required, `src/llm/mod.rs`; runtime writer owns
`src/uar/runtime/manager.rs` construction and graph wrapper integration. The
two writers need a small handoff on the exact `DestinationScope` construction;
no shared source file should have two writers.

Changed by this task: only this design artifact,
`/tmp/uar476-provider-admission-design.md`. No verification command was run.
All compile behavior, actual endpoint selection, capability admission,
disabled/cooldown handling, fallback behavior, wire capture, cancellation and
image budget behavior remain unverified.

## Superseding provenance and frozen-worktree audit — 2026-10-31

This section supersedes every provenance, phase-status and proposed-contract
statement above. The earlier sections were written after source inspection in
the original checkout, not the required frozen worktree. They are not evidence
about the frozen tree unless restated here.

### Actual inspected source identity

All source inspection for this audit used an explicit working directory:

```text
/Users/gqadonis/.claude/worktrees/production-readiness-476
```

Observed `pwd` and `git rev-parse HEAD` output:

```text
/Users/gqadonis/.claude/worktrees/production-readiness-476
8e0602eecbd37504e8cdf8945f2f1c5537ff6701
```

The frozen worktree's `.kbd-orchestrator/current-waypoint.json` names
`production-readiness-476-baseline`, with status
`IMPLEMENTED_ACCEPTANCE_PENDING`, stage `verify`, and tested source
`8e0602eecbd37504e8cdf8945f2f1c5537ff6701`. It does **not** name
`identity-pass-through` or report 144/170 implementation items. This task
remained source-only despite the waypoint's phase status: no acceptance command
was run.

The mistaken original-checkout facts were its separate waypoint identity
(`identity-pass-through`, 144/170) and any source conclusion presented without
the frozen worktree identity. The original source surfaces inspected before the
correction included its `AGENTS.md`, waypoint, plan, `src/llm/{router,
orchestrator,liter_driver,mod}.rs`, `src/llm/{registry,catalog}.rs`, and
`src/uar/runtime/manager.rs`. No original private credential was read and no
original checkout file was written. The design below is re-grounded against the
frozen path; it does not claim the two checkouts are equivalent.

### Frozen-source findings and changed conclusion

The frozen source confirms these seams:

- `ModelRouter::route_with_preferred_model` first forms runtime candidates from
  registered enabled providers/models and health, then supplements with catalog
  entries whenever a provider is configured. That remains the disabled-model
  re-entry problem.
- `DestinationRequestPreparations::prepare` installs exact template, settings
  and budget state before `prepare_user_attempt` inserts `response_format`.
  The retry closure and each fallback call that method again before
  `open_driver_stream`.
- `DestinationBudgetDriver` currently injects only budget contract and output
  cap, then directly calls `inner.stream`; it is the graph bypass to close.
- `LiterLlmDriver::from_endpoint_profile` builds the controlled compatible
  client but passes it through `from_client`, which initializes
  `generic_chat_adapter` to `false`. `from_client` is genuinely opaque;
  `with_bound_model` preserves its boolean. A profile-controlled constructor
  must explicitly retain its successful generic-compatible construction fact.

One material correction was found in the frozen `LiterLlmDriver::stream` path.
The emitted final-provider-wire receipt currently sets:

```rust
"fit_guarantee": req.budget_contract.is_some(),
```

That is false as a real-image guarantee when the final request contains image
parts: `SyntheticOpenAiCompatibleChatV1` establishes the serialized wire count,
not a provider's image-token accounting. The provider implementation must
preserve final-wire counting and overflow enforcement, but set this receipt's
fit guarantee to false whenever the final request contains an image unless a
separate destination-specific image-accounting contract exists. No such
contract was found in the inspected frozen provider seams. `count_quality` may
continue to describe the serialized representation; it must not be interpreted
as semantic image-fit evidence.

### Final coordinated contract

This replaces the earlier `Vec` scope proposal. `BTreeSet` is correct because
it answers membership only; the existing `Orchestrator::fallback_drivers` vec
continues to own fallback attempt order.

```rust
// src/llm/router.rs — provider-owned, crate-private API
use std::{collections::BTreeSet, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationScope {
    Exact { primary: String },
    PrimaryWithFallbacks {
        primary: String,
        fallbacks: BTreeSet<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationAdmissionError {
    MalformedDestination,
    OutsideCapturedScope,
    RuntimeMetadataUnavailable,
    ProviderDisabled,
    ModelDisabled,
    ProviderInCooldown,
    MissingRequiredCapability { capability: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdmittedDestination {
    pub(crate) destination: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DestinationAdmission {
    scope: DestinationScope,
    router: ModelRouter,
}

impl DestinationAdmission {
    pub(crate) fn new(scope: DestinationScope, registry: Arc<ProviderRegistry>) -> Self;

    pub(crate) async fn admit(
        &self,
        destination: &str,
        requirements: &RouteRequirements,
    ) -> Result<AdmittedDestination, DestinationAdmissionError>;
}
```

The runtime writer constructs no unrestricted/unbound scope. It constructs this
scope only from private `RunModelBindings` after actual identities are captured:
`target_selection` produces `Exact { primary }`; otherwise it produces
`PrimaryWithFallbacks { primary, actual captured fallback model IDs }`. It must
not reconstruct scope from ambient configuration or credential grants. The
provider writer owns the type and `new`/`admit`; runtime owns construction and
injection. This is the single agreed interface with the runtime-design writer.

`admit` is called after final request preparation and response-format insertion
for every primary retry and fallback, and after the graph wrapper applies its
current contract/output cap. It receives freshly derived requirements from the
actual `LlmRequest`; requirements are never captured in `DestinationScope`.
It checks literal `provider/model` membership, live registered
provider/model enabled state, live cooldown and the required final controls.
It does not select a model, acquire credentials, expand authority or alter
fallback order. `detect_provider` and a profile `wire_model` remain adapter
details and must not replace literal destination identity.

For this bounded work, `ProviderConfig`/`ModelConfig` are the existing
operator-declared runtime metadata used by the evaluator. `ModelCatalog` stays
discovery/ranking metadata and cannot resurrect a disabled registered model or
satisfy a hard capability for a destination lacking matching runtime model
metadata. This distinguishes current operator declaration from the separate
wire-path evidence without adding a public general allow/deny API or a global
capability-provenance redesign solely because `supports_tools` defaults true.

The existing `generic_chat_adapter` boolean is the narrow Liter wire-path
evidence. The provider writer should retain that existing field, set it true in
the controlled successful `from_endpoint_profile` compatible branch, keep it
false for `from_client`, and preserve it through `with_bound_model`. This
corrects the frozen source's profile-path mismatch without treating aliases or
catalog metadata as adapter certification.

### Audit boundary

Commands in this correction were source inspection only: `pwd`, `git rev-parse
HEAD`, `sed`, and `rg`, each invoked with the frozen worktree as explicit
working directory. No test, lint, formatter, Cargo command, compiler probe,
build, source write, dependency change, vendor edit, external mutation or paid
call ran. This `/tmp` documentation update is not source implementation or
acceptance evidence. The corrected capability admission, fit-guarantee behavior,
actual adapter selection, fallback/cancellation behavior and tool identity
preservation remain unverified.
