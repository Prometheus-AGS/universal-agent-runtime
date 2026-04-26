---
name: kbd-reflect
description: >
  Generate the phase reflection report after all changes in the phase are
  complete. Seeds the next phase's planning inputs. Aggregates artifact-refiner
  QA results and reports back to iterative-evolver when running inside an
  evolution cycle.
---

# /kbd-reflect

Run the **Reflect** phase of the KBD lifecycle.

## What this does

Generates `.kbd-orchestrator/phases/<phase-name>/reflection.md` summarizing:

- Goal achievement percentage (MET / PARTIAL / NOT MET per goal)
- Delivered changes (from progress.json and archive)
- **Artifact quality summary** (from artifact-refiner logs)
- Technical debt introduced
- Lessons captured for knowledge base
- Recommended focus for next phase

## Artifact Quality Summary

Aggregate artifact-refiner results from all changes in the phase:

1. **Read refinement logs** — `.refiner/artifacts/<change-id>/refinement_log.md`
   for each change in `progress.json`
2. **Compute pass rate** — `changes_passed / total_changes`
3. **List constraint violations** — group by constraint name, count occurrences
4. **Identify recurring patterns** — constraints that failed across 2+ changes

Include in `reflection.md` under `## Artifact Quality Summary`:

```markdown
## Artifact Quality Summary

| Metric                       | Value     |
| ---------------------------- | --------- |
| Changes with QA              | 8/10      |
| First-pass pass rate         | 6/8 (75%) |
| Changes requiring refinement | 2         |
| Total refinement iterations  | 5         |

### Recurring Constraint Violations

- `no-any-types`: 3 changes (change-002, change-005, change-007)
- `test-coverage-80`: 2 changes (change-003, change-008)
```

## Evolver Feedback

When this phase is part of an iterative-evolver cycle:

1. **Read evolver-bridge.json** — `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`
2. **Map change completion to evolver items** — using `item_to_change_map`
3. **Write evolver execution results** — update `.evolver/evolutions/<name>/state.json`:
   - Set `execution_results` with per-item completion status
   - Include artifact quality metrics
4. **Trigger evolver reflect** — the outer evolver loop picks up from here

This bridges the KBD inner loop back to the iterative-evolver outer loop,
enabling the evolver's Reflect phase to measure actual progress against
strategic goals.

## Prerequisites

All changes for this phase must be:

- Completed (status `DONE` in `progress.json`)
- QA gate passed (artifact-refiner, unless skipped)
- If OpenSpec: verified (`/opsx:verify`) and archived (`/opsx:archive`)
- If native KBD: moved to `.kbd-orchestrator/changes/archive/<date>-<id>/`

## How to invoke

1. **Discover project identity**
2. **Confirm the active phase** — from argument or waypoint
3. **Read `progress.json`** — incorporate work done by all tools
4. **Read artifact-refiner logs** — aggregate QA results
5. **Load all change data** — from `openspec/changes/archive/` if OpenSpec,
   or `.kbd-orchestrator/changes/archive/` if native KBD
6. **Follow the reflect protocol** in `../prompts/reflect.md`
7. **Write reflection** to `.kbd-orchestrator/phases/<phase>/reflection.md`
8. **If evolver bridge exists**: write execution results back to evolver state
9. **Advance the waypoint** to the next phase
10. **Trigger**: `echo '[kbd] Reflection complete — advance to next phase with /kbd-new-phase'`

## Examples

```
/kbd-reflect                             # uses active waypoint phase
/kbd-reflect phase-1-foundation          # explicit phase name
```
