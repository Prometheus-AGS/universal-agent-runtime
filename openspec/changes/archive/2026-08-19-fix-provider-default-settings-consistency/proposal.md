## Why

The supported `local` memory embedding provider is rejected by the canonical settings schema, leaving settings only partially initialized. A subsequent default-provider request can then fail persistence after changing the live registry, so the API reports failure while runtime routing has already changed.

## What Changes

- Accept `local` wherever the memory embedding provider is validated while continuing to reject unknown values.
- When settings persistence is configured, make default-provider selection persistence-consistent: validate the target, persist the selection, and publish it to the live provider registry only after persistence succeeds. Preserve the existing registry-only behavior when no settings manager is configured.
- Add focused positive and negative controls for settings initialization, missing providers, persistence failure, and durable default-provider round trips.
- Preserve the existing HTTP contracts, runtime UX, provider API shape, and realtime state surfaces; this change corrects the state they report rather than adding a new surface.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `provider-model-settings-certification`: Require supported provider values to remain consistent across resolved configuration and settings validation, and require a rejected default-provider write to leave durable and live routing state unchanged.

## Impact

- Affects the host-side settings schema, provider default-selection handler, and focused Rust regression tests.
- Restores compatibility with the existing `local` memory embedding profile without changing provider interfaces or adding dependencies.
- Prevents the runtime UX and realtime provider state from displaying a default that a failed request did not persist.
- Requires KBD child execution, evidence, reflection, and handoff updates before control returns to the parent `screen-by-screen-validation` work; it does not change the outer phase goals.
