---
name: entity-graph-optimize
description: >
  Audit existing @prometheus-ags/prometheus-entity-management integrations for architectural violations (components→hooks→stores),
  performance issues (Zustand selectors, subscription churn), missing cascade/schema registration, and memory growth.
  Sub-skills: full CLAUDE.md audit, render/subscriber analysis, and eviction/GC strategies using graph APIs.
---

# Entity Graph — Optimize

Use this skill when an app **already** uses @prometheus-ags/prometheus-entity-management (or is mid-migration) and needs structured review plus targeted fixes.

## Scope

- **Architecture**: Enforce three-layer data flow from repo `CLAUDE.md` / `AGENTS.md`.
- **Performance**: Hook selector granularity, list `queryKey` stability, dedupe keys, realtime flush interval.
- **Memory**: Entity accumulation (library note: no automatic GC); `removeEntity`, list trimming, route-based cleanup.

## Sub-skills

| Command | Purpose |
|---------|---------|
| `/entity-audit` | Full checklist against CLAUDE.md + JSDoc + import boundaries |
| `/entity-perf` | Re-render and subscription patterns; unnecessary `useStore` breadth |
| `/entity-gc` | Eviction strategies, `removeEntity`, patch cleanup, list invalidation |

## PMPO

`prompts/specify.md` → `plan.md` → `execute.md` → `reflect.md` → `persist.md`.

## Honest constraints

The core library **does not yet ship automatic entity garbage collection**. The **entity-gc** sub-skill documents **application-level** patterns using `removeEntity`, clearing lists, and navigation-bound cleanup — not a nonexistent built-in GC toggle.

## pnpm

Use **pnpm** for any repo scripts during verification.
