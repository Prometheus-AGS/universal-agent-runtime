---
name: prometheus-entity-skills
description: >
  Bundle index for Agent Skills that teach coding agents how to build and operate apps with
  @prometheus-ags/prometheus-entity-management (normalized Zustand entity graph, hooks, GraphQL,
  CRUD, realtime, Prisma, and performance). Use when working in this monorepo, installing Claude Code
  plugins for entity-graph workflows, or aligning agent docs with the library public API. Load the
  plugin that matches the task (setup, CRUD, GraphQL, realtime, Prisma, optimize); open nested
  SKILL.md files for narrow scopes. Always verify agent references against
  prometheus-entity-skills/_shared/references/library-exports.json after API changes.
license: MIT
metadata:
  bundle: prometheus-entity-management
  library: "@prometheus-ags/prometheus-entity-management"
  spec: "https://agentskills.io/specification"
  progressive_disclosure:
    - "Tier 1: this file + plugin name — pick a plugin"
    - "Tier 2: plugin root SKILL.md — workflow and constraints"
    - "Tier 3: skills/<sub-skill>/SKILL.md — focused playbooks"
    - "Tier 4: references/, agents/, prompts/ — load on demand"
---

# Prometheus entity skills (bundle index)

This directory is the **canonical skill pack** shipped beside the library. It follows the [Agent Skills specification](https://agentskills.io/specification) for leaf skills: each invokable skill lives in its own folder with a **`SKILL.md`** (singular) file, YAML frontmatter (`name`, `description`, …), and optional `scripts/`, `references/`, `assets/`.

**`SKILLS.md` (this file)** is a **bundle catalog** only—it is not a substitute for per-skill `SKILL.md` files. Use it to choose a plugin, map sub-skills, and find shared references.

## Claude Code plugins and marketplace

Each first-level folder under `prometheus-entity-skills/` with `.claude-plugin/plugin.json` is an installable **plugin**. Paths in `plugin.json` are **relative to the plugin root** (the folder that contains `.claude-plugin/`), per [Claude Code plugin manifests](https://code.claude.com/docs/en/plugin-marketplaces).

Add the marketplace by pointing Claude Code at the directory that contains this file’s sibling `.claude-plugin/marketplace.json` (see [Create and distribute a plugin marketplace](https://code.claude.com/docs/en/plugin-marketplaces)), then install plugins by name, for example:

```text
/plugin install prometheus-entity-graph-crud@prometheus-entity-skills
```

Plugin sources in the marketplace are paths relative to `prometheus-entity-skills/` (for example `./entity-graph-crud`).

## Plugin map

| Plugin directory | `plugin.json` name | Focus |
| ---------------- | ------------------- | ----- |
| `entity-graph-setup/` | `prometheus-entity-graph-setup` | Adopt the library in an existing app; detect legacy data layers; migration plans |
| `entity-graph-crud/` | `prometheus-entity-graph-crud` | CRUD UI, `useEntityCRUD`, tables, forms, relations / `registerSchema` |
| `entity-graph-graphql/` | `prometheus-entity-graph-graphql` | GQL client, descriptors, hooks, subscriptions |
| `entity-graph-realtime/` | `prometheus-entity-graph-realtime` | RealtimeManager, adapters, channels, local-first |
| `entity-graph-prisma/` | `prometheus-entity-graph-prisma` | Prisma mapping, generators, API routes |
| `entity-graph-optimize/` | `prometheus-entity-graph-optimize` | Audits, performance, GC / eviction |

## Sub-skills (nested `skills/*/SKILL.md`)

| Plugin | Sub-skill folders |
| ------ | ----------------- |
| **entity-graph-setup** | `entity-graph-init`, `entity-graph-detect`, `entity-graph-migrate` |
| **entity-graph-crud** | `entity-crud-page`, `entity-crud-form`, `entity-crud-table`, `entity-crud-relations` |
| **entity-graph-graphql** | `entity-gql-setup`, `entity-gql-hooks`, `entity-gql-subscription` |
| **entity-graph-realtime** | `entity-realtime-setup`, `entity-realtime-channel`, `entity-realtime-local-first` |
| **entity-graph-prisma** | `entity-prisma-setup`, `entity-prisma-generator`, `entity-prisma-api`, `entity-prisma-migrate` |
| **entity-graph-optimize** | `entity-audit`, `entity-perf`, `entity-gc` |

Each sub-skill is a normal Agent Skill directory with its own `SKILL.md` and must keep `name` in frontmatter aligned with the folder name per agentskills.io rules.

## Shared references (monorepo paths)

All paths below are relative to the **repository root** of `prometheus-entity-management`:

| Path | Role |
| ---- | ---- |
| `prometheus-entity-skills/_shared/references/library-exports.json` | Sorted list of **runtime export names** from `dist/index.mjs`; must match `pnpm run verify:skills` |
| `prometheus-entity-skills/_shared/references/library-api.md` | Human-oriented API notes for agents |
| `prometheus-entity-skills/_shared/references/architecture-rules.md` | Non-negotiable layering (Components → Hooks → Stores) |
| `prometheus-entity-skills/_shared/references/branding.md` | Example UI tokens for generated demos |
| `prometheus-entity-skills/_shared/references/schemas/*.schema.json` | JSON Schemas for manifests and filters |

Regenerate the export ledger after changing `src/index.ts` exports:

```bash
pnpm run refresh:exports
```

## Non-negotiable architecture (summary)

- **Components** must not call the graph store directly; **hooks** orchestrate; **stores/adapters** own I/O.
- **Lists store entity IDs only**; entity data lives once in the graph.
- Skills that generate code must follow `AGENTS.md` / `CLAUDE.md` in the library repo.

## Validation

- **Library ↔ ledger:** `pnpm run verify:skills` (requires `pnpm run build` first).
- **Leaf skills:** Prefer the official validator from the Agent Skills ecosystem when packaging for external marketplaces (`skills-ref validate ./path` per [agentskills.io](https://agentskills.io/specification)).
