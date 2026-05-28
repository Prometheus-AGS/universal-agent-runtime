# Phase Execution — submodule-skills-and-entity-devtools-expansion

- Generated: 2026-05-27
- Author: claude-code (kbd-execute)
- Source plan: [plan.md](./plan.md)
- Source assessment: [assessment.md](./assessment.md)

## Backend selection

| Decision | Value | Rationale |
|---|---|---|
| Backend | **`openspec`** | OpenSpec is configured (`openspec/config.yaml schema: spec-driven`) and the plan already enumerates 11 `ssed-*` changes that map 1:1 to OpenSpec changes. Cross-tool traceability is required because work lands in three different repos. |
| QA gate | **artifact-refiner per change** | Default; opt out only for doc-only changes < 3 files. |
| Worktree | `~/.claude/worktrees/adoring-booth-312094` *(currently in-repo)* | Do **not** relocate — see change 1 design D5/§Non-Goals. |
| Source of truth | `.kbd-orchestrator/` | Surreal-memory MCP mirrors when reachable; not authoritative. |

CLI note: the `openspec` binary is **not installed on this machine**. Until a CLI is in place, every `/opsx:*` invocation in this phase is performed by hand following the on-disk schema observed in `openspec/changes/anthropic-native-driver/`. The structures written are byte-compatible with what the CLI produces. Installing the CLI is captured as a follow-up but not a blocker for execution.

## Dispatch contract

The 11 ordered changes in [plan.md](./plan.md) §"Ordered change list" execute sequentially with limited parallelism after the foundation changes land. Each change goes through the same lifecycle:

```
/opsx:new <change-id>
  → /opsx:continue   (proposal.md)
  → /opsx:continue   (specs/<capability>/spec.md … one per capability)
  → /opsx:continue   (design.md)
  → /opsx:continue   (tasks.md)
/opsx:apply <change-id>             # implement the tasks
artifact-refiner QA (if applicable)
/opsx:verify <change-id>
/opsx:archive <change-id>
update progress.json + waypoint
```

### Execution waves

| Wave | Changes | Why batched | Parallelism |
|---|---|---|---|
| W0 (foundation) | 1 `worktree-persistence-convention` | Must land first — every later change is developed in a worktree. | serial |
| W1 (KBD orchestrator core) | 2 `kbd-nested-phase-schema`, 3 `kbd-process-hooks`, 4 `kbd-new-phase-skill`, 5 `kbd-child-phase-skills` | Same repo (`prometheus-skill-system`), strict dep chain (2→3→4→5). | serial |
| W2 (KBD orchestrator extensions) | 6 `kbd-memory-first-execution`, 7 `kbd-agent-rules-injector` | Both depend on hooks (W1.3); can run in parallel after that. | parallel-2 |
| W3 (this repo doc routing) | 8 `uar-uiux-skill-routing` | Uses the rule-injector machinery from change 7; depends on W2. Lands in this repo, not `prometheus-skill-system`. | serial |
| W4 (entity stack — skill + adapter) | 9 `entity-surreal-live-adapter` | Spans `prometheus-entity-skills` (skill side) and `prometheus-entity-management` (adapter side). Independent of W2/W3 — could start after W0, but staged here to keep cognitive load manageable. | serial |
| W5 (entity explorer) | 10 `entity-explorer-fab-panel` | Depends on UI/UX routing rules from W3 and live-query adapter from W4. | serial |
| W6 (stretch) | 11 `entity-explorer-browser-extension` | Gate on explicit user confirmation; otherwise defer to next phase. | stretch |

### Per-change agent assignments (from plan §"Ordered change list")

| # | Change | Agent / executor |
|---|---|---|
| 1 | `ssed-worktree-persistence-convention` | claude-code |
| 2 | `ssed-kbd-nested-phase-schema` | claude-code |
| 3 | `ssed-kbd-process-hooks` | claude-code |
| 4 | `ssed-kbd-new-phase-skill` | claude-code |
| 5 | `ssed-kbd-child-phase-skills` | claude-code |
| 6 | `ssed-kbd-memory-first-execution` | claude-code |
| 7 | `ssed-kbd-agent-rules-injector` | claude-code (needs web search) |
| 8 | `ssed-uar-uiux-skill-routing` | claude-code (needs web search) |
| 9 | `ssed-entity-surreal-live-adapter` | claude-code (skill side) + `entity-graph-realtime` (adapter side) |
| 10 | `ssed-entity-explorer-fab-panel` | `frontend-design` + `entity-graph-optimize` |
| 11 | `ssed-entity-explorer-browser-extension` | `frontend-design` (gated) |

### Cross-repo commit policy

Three target repos receive commits from this phase:

- **`universal-agent-runtime`** (this repo): changes 1, 8, parts of 10's tests if integration is shown here.
- **`prometheus-skill-system`** at `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`: changes 2–7, 9 (skill side).
- **`prometheus-entity-management`** at `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`: changes 9 (adapter), 10, 11.

Each change's `/opsx:apply` must commit to the *correct* origin. The worktree this phase runs in covers only `universal-agent-runtime`; changes 2–7, 9, 10, 11 must be applied in the other repos directly and tracked here by reference. Concretely: each cross-repo change's `tasks.md` will record the target-repo working directory and the resulting commit SHAs.

## QA gate decisions per change

Per the kbd-execute skip rules ("fewer than 3 files modified" OR "documentation-only"):

| # | Files modified (est.) | Doc-only? | QA required |
|---|---|---|---|
| 1 | 7 (3 scripts + 2 docs + .gitignore + project.json) | no | **yes** |
| 2 | 3–5 (schema + 1–2 skills) | no | **yes** |
| 3 | 6+ (hooks core + 5 skill wirings + log path) | no | **yes** |
| 4 | 2 (skill dir + orchestrator SKILL.md tweak) | mostly | **yes** (touches behavior) |
| 5 | 2–3 (two skill dirs + status update) | mostly | **yes** (touches behavior) |
| 6 | 4–6 (two skills + docs + retention policy) | partly | **yes** |
| 7 | 3+ (skill + rules cache + docs) | partly | **yes** |
| 8 | 2 (CLAUDE.md, AGENTS.md, references cache) | yes | **skip** (`--skip-qa`) |
| 9 | 4+ (skill + adapter + tests + docs) | no | **yes** |
| 10 | 10+ (UI package) | no | **yes** |
| 11 | 12+ (extension package) | no | **yes** |

## Initial state at execute kickoff

- Change 1 (`ssed-worktree-persistence-convention`) has all four OpenSpec artifacts authored (proposal, spec, design, tasks) — created during this `/kbd-execute` invocation per the user's `--args` ("which includes /opsx:continue first").
- Changes 2–11 are **not yet scaffolded**. Next executor action: run `/opsx:new ssed-kbd-nested-phase-schema` (W1 start) once change 1 is `/opsx:apply`'d (its `tasks.md` is the to-do list) — or, if you want to scaffold ahead, batch-create the remaining 10 `/opsx:new` calls now.

## Per-change progress contract

After each change reaches DONE in `progress.json`:

```jsonc
{
  "phase": "submodule-skills-and-entity-devtools-expansion",
  "changes_total": 11,
  "changes_completed": <N>,
  "completed_changes": ["ssed-worktree-persistence-convention", …],
  "active_change": "<next-change-id>",
  "blocked_changes": [],
  …
}
```

## Hand-off points

- **To `surreal-memory`**: after each `/opsx:archive`, log a phase-event entity `{ kind: "change-archived", change: "<id>", phase, completed_at, commit_shas }`. (Pending change 6 `ssed-kbd-memory-first-execution` to make this default-on; for now it is best-effort.)
- **To the user**: at each of the wave boundaries (W0→W1, W1→W2, W2→W3, W3→W4, W4→W5), pause for explicit confirmation. Wave boundaries are decision points for "do we need a child phase here?" — exactly the moment the hooks system (change 3) is designed to highlight.

## Risks & open items at kickoff

1. `openspec` CLI absence — manual scaffolding holds for now; install before W4 (entity work is fiddlier).
2. Child-phase data model (changes 2 + 5) is a schema change to `current-waypoint.json` that ripples to every tool — write the migration test in change 2's tasks.
3. Web-search dependencies for changes 7 and 8 — run those searches at the start of those waves, not later.
4. Cross-repo PR coordination — each repo's CI must be green independently; do not stack interdependent commits across repos in a single change unless the dependency is documented and tested.
5. v1-vs-v2 decision for the browser extension (change 11) is **not yet made** — default is v2/stretch; user confirmation required to promote to v1.

## Next concrete command

Change 1 is artifact-complete. Implementation (`/opsx:apply ssed-worktree-persistence-convention`) is the next step. After change 1 archives, run `/opsx:new ssed-kbd-nested-phase-schema`.
