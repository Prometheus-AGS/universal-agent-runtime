---
name: entity-audit
description: >
  Slash command /entity-audit — run full codebase audit against @prometheus-ags/prometheus-entity-management CLAUDE.md
  rules: component/store boundaries, hook I/O, registerSchema coverage, JSDoc on public hooks, and
  realtime/list patterns. Produces a severitized report using references/audit-checklist.md.
---

# /entity-audit

## Invoke

Run **auditor** agent (`agents/auditor.md`) across the target tree.

## Steps

1. Load `references/audit-checklist.md` — check every box with evidence (file:line).
2. Grep for violations:
   - `useGraphStore` in UI paths
   - Raw `fetch` in components
3. Cross-check CRUD types vs `registerSchema` registrations.
4. Emit P0–P3 table + recommended patches.

## Deliverable

`AUDIT-ENTITY-GRAPH.md` (or append to team wiki) with sign-off owner.

## References

- `../CLAUDE.md`
- `../references/audit-checklist.md`
- Repo root `CLAUDE.md` / `AGENTS.md`
