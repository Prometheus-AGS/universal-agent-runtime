## Context

The frontend currently uses `types/chat-content.ts` as both a persistence shape and a render model. Its nine variants are neither the shared cross-platform `ContentBlock` protocol nor the complete UAR view catalog, and `use-chat-runtime.ts` silently drops variants its switch does not recognize. Existing chat components cover several rich events as pseudo-tool calls, while C-06/C-07 already provide validated AG-UI event projections and durable run-event storage and C-08/C-09 provide the only markdown, Mermaid, Shiki, SVG, and sanitization boundaries.

The binding §8 target separates those concerns: `ContentBlock` is the portable wire/storage union; `Chunk` is UAR's richer render union; one pure projection connects them; official/custom AG-UI events add typed runtime chunks; Assistant UI named data parts register the React presentation. Flutter parity is protected by keeping the portable union exact rather than adding UAR-only wire variants.

Recharts is already pinned at 3.10.1 and used in two frontend modules. Its current chart components enable an accessibility layer by default and support responsive sizing. The local `components/ui/chart.tsx` helper emits CSS from application chart configuration with `dangerouslySetInnerHTML`, so it is not an acceptable boundary for provider/model-controlled configuration.

## Goals / Non-Goals

**Goals:**

- Make the shared portable `ContentBlock` discriminated union exact and exhaustive.
- Define all §8 `Chunk` variants, a single `toChunks` projection, exhaustive phase/visibility maps, and named Assistant UI data-part registration.
- Render every bubble-visible catalog entry with Flat 2.0, accessible controls, safe fallbacks, and existing security boundaries.
- Keep unknown CUSTOM/RAW events durable and trace-visible instead of dropping them.
- Preserve already-delivered C-06 through C-11 behavior and historical persisted messages while migrating the partial local content shape.
- Remove the A2UI testing destination from production while keeping the live renderer and runtime-console views.

**Non-Goals:**

- Adding backend routes, provider branches, or new AG-UI wire event names.
- Replacing the maintained A2UI renderer or the C-08/C-09 markdown/rendering engines.
- Reworking the C-11 run trace or the C-14 admin migration.
- Accepting arbitrary chart component configuration, HTML, CSS, JavaScript, or formatter functions from runtime payloads.

## Decisions

### 1. Separate portable blocks from runtime chunks

Add `shared/content/content-block.ts` with the exact camelCase union from the cross-platform contract. Add `features/chat/model/chunk.ts` for the richer view union and `to-chunks.ts` as the only portable-block projection. Every discriminant switch calls `assertNever`; the renderer registry and phase map are typed with `satisfies Record<ChunkKind, ...>` so a new variant fails compilation at all required seams.

The existing persisted partial variants receive a narrowly scoped decode adapter. Historical rows are converted once at the PGlite read boundary into portable blocks and/or typed runtime chunks; new writes use the canonical shape. This avoids silently invalidating local transcripts while removing the partial union as an authoring API.

Alternative: widen the old union indefinitely. Rejected because it preserves one type with conflicting wire, persistence, and presentation responsibilities and cannot protect Flutter parity.

### 2. Use one deterministic runtime projection

Portable blocks project through `toChunks(blocks, context)`. Validated official/custom events project through `toRuntimeChunk(eventRow)` and share the same `Chunk` union, phase map, visibility map, and renderer registry. Stable ids derive from message/run identity plus sequence/index; time is supplied by projection context rather than read from global state.

Tool-use/result pairs and A2UI/artifact kinds are joined by stable ids during the single pass. Unknown CUSTOM and RAW inputs become `raw` chunks with their payload intact for the inspector and hidden bubble visibility. No normalizer default silently returns nothing.

Alternative: switch directly inside `EnhancedThread`. Rejected because it duplicates normalization, couples transport to presentation, and leaves persistence and trace semantics inconsistent.

### 3. Register rich presentation as Assistant UI data parts

`RichDataRenderers` mounts stable `useAssistantDataUI` registrations for named chunk families. `use-chat-runtime.ts` emits `{ type: "data", name, data: chunk }` for rich chunks while text/reasoning continue using the Assistant UI native parts that own streaming behavior. A typed `ChunkRenderer` remains available for stories and non-Assistant-UI projections and uses the same family components.

State snapshot/delta, step, and raw chunks are intentionally hidden in the bubble and remain visible in C-11 trace/inspector views. Usage renders in the run footer. Errors use a dedicated recovery surface, never assistant prose.

Alternative: encode all rich chunks as pseudo-tool calls. Rejected because tool grouping changes their semantics and prevents named data-part registration.

### 4. Reuse established trust boundaries by MIME/kind

Markdown uses `MarkdownBubble`; Mermaid and code use the existing lazy blocks; SVG uses the shared DOMPurify helper; A2UI uses the maintained default-deny renderer; JSON renders as escaped React text. HTML artifacts render only in a sandboxed iframe without same-origin or script privileges. Unsupported or invalid content falls back to escaped source or a download affordance.

Image data URLs are constructed only for allowlisted image MIME values. Visible images require a useful alt value; absent alt text produces an explicit non-visual fallback instead of an unlabeled image. App-owned paths are not treated as browser URLs.

### 5. Retain Recharts behind an application-owned chart model

Keep exact Recharts 3.10.1 and render a small typed chart payload (`bar`/`line`, labels, finite numeric series) with application-selected tokens, axes, tooltip, responsive sizing, and accessibility layer. Reject/escape malformed chart artifacts into the JSON/source fallback. Do not pass runtime data into `ChartContainer` configuration or any API that injects CSS/markup.

Alternative: replace Recharts. Rejected because the incumbent is current, typed, responsive, accessibility-capable, and already paid for in the bundle; replacement adds dependency and migration cost without an observed gap.

### 6. Keep A2UI live features; gate the tester out of production

The dedicated A2UI testing destination is a development surface. Navigation inventory and route resolution exclude it when `import.meta.env.PROD`; the page remains available in development for real round-trip testing. Live chat A2UI chunks, schemas/store/service, policy-gated renderer, and runtime-console protocol state remain production features.

Alternative: delete the upgraded tester. Rejected because the July proposal's premise was superseded when the page became a real round-trip tool, while the current phase still requires removing it from the production surface.

## Risks / Trade-offs

- **Historical rows use the partial local shape** → Decode only at the PGlite boundary, cover every legacy discriminant, and persist canonical data on subsequent writes.
- **The complete catalog is visually dense** → Default secondary detail to collapsed/quiet surfaces; keep trace-only kinds out of bubbles.
- **Two projections could drift** → Both produce the same `Chunk` union and must satisfy the same phase/visibility/renderer records and exhaustive tests.
- **Chart payloads cross an untrusted boundary** → Parse into a closed schema of strings and finite numbers; application code alone chooses components, tokens, and formatter behavior.
- **Development-only A2UI reachability can leak into production navigation** → Derive navigation, command palette, and route resolution from one environment-filtered inventory and test both modes.

## Migration Plan

1. Land the shared portable type, chunk union, exhaustive projection/maps, and legacy decoder with type and unit tests.
2. Add family renderers and named Assistant UI data-part registration, reusing existing components/security helpers.
3. Move stream reduction and persisted reads to canonical blocks/chunks while retaining historical decode coverage.
4. Gate the A2UI testing destination out of production and update navigation snapshots.
5. Run focused catalog/security tests and cheap frontend gates; then run the full Wave 4 frontend test and production-build sequence.

Rollback is code-only: restore the prior runtime adapter and navigation inventory. The persistence reader remains backward-compatible, and no backend or provider contract changes.

## Open Questions

None. The remaining implementation choices are constrained by the §8 catalog, exact installed dependencies, and existing trust boundaries.
