# Provider capability-dispatch handoff

Phase: `production-readiness-476-capability-dispatch` / Execute. The whole
phase verification gate is **CLOSED**. This handoff records source changes and
interfaces only; it is not acceptance evidence.

## Changed paths

- `src/llm/router.rs`
  - Adds crate-visible immutable `DestinationScope::{Exact,
    PrimaryWithFallbacks}` backed by literal model identities and a `BTreeSet`
    of fallback membership only.
  - Adds `DestinationAdmission`, created from captured scope plus an optional
    live `ProviderRegistry`. `without_registry` is still scoped and admits
    only no-hard-capability requests; it never acts as an unrestricted grant.
  - Adds live send-boundary evaluation of literal provider/model registration,
    enabled state, cooldown, declared capability, context and catalog-priced
    cost constraints. Registered disablement now vetoes catalog re-entry. The
    same live enabled model metadata supplies an optional downward-only output
    bound for the admitted literal destination.
  - Keeps catalog information as routing/ranking metadata. Catalog-only
    candidates cannot satisfy hard capability requirements or a context
    minimum. Any exact registered provider/model tuple is authoritative even
    when its live facts filter it out; catalog supplementation remains only for
    unregistered legacy text discovery.
  - Adds final-request derivation for images across all final messages, actual
    tools or required/named tool choice, effective structured format, a trusted
    explicit enabled-reasoning input, and the final budget context size. Provider
    dialect defaults, `thinking_config`, and disabled dialect fields do not
    manufacture a hard reasoning requirement; `ReasoningEffort::None` is not
    an enabled reasoning capability demand.
  - Keeps ordinary typed-image provenance private to `DestinationAdmission`.
    The public `RouteRequirements` shape remains advisory and cannot grant an
    adapter admission; the admission instance consults the final image demand.
  - Adds a private captured run-local plaintext builder. It can bypass only a
    final `RuntimeMetadataUnavailable` result for an exact destination whose
    captured credential decision was `ExplicitRequest`; runtime must supply
    only those primary/fallback bindings. The final request must have no
    declared tools, reasoning, image, structured-output, context, or catalog
    cost demand, and the provider health monitor is queried again before send.
    Known provider/model disablement and every other live evaluator refusal
    remain terminal. This preserves admitted private aliases without turning a
    registry miss into a capability claim or allowing server-owned credentials
    to survive removal/revocation.
- `src/llm/orchestrator.rs`
  - Adds `with_destination_admission(Arc<DestinationAdmission>)` and applies
    it after each primary/fallback `prepare_user_attempt` and before
    `open_driver_stream`.
  - Adds `with_finite_accounting_requires_proven_image_bound(bool)`. When the
    trusted host selects it, every final image-bearing attempt is refused before
    admission or stream creation because the provider has no real image-token
    bound; text and no-image synthetic requests preserve existing behavior.
  - Uses the runtime-owned canonical message projector for both prepared and
    canonical provider messages. That projector retains only validated inline
    image URLs while continuing to scrub text and tool arguments.
  - Replaces the observed ordinary request debug payload (`?req.messages`) with
    `message_count` and `tool_count`; JSON message values can contain raw data
    URLs, so `ImageUrl`'s custom `Debug` cannot protect that boundary.
  - Makes admission rejection a terminal pre-send primary failure; it does not
    cool down the provider or manufacture fallback authority. Rejected
    fallbacks are skipped without health failure accounting.
  - Applies `AdmittedDestination::restrict_output` after each prepared final
    admission and before manifest/stream creation, including the non-streaming
    driver path. It takes the minimum of a live registered model output limit,
    any request cap, and any existing destination contract output limit; it
    never creates, replaces, or raises a contract. This fixes ordinary fallback
    output restriction only, not image token accounting or fit certification.
- `src/llm/liter_driver.rs`
  - Records generic-compatible adapter evidence only for the controlled,
    successful endpoint-profile construction branch, whose constant Liter hint
    selects the generic path independently of provider/model spelling.
    `new` and `from_endpoint_profile` set the fact only after their pinned
    `DefaultClient::new` branch succeeds. `from_client` remains opaque/false
    and `with_bound_model` preserves the fact.
    `LlmDriver::supports_admitted_inline_images()` therefore exposes captured
    generic adapter construction for both successful constructors; it is not
    catalog metadata or a provider-name capability certification.
  - Removes the leaf-wide inline-image adapter refusal so established native
    Anthropic/Liter conversion paths remain intact. The new ordinary typed path
    is instead admitted through the host-captured destination identity set.
  - Keeps final-wire counting but reports `fit_guarantee: false` for image
    requests because no destination-specific real image-token accounting
    contract exists.
- `src/llm/mod.rs`
  - Exports ingress-owned `inline_image` as crate-visible.
  - Replaces `ImageUrl` derived `Debug` with a redacted representation: URL
    kind, length and SHA-256 only; detail is normalized to known values or an
    unsupported marker. It never renders raw inline bytes, URLs, or arbitrary
    detail text. The formatter uses the bound formatter argument and renders
    the digest with the repository's byte-wise hexadecimal idiom.

## Runtime interface

```rust
DestinationScope::Exact { primary: String }
DestinationScope::PrimaryWithFallbacks {
    primary: String,
    fallbacks: BTreeSet<String>,
}

DestinationAdmission::new(scope, admitted_inline_image_destinations, Arc<ProviderRegistry>)
DestinationAdmission::without_registry(scope, admitted_inline_image_destinations)
DestinationAdmission::with_admitted_inline_image_adapter_required(bool)
DestinationAdmission::with_admitted_run_local_plaintext_destinations(BTreeSet<String>)
DestinationAdmission::admit(destination, &RouteRequirements).await
AdmittedDestination::restrict_output(&mut LlmRequest)
Orchestrator::with_destination_admission(Arc<DestinationAdmission>)
Orchestrator::with_finite_accounting_requires_proven_image_bound(bool)
requirements_for_final_request_with_explicit_reasoning(&LlmRequest, bool)
```

`AdmittedDestination` intentionally carries no duplicated destination string;
its only send-boundary result is the optional live output restriction.

The runtime must construct scope solely from captured `RunModelBindings`, then
inject the same `Arc<DestinationAdmission>` into the orchestrator and graph
budget wrapper. It must pass the immutable `BTreeSet<String>` returned by
`RunModelBindings::admitted_inline_image_destinations()` to each admission
constructor. For final graph admission it must call the explicit-reasoning
helper with `reasoning_effort.is_some_and(|effort| effort.thinking_budget().is_some())`.
It must chain
`with_admitted_inline_image_adapter_required(canonical_typed_image_route)` on
the shared admission while constructing it. The admission applies the marker
only if the final request has images. Provider final-request derivation is
intentionally after destination preparation and is not a substitute for
advisory preselection requirements.

For the registry-present run-local compatibility path, runtime must additionally
derive `BTreeSet<String>` only from the captured primary/fallback `BoundModel`s
whose preserved `CredentialSourceDecision::source()` is
`CredentialSource::ExplicitRequest`, then chain
`with_admitted_run_local_plaintext_destinations(...)`. It must not infer this
authority from `run_credentials.is_some()`, catalog aliases, or a reconstructed
connection. The set only makes metadata-absent, plain-text sends eligible for
the provider-owned second health check; it grants no model capabilities,
context, cost, output limit, fallback, or credential authority.

## Unresolved / unverified

- No test, formatter, linter, Cargo command, compiler probe, build, container
  action, or external call ran in this repair cohort.
- A prior pre-commit minimal-library compile attempt stopped before later
  profiles or acceptance steps. Its log identified the now-repaired provider
  digest formatting error and redundant test `Arc` import; it also reported an
  unrelated moved-value error in `src/uar/runtime/manager.rs`. No successful
  verification result is claimed, and no follow-up command ran in this cohort.
- Authored but did not execute
  `compare_candidates_tests::disabled_runtime_catalog_model_is_not_reentered_or_preferred`.
  It selects a real `ModelCatalog::global().models_with_capabilities` entry,
  registers that exact identity with `ModelConfig::enabled = false`, and asserts
  both ordinary routing and a matching preferred destination return no route.
  This is supplemental component coverage for the disabled-runtime-model
  catalog-reentry defect, not a substitute for the public mounted scenario.
- Authored but did not execute
  `compare_candidates_tests::registered_context_limit_is_not_reentered_from_catalog_or_preference`.
  It selects a real catalog entry with at least 8192 context tokens, registers
  the same identity enabled at 4096, and asserts ordinary and preferred routing
  return no route. This covers the source-only advisory context re-entry repair
  without changing the public `RouteRequirements` or no-registry admission.
- Compilation, fallback ordering, graph-wrapper enforcement, cancellation,
  tool identity, mounted HTTP behavior, and raw-image diagnostic absence remain
  unverified pending the lead's complete nine-item implementation inventory and
  the separately authorized bounded phase-acceptance gate.
- A serialized synthetic wire count remains evidence only for that synthetic
  representation. `RequestBudgetContract::synthetic_exact(base_url)` is the
  only public strict contract source and intentionally permits existing fixture
  limit/profile mutations; it is not a real-image certification. There is no
  public real destination image-token accounting contract or strict real-image
  fit guarantee in this provider slice. Runtime owns graph-send finite-policy
  refusal and must pass its existing finite-envelope predicate to the ordinary
  orchestrator builder; the provider receipt reports
  `fit_guarantee: false` whenever image parts are present.
- Checkpoint resume has no persisted typed-ingress provenance marker. Runtime
  must not infer it from image bytes; this route remains an unverified
  prerequisite until a host-persisted marker is allocated.
- CD06 fallback-specific output ceilings do not come from
  `ModelConfig::max_output_tokens`: manager applies that field only while
  deriving the primary run-level cap, and `fallback_llm_config` carries no
  ceiling. Final destination admission now takes the existing live registered
  fallback maximum as an additional restriction even when no exact preparation
  is installed. A distinct fallback `DestinationRequestPreparation` remains
  stricter authority when present: its cloned budget contract is reduced by the
  request cap in `DestinationRequestPreparations::prepare`. Neither path
  manufactures a budget contract or proves real image-token fit.
- D1 run-local alias compatibility is source-only and unverified. The provider
  admission builder now expects runtime's exact captured `ExplicitRequest`
  destination set; runtime wiring and behavior through the ordinary/graph
  paths remain pending the closed-gate whole-phase inventory. The exception is
  deliberately narrower than the existing no-registry text path: it refuses
  every final capability, context, or cost demand and consults live cooldown
  state even if the provider alias has no registry metadata.
