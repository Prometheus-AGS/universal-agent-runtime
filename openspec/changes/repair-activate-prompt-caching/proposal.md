## Why

The Prompt Caching settings surface currently calls an unregistered namespace and renders editable fallback values after the resulting 404. The runtime also resolves a caching flag without consistently applying it, so operators cannot trust that configuration changes affect Anthropic requests.

## What Changes

- Add a durable, admin-controlled global prompt-caching setting with a safe default of disabled.
- Add owner-scoped session and authenticated-user overrides with explicit inheritance and deterministic precedence.
- Propagate the resolved setting through every policy-bearing Anthropic request while leaving OpenAI provider-managed caching unchanged.
- Replace missing session-configuration 404 responses with an explicit empty response and preserve owner isolation.
- Refine the settings and Session Configuration interfaces so only authoritative state is editable and failures are recoverable and accessible.
- Add operator documentation for configuration, provider behavior, observability, cost, durability, and troubleshooting.
- Update KBD and tracked Prometheus workflow history with the implementation and verification evidence.

## Capabilities

### New Capabilities

- `prompt-caching-control-plane`: Defines global, user, session, and per-request prompt-caching policy, provider behavior, persistence, and runtime propagation.

### Modified Capabilities

- `frontend-configuration-surfaces`: Requires an authoritative and recoverable Prompt Caching settings surface.
- `session-configuration`: Adds a persistent tri-state prompt-caching override and explicit empty-config behavior.
- `customer-documentation`: Adds task-focused prompt-caching configuration and operations guidance.

## Impact

This change affects the Axum settings, user-settings, session-configuration, and compatibility APIs; SettingsManager and configured persistence providers; run-policy resolution and LLM driver dispatch; React settings and entity-backed session configuration; Docusaurus provider documentation; and installed macOS service verification. No new dependency is required. OpenAI request behavior remains provider-managed, and realtime entity state gains only the additive session override field.
