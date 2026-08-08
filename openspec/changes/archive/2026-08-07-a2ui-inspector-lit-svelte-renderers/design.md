## Context

The UAR React renderer owns the certified catalog and `web_core` adapter. Change 22 adds two framework adapters and a diagnostic surface without changing protocol semantics. The primary users are renderer engineers debugging live streams and framework consumers verifying parity. The current base is accessible but lacks a viewable development surface, robust empty/error presentation, and framework-neutral parity assertions.

## Goals / Non-Goals

**Goals:**

- Define one framework-neutral semantic rendering contract over `SurfaceModel`.
- Implement Lit and Svelte adapters that subscribe to `web_core` state and render the certified baseline.
- Provide a dev-only Inspector with synchronized timeline, preview, source, freeze, filtering, connection, and recovery states.
- Expose a Storybook addon registration entry that Change 25 can install.
- Test equivalent roles, accessible names, states, and text across all three renderers.

**Non-Goals:**

- Pixel-identical framework output or shared internal DOM wrappers.
- Production routing, backend changes, or a second A2UI protocol implementation.
- Full Storybook ownership, visual regression, or theming/i18n work assigned to Changes 21 and 25.

## Decisions

1. **Semantic DOM is the parity contract.** Tests compare normalized accessibility-relevant output rather than class names or wrapper structure. Exact DOM equality would couple independent frameworks without improving interoperability.
2. **Every renderer consumes `web_core` models.** Lit uses reactive custom elements and Svelte uses stores/components around `SurfaceModel`; neither reparses protocol messages. A hand-built parallel state machine was rejected because it could drift from React.
3. **Inspector layering follows service → store → hook → component.** The service adapts `EventSource`; the store owns messages, connection, selection, freeze and last-good state; hooks expose selectors/actions; components only render and invoke hooks.
4. **Freeze pauses presentation, not ingestion.** Incoming messages remain counted and buffered so Resume is lossless. A persistent frozen banner shows queued count and snapshot time.
5. **Malformed messages preserve the last-good preview.** The selected raw payload and validation path remain visible beside an error panel. Crashing or blanking the whole tool was rejected because diagnosis is the tool’s purpose.
6. **Storybook integration is an addon entrypoint, not a Storybook installation.** Change 25 remains responsible for Storybook version/configuration and consumes this package’s registration/panel exports.
7. **Dependencies are minimal and verified.** `lit` and `svelte` are package-local runtime/peer dependencies; test compilers stay package-local dev dependencies.

## Risks / Trade-offs

- **Framework behavior drift** → One shared fixture and normalized semantic snapshot suite runs for every renderer.
- **High-volume message streams consume memory** → Store uses a documented bounded history and exposes dropped-message count.
- **Freeze semantics confuse users** → Label it “Freeze preview,” continue connection/status updates, and show queued messages explicitly.
- **SSE payloads contain sensitive runtime data** → Dev-only package, no persistence, redaction hook before display, and no production route.
- **Storybook API changes before Change 25** → Export a narrow addon descriptor/panel component with Storybook packages as optional peers.

## Migration Plan

Add all three packages to the existing pnpm workspace, validate package-local builds/tests, then expose the addon entrypoint for Change 25. Existing React consumers remain unchanged. Rollback removes the packages and addon registration; no protocol or persisted data migration is required.

## Open Questions

None. Change 25 selects the concrete Storybook host version and registration wiring.
