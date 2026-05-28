## Why

Change 2 (`kbd-nested-phase-schema`) introduced the *fields* (`parentPhase`, `childPhases`, `childPointer`) that support nested phases. Change 4 (`kbd-new-phase-skill`) introduced the top-level phase creator. What's missing is the **two skills that read and write the nested-phase fields**: `/kbd-new-child` (creates a child inside the active phase) and `/kbd-next-child` (advances `childPointer` to the next member of `childPhases`).

Together they enable the user's stated workflow: "split work up more effectively based on complexity discovered during work being done". A parent phase can spawn one or more child phases mid-flight; each child has its own scope and lifecycle while remaining owned by the parent.

This is the last W1 change. After it lands, the KBD orchestrator has a complete phase lifecycle surface: new (top-level), next (top-level auto-seeded), new-child, next-child, plus the supporting status + status-rendering + hook fires.

## What Changes

### Two new skills under `skills/process/kbd-process-orchestrator/skills/`

- **`kbd-new-child/`** — `SKILL.md` + `kbd-new-child.sh`. Accepts `<child-name> [goals…]`. Creates `.kbd-orchestrator/phases/<parent>/children/<child-name>/` with `goals.md` and `progress.json` (mirroring the top-level `phases/<name>/` layout). Updates the parent's waypoint to append `<child-name>` to `childPhases[]` and set `childPointer` to the new child. Writes a separate `current-waypoint.json` inside the child directory for child-scoped tools, but the project-level waypoint at `.kbd-orchestrator/current-waypoint.json` still describes the parent (with the child reflected in `childPointer`). Fires `child:before`.

- **`kbd-next-child/`** — `SKILL.md` + `kbd-next-child.sh`. With no argument, advances `childPointer` to the next entry in `childPhases[]` after the current pointer (canonical iteration order from change 2). With an explicit `<child-name>`, jumps directly to that named child. Fires `child:after` for the closing child (if any) and `child:before` for the new active child. Refuses to advance past the last child.

### Child phase directory layout

```
.kbd-orchestrator/phases/<parent-name>/
├── goals.md
├── progress.json
└── children/
    ├── <child-1>/
    │   ├── goals.md
    │   └── progress.json
    └── <child-2>/
        ├── goals.md
        └── progress.json
```

Children live under their parent's directory so the relationship is self-describing on disk. Each child gets its own `goals.md` and `progress.json` with the canonical field set; `parentPhase` is set to the parent's name.

### Waypoint behaviour

The single `.kbd-orchestrator/current-waypoint.json` represents the *active focus*. When a child is selected:

- `phase` remains the parent name.
- `childPhases` lists the children declared so far.
- `childPointer` names the active child.
- `currentTask` is updated to point at the child's next step (e.g. `"run kbd-assess for <parent>/<child>"`).
- `exactNextCommand` is updated accordingly.

Tools that don't know about nested phases see a normal parent-phase waypoint and continue working; tools that do see the active child via `childPointer`.

### Hook contract (closes change 3 forward-looking requirement)

Change 3's spec req 7 scenario "Child bracket fires (forward-compatible)" explicitly hands the obligation to this change. Implementation:

- `/kbd-new-child` fires exactly one `child:before` for the new child after the parent's waypoint is updated.
- `/kbd-next-child` fires `child:after` for the closing child (if `childPointer` was non-null) and `child:before` for the new active child.

### Validation

- `<child-name>` follows the same regex/no-traversal rules as `/kbd-new-phase`.
- `/kbd-new-child` refuses if no top-level phase is active.
- `/kbd-new-child` refuses if `<child-name>` is already in `childPhases[]`.
- `/kbd-next-child` refuses if `childPhases` is empty (with hint to run `/kbd-new-child` first).
- `/kbd-next-child` refuses if the pointer is already on the last child (with hint to run `/kbd-reflect` then `/kbd-next-phase`).

### Orchestrator documentation

Update `kbd-process-orchestrator/SKILL.md` "Quick Start Commands" / per-skill list to add `/kbd-new-child` and `/kbd-next-child` alongside `/kbd-new-phase` and `/kbd-next-phase`. The "Nested phases" subsection added by change 2 already documents the schema; this change adds the operational complement.

### Non-changes

- **No deeper nesting (grand-children).** Children of children are out of scope. The schema's `parentPhase` field is single-valued; a real grand-child story is a separate change.
- **No automatic child-phase creation from parent reflection.** That's a possible future skill (`/kbd-suggest-child`); not in this change.
- **No retroactive migration of existing flat phases into nested layouts.**

## Capabilities

### New Capabilities

- `kbd-child-phase-skills`: Two skills (`/kbd-new-child`, `/kbd-next-child`) that create and traverse child phases owned by the currently-active top-level phase, honouring `parentPhase` / `childPhases` / `childPointer` semantics from `kbd-nested-phase-schema` and firing `child:before` / `child:after` per `kbd-process-hooks`.

### Modified Capabilities

- None as separate spec entries. `kbd-nested-phase-schema` gains its first writer; `kbd-process-hooks` gains its child-bracket fires — both contracts unchanged.

## Impact

- **Risk**: Low-medium. The data model is already designed and tested (change 2); the hook surface is already wired (change 3). This change combines them. Risk concentrates in the `childPointer` invariants — `/kbd-next-child` must reject illegal pointer states and must never silently corrupt them.
- **Affected files** (skill-system): `skills/kbd-new-child/` (new), `skills/kbd-next-child/` (new), `kbd-process-orchestrator/SKILL.md` (one-line additions). Optional: `shared/lib/childphase.sh` if logic accumulates.
- **Cross-repo**: Yes — same `prometheus-skill-system`, same topic-branch commit.
- **Reversibility**: Trivial — delete the two skill directories.
- **Unblocks**: The full nested-phase workflow becomes usable. Memory-first execution (change 6) and the rule-injector (change 7) can be built atop a complete lifecycle surface.
