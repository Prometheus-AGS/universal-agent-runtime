---
name: kbd-plan
description: >
  Create a prioritized, ordered change list for the current project phase.
  Project-agnostic: reads assessment and project constraints to produce
  an ordered change list. Auto-detects OpenSpec availability and emits
  appropriate change formats. Bridges with iterative-evolver plan items
  when running inside an evolution cycle.
---

# /kbd-plan

Run the **Plan** phase of the KBD lifecycle for any project.

## What this does

Reads `.kbd-orchestrator/phases/<phase-name>/assessment.md` and produces an
ordered list of changes to implement this phase. Refreshes the waypoint so
every tool knows the exact next step.

Output:

- `.kbd-orchestrator/phases/<phase-name>/plan.md` — ordered change list
- `.kbd-orchestrator/current-waypoint.md` and `current-waypoint.json` — refreshed

## OpenSpec Detection

Before emitting changes, detect the change management backend:

1. **Check for `openspec/` directory** at project root
2. **Check for `openspec/` directory** inside the skill being developed
3. **Check `.kbd-orchestrator/project.json`** for `"change_backend": "openspec"`

If any of these exist, emit changes as OpenSpec structures. Otherwise, use
native KBD change files.

```
OpenSpec detected?
  YES → emit /opsx:new <change-id> commands
      → create openspec/changes/<change-id>/proposal.md
      → tasks tracked in openspec/changes/<change-id>/tasks.md
  NO  → create .kbd-orchestrator/changes/<change-id>/change.md
      → tasks tracked inline with [ ] / [/] / [x] markers
```

## Evolver Bridge

When this plan is being created as part of an iterative-evolver cycle:

1. **Read evolver plan** — check `.evolver/evolutions/<name>/plan.json`
2. **Map evolver items to KBD changes** — each evolver plan item becomes one
   or more KBD changes with a `evolver_item_id` reference
3. **Write bridge file** — `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`:

```json
{
  "evolution_name": "<name>",
  "evolver_plan_path": ".evolver/evolutions/<name>/plan.json",
  "item_to_change_map": {
    "evolver-item-1": ["change-001", "change-002"],
    "evolver-item-2": ["change-003"]
  }
}
```

This enables the Reflect phase to report back to the evolver with precise
completion status per plan item.

## How to invoke

1. **Discover project identity** — read `.kbd-orchestrator/project.json` or infer
2. **Confirm the active phase** — from argument or waypoint
3. **Load assessment** — from `.kbd-orchestrator/phases/<phase>/assessment.md`
4. **Read project constraints** — from `AGENTS.md` and project spec files
5. **Detect change backend** — OpenSpec or native KBD (see OpenSpec Detection)
6. **Check for evolver bridge** — is this phase driven by an evolution cycle?
7. **Follow the plan protocol** in `../prompts/plan.md`
8. **Write plan.md** with ordered change list and recommended agent per change
9. **Emit change structures** via OpenSpec or native KBD
10. **Write evolver-bridge.json** if evolver plan exists
11. **Refresh waypoint** files (`current-waypoint.md` and `current-waypoint.json`)

## Examples

```
/kbd-plan                                # uses active waypoint phase
/kbd-plan phase-1-foundation             # explicit phase name
```
