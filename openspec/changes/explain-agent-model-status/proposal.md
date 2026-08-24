## Why

The Admin Agents list displays an amber unresolved-model warning for agents that successfully inherit the configured system default. The status also lacks an explanation on hover or keyboard focus, so operators cannot distinguish a real routing failure from normal default inheritance.

## What Changes

- Hydrate the normalized provider metadata with the default provider and that provider's default model before resolving agent status.
- Distinguish loading and provider-registry failures from a confirmed unresolved route instead of presenting every incomplete projection as the same warning.
- Add hover- and focus-triggered status explanations: inherited routes name the effective provider/model, while genuine warnings explain how to resolve the missing route.
- Add focused regression coverage for provider-default projection and status-tooltip behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `admin-agent-model-status-indicator`: Make every non-explicit status truthful from loaded provider metadata and explain it on hover and keyboard focus.

## Impact

- Frontend provider projection and typed provider-default hooks.
- Admin Agents list status rendering and accessibility behavior.
- No backend API, provider compatibility, dependency, or realtime event contract changes.
- The active KBD phase retains ownership; progress is updated only through its canonical workflow, not by editing generated waypoint projections.
