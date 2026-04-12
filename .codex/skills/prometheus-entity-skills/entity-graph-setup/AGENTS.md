# AGENTS.md — entity-graph-setup

Instructions for autonomous or semi-autonomous agents running this skill.

## Role

You are integrating **@prometheus-ags/prometheus-entity-management** into an existing or new React application. Your output is **actionable**: concrete file paths, dependency lines, hook signatures, and a **migration plan** the team can execute in PR-sized chunks.

## Operating principles

1. **Evidence first** — Grep and read files before asserting what stack the app uses. Record findings in `entity_manifest` / `stack_report` artifacts.
2. **Respect layering** — Components consume **hooks only**. Hooks call **store/engine** APIs. **I/O** lives in configureEngine fetchers, adapters, or existing API modules invoked from those layers.
3. **Normalized graph** — Lists = **IDs**; entities = **single canonical row** per `type:id`. Migration plans must call this out when old code duplicated rows in query cache.
4. **Minimize blast radius** — Default to **incremental** migration (route-by-route or feature-flagged modules).

## Required phases

Use the prompt templates under `prompts/` in order unless the user already completed an earlier phase:

| Phase | Artifact |
|-------|----------|
| Specify | `entity_spec` + `stack_report` (markdown or JSON) |
| Plan | `migration_plan.md` with ordered tasks and rollback |
| Execute | Patches / new files listing |
| Reflect | Commands run + results summary |
| Persist | State update + optional orchestrator JSON |

## Commands

- Initialize session state: `bash prometheus-entity-skills/entity-graph-setup/scripts/state-init.sh <workspace_root>`
- Orchestrators: `bash prometheus-entity-skills/entity-graph-setup/scripts/detect-orchestrators.sh` (from repository root; adjust path if the skill pack is copied elsewhere)

## Quality bar

- TypeScript **strict**-friendly generated code; no `any` unless at a third-party boundary with a comment.
- Mention **SSR hydration** (e.g. Next.js server payload → `upsertEntity`) when the app uses RSC + client graph.
- Link to sibling skills for deep work: **entity-graph-crud**, **entity-graph-graphql**, **entity-graph-prisma**, **entity-graph-realtime**.

## Failure modes to avoid

- Installing the library without a **fetch** story (engine stays no-op or broken).
- Registering **`EntitySchema`** with incorrect `listKeyPrefix` → silent missed invalidations; validate keys against real `useEntityList` usages.
- Teaching components to read **`useGraphStore`** directly.
