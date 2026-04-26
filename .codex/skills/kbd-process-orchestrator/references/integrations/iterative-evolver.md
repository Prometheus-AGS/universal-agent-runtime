# KBD Integration: iterative-evolver

KBD and `iterative-evolver` form a **bidirectional nested PMPO loop**:

- **Evolver → KBD**: The evolver's Execute phase delegates to KBD for
  fine-grained task decomposition, multi-tool dispatch, and artifact QA
- **KBD → Evolver**: KBD's Assess phase can delegate to the evolver for
  deep multi-dimensional analysis with cross-session continuity

**Global skill location**: `skills/process/iterative-evolver/SKILL.md`
**Entry commands**: `/evolve`, `/evolve-assess`, `/evolve-report`
**State backend**: `.evolver/evolutions/<evolution-name>/state.json`

---

## Direction 1: Evolver Delegates Execute to KBD

When the evolver runs a `software` domain evolution, its Execute phase
creates a KBD phase and runs the full KBD inner loop:

```
evolve-execute (outer)
  ├─ Map plan.json items → KBD phase goals
  ├─ Write evolver-bridge.json
  ├─ /kbd-plan → decompose into changes (OpenSpec auto-detected)
  ├─ /kbd-execute → multi-tool dispatch + artifact-refiner QA
  └─ /kbd-reflect → results flow back via evolver-bridge.json
```

### Evolver Bridge File

Located at `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`:

```json
{
  "evolution_name": "<evolution-name>",
  "evolver_plan_path": ".evolver/evolutions/<name>/plan.json",
  "item_to_change_map": {
    "evolver-item-1": ["change-001", "change-002"],
    "evolver-item-2": ["change-003"]
  }
}
```

KBD's `/kbd-plan` populates `item_to_change_map` during change decomposition.
KBD's `/kbd-reflect` uses it to report per-item completion back to the evolver.

---

## Direction 2: KBD Delegates Assess to Evolver

| KBD Phase           | Evolver Role                                             | Entry Point      |
| ------------------- | -------------------------------------------------------- | ---------------- |
| **Assess**          | Deep codebase + spec gap analysis with domain context    | `/evolve-assess` |
| **Full phase loop** | Run the complete Assess → Analyze → Plan → Reflect cycle | `/evolve`        |
| **Reflect**         | Generate a structured reflection with before/after delta | `/evolve-report` |

> **Note**: KBD's `/kbd-assess` is a lighter, waypoint-aware wrapper.
> When a phase requires deep multi-dimensional analysis or cross-session
> continuity, delegate to `iterative-evolver` with `evolution_domain: software`.

### How to Invoke (KBD → Evolver Contract)

```yaml
# Pass this to /evolve when starting a new KBD phase assessment
evolution_name: '<project-name>-<phase-name>'
evolution_domain: software
goals:
  - description: '<phase goal 1>'
    priority: high
  - description: '<phase goal 2>'
    priority: medium
constraints:
  # Import from .kbd-orchestrator/constraints.md blocking constraints
target_state:
  description: '<one paragraph describing the completed phase outcome>'
context:
  project_path: '<absolute path to project root>'
  documents:
    - 'AGENTS.md'
    - '.kbd-orchestrator/phases/<phase>/plan.md'
  prior_assessments:
    - '.kbd-orchestrator/phases/<prev-phase>/reflection.md'
workflow_triggers:
  - event: on_cycle_complete
    action:
      type: command
      target: "echo '[kbd] evolver cycle complete — update assessment.md'"
```

### What KBD Reads Back

After `/evolve-assess` or `/evolve-report`, KBD reads:

- `.evolver/evolutions/<evolution-name>/state.json` → `assessment.report`
- `.evolver/evolutions/<evolution-name>/reports/` → latest report file

KBD then copies or symlinks the report to:
`.kbd-orchestrator/phases/<phase>/assessment.md`

---

## Cross-Session Resumption

The `evolution_name` is the resume key. KBD stores it in `project.json`:

```json
{
  "evolver_evolution_name": "<project-name>-<phase-name>"
}
```

If the evolver session was interrupted, re-running `/evolve "<evolution-name>"`
resumes from the last checkpoint automatically.

---

## State File Summary

| File                                                   | Owner            | Direction                         |
| ------------------------------------------------------ | ---------------- | --------------------------------- |
| `.evolver/evolutions/<name>/plan.json`                 | Evolver          | Evolver → KBD (plan items)        |
| `.evolver/evolutions/<name>/state.json`                | Evolver          | KBD → Evolver (execution results) |
| `.kbd-orchestrator/phases/<phase>/evolver-bridge.json` | KBD              | Bidirectional mapping             |
| `.kbd-orchestrator/phases/<phase>/assessment.md`       | KBD              | Evolver → KBD (deep assessment)   |
| `.kbd-orchestrator/phases/<phase>/reflection.md`       | KBD              | KBD → Evolver (phase results)     |
| `.refiner/artifacts/<change-id>/refinement_log.md`     | Artifact-refiner | KBD → Evolver (QA data)           |

---

## When NOT to Use

- For small phases with < 3 changes: use `/kbd-assess` directly (lightweight)
- When the phase is well-understood: skip evolver, write `assessment.md` manually
- `iterative-evolver` is most valuable for the first assessment of a major new phase
- The KBD delegation from evolver is most valuable for phases with 3+ plan items
  that benefit from multi-tool dispatch and artifact QA
