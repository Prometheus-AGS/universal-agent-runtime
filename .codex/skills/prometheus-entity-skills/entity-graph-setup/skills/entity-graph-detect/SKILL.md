---
name: entity-graph-detect
description: Static analysis pass to infer EntityType definitions, id fields, and relation hints from TypeScript, Prisma, GraphQL SDL, OpenAPI, and API routes—outputs entity_manifest JSON aligned with shared schemas.
---

# entity-graph-detect

**Detection** sub-skill: scan the codebase and APIs to produce an **`entity_manifest`** consumable by **init** and **migrate** phases.

## When to use

- Unknown or large domain; you need a **machine-readable** inventory before writing `registerSchema`.
- Greenfield documentation for architects (“what would live in the graph?”).

## Inputs

- Repository root with read access to `package.json`, `tsconfig`, `prisma/schema.prisma`, `*.graphql`, `openapi.*`, `app/api/**`, `src/api/**`.

## Workflow

1. Run checklist in `prometheus-entity-skills/entity-graph-setup/references/codebase-analysis.md`.
2. For each resource, emit one object validated against `prometheus-entity-skills/_shared/references/schemas/entity-types.schema.json`.
3. Cross-check **list** vs **detail** field shapes (flag partial list rows).
4. Infer **relations** from FK naming (`userId` → `belongsTo` `User`) — mark confidence **high/medium/low**.
5. Output **`entity_manifest.json`** + human **`detect_summary.md`** (risks, ambiguities).

## Deliverables

- `entity_manifest.json` — array of entity definitions.
- `detect_summary.md` — top ambiguities (e.g. polymorphic ids, union types).
- Optional **`relation-schema.schema.json`-aligned** draft for `registerSchema` (TS functions for `listKeyPrefix` still hand-written in execute phase).

## References

- `prometheus-entity-skills/entity-graph-setup/agents/analyzer.md` — role boundaries.
- `prometheus-entity-skills/_shared/references/schemas/relation-schema.schema.json` — relation JSON hints.

## Caveats

- **Inference ≠ truth** — always confirm against running API responses.
- **custom** `FilterClause` predicates cannot be serialized in JSON manifests; document them in prose.
