# Capability dispatch runtime handoff

Phase: `production-readiness-476-capability-dispatch` Execute. Verification is
**CLOSED**. No test, lint, formatter, typecheck, Cargo command, compiler probe,
build, package command, review gate, or `git diff --check` was run.

## Runtime-owned changes

- `src/uar/runtime/turn/request.rs`
  - Added crate-private `current_user_content: Option<MessageContent>`.
  - Added `with_admitted_current_user_content`, which retains typed admitted
    content and sets `input` to the complete ordered text projection. Text
    parts join with a newline; image parts never enter `input`; image-only
    input remains `Some("")`.
  - `new` initializes the added field to `None`, preserving text constructors.
- `src/uar/runtime/manager.rs`
  - Initializes the sole checkpoint request literal and destructures the new
    field.
  - Adds a digest-or-`none` current-content value to checkpoint continuation
    identity, never raw inline bytes.
  - Constructs one canonical typed user `Message` and uses it for initial,
    retained warm-session, and restored/inherited-history append paths.
  - Uses only projected text for existing matching, retrieval, guardrail, and
    run-context paths.
  - Derives preselection hard needs from canonical retained/current images,
    actual admitted client/native/MCP tool declarations, resolved response
    format, and explicitly enabled reasoning only. The native visibility bit
    is captured from the already filtered descriptor set before it transfers
    into activation; MCP visibility is read from activation's current
    projected descriptor snapshot after activation. `Some(ReasoningEffort::None)`
    preserves the host's no-reasoning selection without creating a hard
    reasoning requirement. Hard no-match and no-registry paths refuse before
    captured client construction instead of falling back to a preferred
    configuration.
  - Creates `DestinationScope` solely from captured `RunModelBindings`, then
    captures the eligible ordinary-inline destinations only from primary and
    fallback driver construction facts, and injects the same scoped
    `DestinationAdmission` into ordinary orchestration and the graph
    `DestinationBudgetDriver`. The private ordinary typed-ingress provenance
    is attached to that admission whenever `current_user_content` is present,
    not to public route requirements; this covers retained images on a later
    text-only warm turn. Graph admission occurs after its captured
    budget/output cap and before the inner stream. The admitted destination's
    private `restrict_output` restriction is applied at that point, before the
    inner driver and cost accounting: it only min-clamps the existing final
    request cap and an already present destination budget contract; it never
    creates, replaces, or increases a contract.
  - Captures a separate private plaintext-destination set only from bindings
    whose existing credential decision is `ExplicitRequest`, then passes it to
    both destination-admission constructors. It carries no catalog capability,
    cost, or output fact; provider admission may use it only for the exact
    captured identity and a final request with no tools, reasoning, images,
    structure, context, or cost requirement.
  - Both ordinary and graph final paths reject an image-bearing request before
    dispatch when the captured model budget has a finite accounting envelope:
    no existing final-wire contract proves real provider image-token input
    accounting. Text-only and synthetic-contract non-image paths remain
    unchanged.
  - Treats a nonempty `artifact.policy.provider.fallbacks` list as the
    captured ordinary-run fallback authority before `RunModelBindings` client
    capture. It retains global retry/health settings but replaces global
    fallback destinations with that list, preventing failure-scope escape.
    An empty artifact list retains the existing global operator failover
    behavior; exact target selection remains fallback-free.
  - Resolves every non-team, artifact-authorized fallback through the provider
    registry before credential binding when no run credential supplies that
    connection. It passes the resulting `LlmConfig` together with its
    credential decision into `RunModelBindings`. The captured driver therefore
    uses that fallback's own endpoint, wire model, protocol, and catalog
    pricing identity rather than rebuilding from the primary configuration. A
    registry-disabled, cooldown, or otherwise unavailable optional artifact
    fallback is removed from the captured failover list before its
    credential/driver capture, preserving a healthy primary; credential-policy
    refusals still fail closed. A run credential keeps its existing
    registry-independent provider/model alias path and supplies connection
    authority before capture. Legacy global fallback endpoint/key choices
    retain their existing `FallbackModel` construction and do not inherit the
    primary's catalog price. Inherited runs continue to reuse their existing
    bindings.
- `src/uar/runtime/turn/bindings.rs`
  - Exposes only captured primary/fallback identities through runtime-private
    `primary_model`, `destination_scope`, and
    `admitted_inline_image_destinations`; it does not expose credential grants
    or query global state.
  - Accepts the root-host captured fallback config and credential decision as
    one private map. Driver capture no longer resolves or reconstructs an
    ordinary fallback from the primary config; it retains an admitted
    fallback catalog pricing identity when one exists.
  - Exposes `admitted_run_local_plaintext_destinations` only for captured
    `CredentialSource::ExplicitRequest` bindings; it never queries a registry
    or turns a credential decision into a capability grant.
- `src/uar/runtime/turn/host/event_projection.rs`
  - Adds canonical-history typed/JSON projectors. They continue to redact text
    and tool arguments, while preserving a URL only when ingress's
    `inline_image_metadata` accepts the same strict bounded syntax. A legacy
    or unsupported URL follows the preexisting generic URL redaction path;
    generic state, diagnostics, and events remain protected.
- `src/uar/runtime/checkpoint_host.rs`
  - Keeps generic secret projection for `GraphState.data` and applies the
    canonical-message projector only to `GraphState.messages` before the
    existing checkpoint protection/hash flow.

## Coordinated interfaces consumed

- Ingress owns `src/llm/inline_image.rs`; runtime uses only its crate-private
  `inline_image_metadata(&ImageUrl)` syntax predicate during protected replay.
  Runtime does not call the decoder here.
- Provider owns `DestinationScope`, `DestinationAdmission`, final-request
  requirement derivation, and `Orchestrator::with_destination_admission`.
  Runtime uses
  `DestinationAdmission::{new(scope, admitted_inline_image_destinations, registry),
  without_registry(scope, admitted_inline_image_destinations)}` and chains the
  private `with_admitted_inline_image_adapter_required` marker. Final request
  requirements use `requirements_for_final_request_with_explicit_reasoning`;
  runtime passes only the host's explicit reasoning control. Provider's
  ordinary builder separately receives the finite-accounting proof flag. Its
  `AdmittedDestination::restrict_output(&mut LlmRequest)` supplies an optional
  cap from the enabled registered final destination only; absent registry/model
  caps leave the existing request untouched.

## Unresolved limits and verification

- Source inspection only; compilation and behavior are unverified.
- Repair cohort source basis: `ActivationContext::{new,new_projected}` takes
  ownership of `native_descriptors`, so preselection retains only its
  pre-transfer nonhidden visibility bit. The later `mcp_descriptors()` call
  returns the active projected MCP snapshot rather than inferring tools from
  registry presence. The recorded pre-commit attempt is at
  `target/production-readiness-476/capability-commit-001.log`, mirrored in
  `.kbd-orchestrator/phases/production-readiness-476-capability-dispatch/evidence/compile-attempt-001.txt`;
  it reports the original E0382 at this call site and a separate provider-owned
  E0277. This runtime agent did not run that attempt.
- The fallback authority finding is source-based: `ProviderPolicy.fallbacks`
  is a defaulted vector, current child rebinding treats its nonempty members as
  concrete fallback authority, and host credential admission validates every
  listed fallback provider. The observed ordinary-run branch previously used
  global fallback destinations whenever credentials/collaboration were absent.
- Destination-binding repair source basis: `Orchestrator::fallback_llm_config`
  deliberately clears `resolved_provider_id`, `base_url`, and `api_key` from
  its primary clone. The prior ordinary binding path discarded the config
  returned by `resolve_credential_binding` and rebuilt that fallback clone,
  while team profiles alone resolved a fallback connection. The new private
  config/decision map retains the host-resolved artifact fallback through
  client construction; `ProviderRegistry::resolve_to_llm_config` returns no
  config for disabled providers, cooldowns, and unavailable models, so the
  manager now removes only those optional entries before capture rather than
  aborting a healthy primary. Run-local credentials intentionally select their
  own validated connection before that registry-only path, so an authorized
  private provider alias is not pruned. No provider API or public
  configuration schema changed.
- Destination-admission source assessment: a captured explicit run-credential
  alias can be absent from a concurrently present global registry, while final
  admission otherwise requires registry provider/model metadata even for a
  plaintext send. The private captured set above is deliberately narrower than
  scope membership. Provider admission must preserve every known disabled,
  cooldown, and model-disabled veto and refuse all capability/context/cost
  requirements before it can admit the metadata-missing plaintext alias.
- CD06's ordinary configured-fallback output restriction is source-based. The
  primary run-level `ModelConfig.max_output_tokens` is captured before
  failover, while ordinary production currently does not install a per-model
  `DestinationRequestPreparations` profile. The private provider admission
  result now imposes a final additional-only cap for both ordinary and graph
  sends, preserving existing exact profiles without synthetic public profile
  injection.
- The phase remains incomplete until all nine ledger items, fixtures, docs,
  settings, and lead inventory are complete. The lead owns any later serial
  gate authorization.
- This slice preserves admitted bytes through canonical protected history. It
  does not certify provider wire delivery, image understanding, native
  Anthropic behavior, checkpoint recovery on unsupported persistence, or
  public/container qualification. A resumed checkpoint currently has no
  persisted ordinary-typed-image route marker; its adapter provenance therefore
  remains an integration prerequisite rather than inferred from retained bytes.
