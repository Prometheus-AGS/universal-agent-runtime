# Design

## Context
The narrow compiler reproducer reports Zod 4 objects crossing core's Zod 3 boundary, two obsolete Catalog calls, and default ComponentApi inference losing the render implementation. versions.toml pins Zod 3.25.76. Selected core 0.12.0 still depends on Zod ^3.25.76 and now requires Catalog(id, protocolVersion, components).

## Goals / Non-Goals
Restore compilation and production bundle, retaining existing A2UI 0.9 protocol and approved catalogs. No UI redesign, provider change, runtime migration, new validation guard or unsafe type assertion.

## Decisions
Restore exact first-party Zod 3.25.76 rather than migrate the authoritative A2UI boundary to Zod 4. Regenerate both existing workspace lock scopes without unrelated dependency upgrades. Add v0.9 catalog constructor arguments and an explicit UarComponentImplementation generic to the existing processor. Preserve renderer state/composition and component identity: Vercel React/Composition guidance applies only to retaining those existing boundaries; no effects, subscriptions or entity state change.

Core API evidence: installed @a2ui/web_core 0.12.0 src/catalog/types.d.ts and processing/message-processor.d.ts. Zod official compatibility reference: https://github.com/colinhacks/zod/blob/v4.6.5/packages/docs/content/library-authors.mdx (Context7).

## Risks / Trade-offs
The uncomfortable limitation is that compilation/bundling alone does not certify live A2UI behavior. Root owns packaged desktop operation. UAR sidecar mode deliberately does not serve the operator SPA; standalone runtime serves generated static/ from disk, not embedded Rust assets.

## Migration Plan
Commit corrected manifests/locks and source together; build static assets locally. Existing source history and immutable release acceptance remain untouched. Rollback is the previous commit, not a reinterpretation of accepted release evidence.
