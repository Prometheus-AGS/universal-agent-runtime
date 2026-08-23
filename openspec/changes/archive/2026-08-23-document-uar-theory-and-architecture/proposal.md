## Why

The current architecture page is a useful diagram but not the conceptual spine
needed to understand why UAR exists, where authority lives, how execution moves
through the runtime, or which claims apply to each build profile. The branded
portal now needs a source-grounded architecture narrative before workflow,
security, API, and history guides can link to one stable authority.

## What Changes

- Replace the single architecture overview with a navigable conceptual section
  covering UAR's problem statement, runtime theory, capability inversion,
  agent-versus-host trust boundary, turn/execution lifecycle, event flow,
  persistence, delegation, and protocol boundaries.
- Document `server-full`, `minimal`, and `embedded-mobile` as separate capability
  and evidence profiles; never transfer a claim silently between them.
- Add source-grounded Mermaid diagrams and concise cross-links that later product,
  security, API, SDK, history, and testing guides can treat as current authority.
- Preserve uncertainty explicitly: present-tense behavior must trace to current
  source, canonical OpenSpec, or observed behavior, and unsupported or planned
  architecture must not be described as delivered.
- Extend the local documentation controls so required architecture routes,
  provenance, profile limits, and source references fail closed when omitted.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dev-portal-2026`: Require a source-grounded public architecture section that
  explains UAR's theory, trust and execution boundaries, lifecycle, protocols,
  persistence, delegation, and profile-specific limits.

## Impact

- **Documentation:** architecture guides under `website/docs/architecture/`,
  their category metadata, navigation, publication routes, and bounded local
  documentation controls.
- **Runtime UX:** no runtime or React behavior changes; the portal explains the
  existing operator/runtime boundary and links readers to current UI concepts.
- **Provider compatibility:** no provider or model integration changes; the
  architecture narrative documents provider access only at the runtime boundary
  and defers provider-specific workflows to the next content change.
- **Realtime state:** no event or transport changes; the guide documents the
  current normalized-event and SSE boundaries without broadening their claims.
- **Dependencies and APIs:** no dependency, public API, storage schema, or build
  profile changes.
- **KBD:** transition the registered change through Execute after bounded source
  evidence passes, then advance the exact next command to
  `document-inference-skills-knowledge-and-agents`.
