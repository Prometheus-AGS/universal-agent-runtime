# Backfill the effective-run-policy model route from the registry default

## Why

Built-in agents (`default-agent`, `orchestrator-agent`) intentionally seed an
empty `policy.provider.default` — they defer to the provider-registry default
rather than pin a model name that goes stale on catalog changes (see
`defaults::default_agent`). Consequently `RunManager` resolved an
`EffectiveRunPolicy` whose `model` was `Some(ModelRoute)` with **empty**
`provider_id`/`model_id` (or, for a truly unset chain, `None`). The
`effective_run_policy` artifact then shipped a blank model.

Downstream provenance surfaces read that artifact to label each assistant turn
with which agent/provider/model produced it. With an empty route, first-party
hosts rendered only the agent id — the on-device embedded runtime, whose real
executing model is the registry-default local provider (e.g. an MLX model on
iOS/macOS), showed a blank provider/model on every response. Operators could not
tell which provider/model actually answered, defeating side-by-side comparison.

The runtime already knows the true executing route: `resolve_default_model()`
returns the provider-registry default (the on-device provider on embedded
builds) or the configured `llm_config.model` (service builds). The artifact must
report it (Base Rules 13/38 — surface real state; don't emit a blank the UI has
to guess about).

## What Changes

- In `RunManager::start_run_with_policy`, after the effective policy is resolved
  (whether supplied by the control plane or resolved in-process), **backfill
  `EffectiveRunPolicy.model`** from `resolve_default_model()` when the resolved
  route is missing or has an empty `provider_id`/`model_id`. A route that is
  already fully specified (agent/conversation/global set a real model) is left
  untouched, so precedence is unchanged.
- The emitted `effective_run_policy` artifact now carries the real executing
  `(provider_id, model_id)` on every deployment mode, so the provenance chip
  renders `agent · provider/model` for both local and cloud routes.

No consumer changes: the KnowMe Flutter `_ProvenanceChip` and React
`provenanceLabel` already render agent · provider/model (mapping the on-device
provider to "On-device") and omit missing parts.

## Impact

- Affected specs: `run-policy-resolution` (extended)
- Affected code: `src/uar/runtime/manager.rs` (`start_run_with_policy`)
- Behavior: additive — only fills a previously-blank model route; a fully
  resolved route is never overwritten. Verified device-side on iPhone (release
  build): the embedded-UAR Orchestrator bubble now reports the on-device model.
