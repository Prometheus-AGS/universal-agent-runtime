## Context

UAR needs a first-party React renderer for the certified `uar.a2ui/1` profile and its entity extensions. The framework-neutral `@a2ui/web_core` machinery is already vendored through `@prometheus-ags/a2ui-core`; Google’s React renderer remains a behavioral reference rather than a product dependency.

## Goals / Non-Goals

**Goals:**

- Adapt `GenericBinder`, `Catalog`, and surface models into idiomatic React.
- Render the 9 certified baseline components and all 7 UAR entity components.
- Preserve strict wire schemas, reactive bindings, action dispatch, accessibility, and fail-closed unknown-component handling.
- Enforce initial-render `<16ms` and streaming-update p95 `<8ms` in CI.

**Non-Goals:**

- Wire the renderer into a specific product page.
- Implement surface theming or expand the certified baseline beyond its 9 components.
- Migrate the entity-management package’s internal UI architecture; that remains Change 18.

## Decisions

1. **Build directly on `web_core`.** A thin React adapter owns subscriptions through `useSyncExternalStore`; duplicating protocol state machinery would create divergent semantics.
2. **Keep baseline and entity catalogs separate.** `urn:uar:a2ui:catalog:1` remains certification-stable, while clients explicitly opt into `urn:uar:a2ui:catalog:1+entities`.
3. **Use strict Zod schemas for each entity surface.** This keeps client capabilities machine-readable and rejects unknown wire fields. Zod 3 is retained because it is the version compatible with the upstream `web_core` types.
4. **Use shadcn/Base UI and react-aria primitives.** These match the existing design system and supply accessible interaction semantics without adding a competing component framework.
5. **Measure warmed renderer work.** The CI benchmark warms React/happy-dom once, then enforces literal budgets so JS-engine initialization is not misreported as renderer latency.

## Risks / Trade-offs

- **CI timing variance** → keep the fixture deterministic and benchmark package-local; trend reporting can supplement the hard gate later.
- **Nested binder types are shallower than runtime resolution** → localize conversions in `resolvedText` and `resolvedAction` instead of spreading casts.
- **Extension catalog interoperability** → require explicit catalog IDs and strict schemas so unsupported clients fail closed.

## Migration Plan

Publish the package and catalogs without changing existing product surfaces. Consumers can opt in by selecting the entity catalog. Rollback consists of removing that opt-in; the certified baseline catalog is unchanged.

## Open Questions

None for Change 17. Product-page integration and full entity-management migration are separately scoped.
