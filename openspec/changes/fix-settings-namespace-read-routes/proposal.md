## Why

Runtime Settings reads currently interpolate internal namespace keys directly into REST URLs, so provider and underscored namespaces fall through to the backend's generic setting-key route and return misleading 404 errors in the installed UI. The provider data and canonical backend routes are intact; the frontend read transport must use the same slug conversion already used by saves.

## What Changes

- Canonicalize every settings namespace GET through `namespaceToSlug()` before constructing its URL.
- Add focused transport coverage for plural, hyphenated, unchanged, and non-2xx behavior.
- Add a local installed-service browser check on port 1906 that observes provider and Context Management route usage and rejects settings namespace 404s.
- Pin the KBD terminal-run rollover implementation that created this fresh phase and record successor-run continuity in the UAR contract.
- Rebuild and install the unchanged backend plus corrected static frontend through the native macOS installer, preserving provider configuration and IDs.
- Do not add backend aliases or modify persistence, payloads, provider configuration, save behavior, or realtime entity state.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `frontend-configuration-surfaces`: Require internal settings namespace keys to resolve to canonical backend slugs for reads while preserving error propagation and runtime UX.
- `kbd-phase-inventory-governance`: Require a causally ordered successor run after terminal lifecycle state before new phase work begins.

## Impact

- Frontend transport: `frontend/src/features/settings/api/settings-api.ts` and a focused adjacent test.
- Installed runtime UX: Provider Overrides and Context Management load configured state without misleading not-found banners; provider compatibility and realtime state contracts remain unchanged.
- Local verification: a port-1906 Playwright configuration/spec plus existing frontend, bundle, OpenSpec, and locked Rust release gates.
- KBD state: the new run/phase remains canonical and the `crates/prometheus-skill-system` gitlink moves to the pushed rollover review commit.
- Deployment: `packaging/native/macos/install.sh` installs the rebuilt local candidate without changing configuration or provider data.
