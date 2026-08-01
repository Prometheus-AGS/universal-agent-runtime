# 14. Backfill the effective-run-policy model route from the registry default

Date: 2026-07-24

## Status

Accepted

## Context

Built-in agents (`default-agent`, `orchestrator-agent`) seed an empty
`policy.provider.default` on purpose: they defer to the provider-registry default
rather than pin a model name that goes stale the moment a provider's catalog
changes, and `seed_builtin_agents` re-seeds the value on every restart (see
`defaults::default_agent`). As a result, `RunManager` resolved an
`EffectiveRunPolicy` whose `model` carried empty `provider_id`/`model_id` (or, for
a fully-unset chain, `None`), and the emitted `effective_run_policy` artifact
shipped a blank model.

That artifact is the single source of per-run provenance — first-party hosts
project it into an "agent · provider/model" chip under each assistant turn
(ADR 0013 established the embedded run-policy resolution + admin surface these
hosts consume). With an empty route, the chip could only show the agent id. On
the embedded, in-process runtime the real executing model is the registry-default
local provider — an on-device MLX model on iOS/macOS — so every embedded response
displayed a blank provider/model, and an operator could not tell which
provider/model actually produced a turn. That defeats the side-by-side comparison
the provenance chip exists for.

The runtime already knows the true executing route. `resolve_default_model()`
returns the provider-registry default (set via `with_provider_registry` +
`set_default` when the embedded runtime is built — `src/embedded.rs`) or, absent a
registry entry, the configured `llm_config.model`. The gap was purely that this
value was never written back onto the policy the artifact serializes.

## Decision

Backfill the resolved model route in `RunManager::start_run_with_policy`, at the
one point every run passes through, after the effective policy is obtained
(whether supplied pre-resolved by the control plane or resolved in-process):

- If `EffectiveRunPolicy.model` is `None`, or its `provider_id`/`model_id` is
  empty after trimming, replace it with the `ModelRoute` from
  `resolve_default_model()`.
- If the route already names a non-empty provider and model — i.e. the agent,
  conversation, or global scope resolved a real model — leave it untouched.

The `effective_run_policy` artifact then reports the model that will actually
execute, on every deployment mode:

- **Embedded** (mobile, in-process desktop) → the on-device local provider +
  model (the registry default).
- **Service** → the configured `llm_config.model`.

## Consequences

- The provenance chip renders `agent · provider/model` for both local and cloud
  routes on embedded hosts, where it previously showed only the agent. No
  consumer change was required — the KnowMe Flutter `_ProvenanceChip` and React
  `provenanceLabel` already render provider/model and map the on-device provider
  to "On-device".
- Precedence is preserved: a fully resolved route from any scope is never
  overwritten; the backfill only fills a route that was going to be blank anyway.
- The backfill sits above `resolve_default_model`, so no policy-resolution
  precedence code changed; the service and embedded resolvers still share the
  transport-free core from ADR 0013.
- Verified by `effective_run_policy_artifact_backfills_the_registry_default_model`
  (embedded lib test) and device-side on iPhone (release build, detached): the
  embedded-UAR Orchestrator bubble reports the on-device model.
