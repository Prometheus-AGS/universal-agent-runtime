---
type: Reference
id: sansaba-web-scaffold-phase-and-cedar-c-29-gate-status
title: Sansaba web scaffold phase and Cedar C-29 gate status
tags:
- sansaba-workspace
- web-scaffold
- cedar-policy
- c29-gate
- local-first
- axum-bff
- authorization
sources:
- stdin
- manual:Sansaba Workspace/web-scaffold-and-c29-gate
timestamp: 2026-07-27T10:43:06.093102+00:00
created_at: 2026-07-27T10:43:06.093102+00:00
updated_at: 2026-07-27T10:43:06.093102+00:00
revision: 0
---

## Context

- Project: **Sansaba Workspace**
- KBD root: `/Users/gqadonis/Projects/sansaba/San Saba Automation/sansaba-workspace`
- Phase: `web-scaffold-and-c29-gate`
- Captured: `2026-07-27T10:42:25Z`
- Phase status: `apply_ready`
- Progress: `changes 4/7`
- Last completed change: `cedar-policy-layer`

## Phase goals

The phase goal is full implementation of the project documentation:

- `CLAUDE.md`
- `docs/SSW-WEB-002-scaffold-plan.md`
- `docs/SSW-ARCH-001-architecture-spec-and-plan.md` Appendix C
- `docs/branding/branding-guide.md`

### G-1: Install KnowMe project skills

Run `add-project-skills.sh` against the repository per `SSW-WEB-002` step 2 so the doctrine is installed as project skills alongside authored skills:

- `pem-local-first`
- `sync-doctrine`
- `content-block-ui`
- `reference-ui-fidelity`
- `hybrid-design-tokens`
- `a11y-gate`

Verified script location as of `2026-07-27`:

- `~/.claude/plugins/marketplaces/knowme-hybrid-architecture/scripts/add-project-skills.sh`
- Cache copy: `~/.claude/plugins/cache/knowme-hybrid-architecture/hybrid-mobile-architecture/1.1.0/scripts/`

### G-2: Scaffold `web/` per `SSW-WEB-002`

Required client stack at `web/`:

- React 19
- Vite 8
- TypeScript 7
- shadcn-ui
- Tailwind 4
- PEM `3.0.0-alpha.0` with PGlite for local-first storage
- Zustand 5 for transient UI state only
- TanStack Router/Table
- Assistant-UI
- `bridge/{a2ui,agui}` layer

Binding requirements from `SSW-WEB-002` §1 and §3:

- Do **not** use `scaffold-tauri.sh` as the app generator; scaffold Vite fresh.
- Transport is **web REST + SSE to the Axum BFF**, not Tauri IPC.
- Strip Tauri coupling:
  - no `src-tauri/`
  - no `invoke()` or `listen()` calls in stores

## Completed change: Cedar policy layer

`kbd-apply cedar-policy-layer` completed and archived:

- Tasks: `21/21`
- Cedar version: `4.11.2`
- Tests: `38` total
  - `22` Rust
  - `16` TypeScript
- Validation: `clippy` and `fmt` clean
- Review document added: `docs/SSW-AUTHZ-003-cedar-policy-model.md`

### Authorization decisions applied

Two reviewed decisions were applied:

1. **Tenant-member-plus-admin model**
   - Keeps the C-29 flow working.
   - A reviewer correcting an empty Receipt Date is an ordinary tenant member, not necessarily an admin.
   - Domain roles remain unmodelled until San Saba confirms real role names.

2. **Reads are authorized**
   - `ReadEntity` now passes through Cedar authorization in addition to the existing tenant scoping in `entities.rs`.

### Guard design: `MutationPermit`

A path/method-inspecting middleware was rejected because it can fail silently when new routes are added and not recognized.

The implemented guard uses a typed `MutationPermit`:

- Mutating helpers require a permit by type.
- The only constructor performs the Cedar decision.
- A handler that forgets authorization cannot call mutation helpers because it lacks the required permit value.
- This turns forgotten authorization into a compile-time error rather than a runtime policy gap.

This directly addresses the `SSW-ARCH-001` risk of authorization bypass through alternate views or handlers.

### Cedar policy details

- Cross-tenant access is explicitly protected with `forbid` even though deny-by-default would already reject it when no permit matches.
- Rationale: Cedar `forbid` cannot be overridden by future `permit` rules, reducing risk from later careless policy additions.
- `source` is policy context, not a permission source.
  - It may be logged or restricted by policy.
  - Nothing is allowed merely because it arrived over chat or agent transport.

### Test coverage added

Coverage now includes:

- Cross-tenant denial
- Admin cannot cross tenant boundaries
- Refusal when no tenant claim is present
- Agent-triggered writes obey the same guard as direct writes

### Intentional dead-code allowances

Four `#[allow(dead_code)]` annotations were added with explicit `reason` values because the taxonomy is intentionally complete before all production routes exist:

- `WriteEntity`
- `AssembleSurface`
- `DispatchTask`
- `InvokeTool`
- Permit constructor

Rationale: defining actions per feature would require revisiting the security model repeatedly as routes land.

## Current blocker and next steps

`letter-agreement-c29-gate` is blocked at task `1.1`, which must verify the anonymized replica before the remaining tasks can proceed:

- Replica existence
- Connection string
- Whether LA `#4016` is seeded

This same infrastructure gap blocks the two remaining `scaffold-web-client` tasks.

Available next action:

- `/kbd-apply test-suite-and-dod` — can proceed now and hardens existing implementation.

Blocked next action:

- `/kbd-apply letter-agreement-c29-gate` — wait for anonymized replica availability.

# Citations

1. stdin
2. manual:Sansaba Workspace/web-scaffold-and-c29-gate