# Current Waypoint

- Phase: `submodule-entity-management-implementation` **(plan_complete)**
- Previous phase: `submodule-skills-and-entity-devtools-expansion` *(reflect_complete; 11/11 changes archived)*
- Backend: OpenSpec
- Wave: **W0** — kickoff
- Status: `plan_complete`
- Progress: **0 / 11 changes** scaffolded
- Active change: `seim-skill-system-pr-bundle` *(state: ready_for_opsx_new)*
- Next pending change: same — first OpenSpec change to scaffold
- Exact next command: `/opsx:new seim-skill-system-pr-bundle`
- Plan: [plan.md](phases/submodule-entity-management-implementation/plan.md)
- Assessment: [assessment.md](phases/submodule-entity-management-implementation/assessment.md)
- Updated at: 2026-05-27T00:00:00Z

## Wave shape (recap)

| Wave | Changes |
|---|---|
| W0 | 1 — commit prior phase's skill-system tree (blocking) |
| W1 | 2 — surreal-live spec correction |
| W2 | 3 — new worktree at `~/.claude/worktrees/seim-entity-management` |
| W3 | 4 surreal-live impl + 5 engine devtools tap *(parallel)* |
| W4 | 6 preflight UI/UX research *(gating)* |
| W5 | 7 event bus + multi-store registry |
| W6 | 8 panel components (the big one) |
| W7 | 9 tree-shake check + 10 extension architecture notes *(parallel)* |
| W8 | 11 extension scaffold *(stretch — scaffold-only)* |

## Default decisions (in effect unless user overrides)

1. Spec reconciliation: delta change preserving historical record.
2. Worktree: new persistent worktree under `~/.claude/worktrees/seim-entity-management`.
3. Chrome extension scope this phase: scaffold-only.
4. Production tree-shake gate: hard fail in `prepublishOnly`.

All KBD skills will emit `starting/ending <kind> <name> [i/n]` lines on stderr (default reporter from the hook system); each change emits its own `phase:before` / `phase:after` hooks; memory recall auto-fires on every `assess:before`.
