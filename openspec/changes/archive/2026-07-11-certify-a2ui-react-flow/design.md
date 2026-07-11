## Context

UAR currently renders a small set of artifact-specific forms in Chat and lets
the A2UI Testing page trigger those artifacts. It does not yet expose a shared,
versioned A2UI message processor or an explicit approved component catalog.
The upstream protocol identifies v0.9.1 as the current production release and
v1.0 as a candidate whose React renderer is not yet stable.

## Decisions

### Production profile targets A2UI v0.9.1

`uar.a2ui/1` accepts the upstream v0.9.1 message lifecycle (`createSurface`,
`updateComponents`, `updateDataModel`, `deleteSurface`) and the
`application/a2ui+json` media type. Every message is declarative data. UAR does
not execute agent-supplied HTML, JavaScript, expressions, URLs, or component
implementations.

### v1.0 is experimental

The v1.0 candidate can be inspected and tested behind an explicit experimental
profile, but it is outside the UAR 1.0 compatibility promise. In particular,
`actionResponse`, action IDs, and `surfaceProperties` do not silently alter the
GA v0.9.1 contract.

### One approved React catalog

Chat and A2UI Testing share one typed renderer and one catalog of locally owned
React components. Structure, data bindings, actions, and progressive updates
are validated before reduction. Unknown components, invalid references,
invalid properties/actions, executable markup, and unsafe URLs fail closed with
a visible diagnostic.

### Stores own protocol state and I/O

The renderer is a pure projection. A store owns surface/component/data state,
ordering, updates, and action progress; services own HTTP. Hooks expose narrow
state and action façades to Chat and Testing.

## Risks / Trade-offs

- A2UI is evolving → pin the production profile and require explicit opt-in for
  candidate behavior.
- A broad catalog increases attack surface → certify a deliberately small
  catalog and reject everything else.
- Progressive streams can reference components not received yet → buffer a
  surface until its root and referenced children are available, then update
  reactively.
- Generic renderers can become visually inconsistent → map only to the existing
  UAR component and token system.

## Verification

Shared fixtures exercise create/update/data/delete, progressive rendering,
actions, invalid inputs, and deterministic replay in Rust and TypeScript. A
real-server browser journey proves run → surface → interaction → response.

## UI/UX routing distillation

The A2UI surface is a dense product tool, so it preserves UAR's existing
restrained token system and familiar controls rather than introducing a new
visual language. UI/UX Pro Max prioritized labeled controls, 44px interaction
targets, visible focus, live announcements, inline recovery, disabled loading
actions, stable keys, and minimal motion. Impeccable audit/critique/harden/polish
identified the unsafe raw-JSON fallback and direct component I/O as the release
blockers; the final detector reports no deterministic anti-pattern findings in
the changed surface. Frontend Design reinforced task-first copy and purposeful
structure. Vercel React Best Practices and Composition Patterns led to a pure,
shared renderer with store-owned I/O, narrow hook façades, stable component
identity, and explicit variants instead of behavioral boolean proliferation.
