## Why

UAR currently persists and renders a partial, UAR-specific content union, so several runtime events disappear from the conversation surface and adding a new variant does not force every projection and renderer to handle it. Wave 4 needs one exhaustive shared `ContentBlock` protocol and complete `Chunk` catalog before the legacy A2UI testing route and the remaining migration work can be retired safely.

## What Changes

- Establish the cross-platform `ContentBlock` wire/storage union and a single exhaustive `toChunks` projection into UAR's richer view union.
- Add the complete §8 chunk catalog, phase mapping, and Assistant UI data-part renderers, including trace-only handling for state, step, usage, error, and unknown raw events.
- Render protocol dividers as spacing with `<div role="separator">`, never as `<hr>`.
- Keep Recharts 3.10.1 as the incumbent chart engine behind a typed, application-owned chart model; model/provider payloads never control component configuration, CSS, or injected markup.
- Remove the dedicated production A2UI testing route and navigation item while preserving the live A2UI renderer, schemas, services, chat parts, and runtime-console protocol state.
- Add compile-time exhaustiveness fixtures, per-catalog stories, focused behavior/security tests, and Wave 4 frontend verification.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `frontend-content-rendering`: Require the shared wire protocol, exhaustive chunk projection and registration, complete render catalog, safe chart/artifact treatment, spacer dividers, and production A2UI surface consolidation.

## Impact

- **Frontend:** chat content types, AG-UI normalization, persisted message projection, Assistant UI data parts, chunk renderers/stories/tests, navigation, and the retired A2UI testing page.
- **Runtime UX:** conversation bubbles expose the full typed runtime catalog while state/raw details remain available through the trace/inspector; the standalone testing page is removed from production navigation.
- **Provider compatibility:** provider-neutral AG-UI and persisted `ContentBlock` inputs remain the boundary; no provider-specific rendering branches or backend route changes are introduced.
- **Realtime state:** known official and custom events normalize to typed chunks, and unknown custom/RAW frames remain inspectable rather than being dropped.
- **Dependencies:** retain exact `recharts` 3.10.1; no new chart dependency is added.
- **Security:** untrusted markdown/SVG/A2UI boundaries remain sanitized or policy-gated, HTML artifacts remain sandboxed, JSON stays text-only, and chart data cannot reach `dangerouslySetInnerHTML` configuration.
- **KBD:** C-12 is recorded in canonical phase state and advances only after Wave 4 verification and OpenSpec archive readiness.
