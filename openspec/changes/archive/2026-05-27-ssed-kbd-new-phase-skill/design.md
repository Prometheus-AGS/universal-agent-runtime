## Context

`kbd-process-orchestrator` ships seven skills today: `kbd-init`, `kbd-assess`, `kbd-plan`, `kbd-execute`, `kbd-reflect`, `kbd-status`, `kbd-next-phase`. Each is **markdown-described** — the directory contains a `SKILL.md` and (optionally) supporting prompts/references; the AI agent reads the SKILL.md and executes the documented steps. **`kbd-next-phase.sh` is referenced in `kbd-next-phase/SKILL.md` but does not actually exist** — the steps it describes are executed by the agent, not by a shipped binary.

This pattern is intentional: the skills are agent-facing instructions, not executable code paths. It keeps the surface area tiny and tool-portable (Roo, Codex, Cursor, etc. all read the SKILL.md the same way). The new "Hooks" wiring landed in change 3 is itself executed by the agent following the SKILL.md "Hook integration" subsection — there's no Python or Node runtime invoking these skills.

This change introduces `/kbd-new-phase` in the same shape: a SKILL.md that documents the manual phase-creation workflow, plus optional helper scripts when a step benefits from one (e.g. atomic JSON rewrites).

This reframing creates a small but real tension with the just-written spec, which requires "a non-empty executable `kbd-new-phase.sh`". Resolution below (D2).

## Goals / Non-Goals

**Goals**
- A `/kbd-new-phase` skill that closes the documented-but-missing gap and follows the same shape as `/kbd-next-phase`.
- The new phase row written to disk carries the nested-phase defaults from change 2 (`parentPhase=null`, `childPhases=[]`, `childPointer=null`).
- A `phase:before` event fires exactly once for the new phase via `kbd_hooks_fire` (change 3 wiring).
- Failure modes are explicit: refuse name collisions, abort on malformed waypoint, warn-but-continue on absent `project.json`, warn-but-continue on missing hook subsystem.

**Non-Goals**
- **No transactional rollback.** If hook fire fails after the phase is on disk, the phase stays. The spec calls this out explicitly ("Idempotency Boundary"); the design records that we considered and rejected a rollback path (Alternatives).
- **No reflection reading.** That's `/kbd-next-phase`'s job; `/kbd-new-phase` is the manual counterpart.
- **No child-phase support.** Reserved for change 5.
- **No re-write of `kbd-next-phase`.** They are siblings.

## Decisions

### D1. The skill is markdown-described first; a helper script is optional, not required

Following the existing convention, `/kbd-new-phase` ships as a `SKILL.md` whose "How to invoke" steps are agent-executable. The agent:

1. Parses arguments from `$ARGUMENTS`.
2. Validates the name (regex + collision check).
3. Creates the phase directory.
4. Writes `goals.md` and `progress.json` atomically (temp + rename).
5. Updates `current-waypoint.json` and `project.json`.
6. Sources `hooks.sh` and fires `phase:before`.
7. Emits Progress Signals.

Every one of these steps is a few lines of shell that the agent runs directly via its `Bash` tool. The agent has all the same primitives (`mkdir`, `jq`, `mv`, `printf`, `date -u`) the sibling skills already use.

### D2. Reconciling spec requirement "Skill Surface" with the no-script convention

The spec requires "a non-empty executable `kbd-new-phase.sh`". The sibling `/kbd-next-phase` ships no such file. To resolve:

- **Option A (chosen)**: Ship a small `kbd-new-phase.sh` that *does* exist and *is* executable, but the SKILL.md remains the source of truth for the workflow. The script is a thin convenience wrapper that performs the atomic JSON writes (the one part that benefits from being scripted vs. step-by-step). If a calling agent invokes the script directly, it runs end-to-end; if the agent follows the SKILL.md steps inline, it gets the same result. This keeps the spec scenario satisfied *and* honours the agent-driven convention.
- **Option B (rejected)**: Edit the spec to drop the `.sh` requirement. Rejected because the spec is already correct in spirit — having *something* executable named after the skill makes cross-tool discoverability obvious, and there's no cost to shipping the script.

So: SKILL.md is canonical; `kbd-new-phase.sh` is a convenience entry point that produces identical results.

### D3. Atomic writes use `jq` + `mv`

Every JSON write goes through:

```sh
tmp="$file.tmp.$$"
jq -r ... > "$tmp"
mv -f "$tmp" "$file"
```

`mv` within the same directory is atomic on POSIX filesystems. This satisfies the spec's "Atomic Write" scenario without any locking primitives. `markdown` writes (`goals.md`) use the same temp-and-rename pattern.

### D4. Waypoint JSON is rewritten, not merged

The skill rewrites `current-waypoint.json` whole, preserving every field that's not explicitly being updated. The rewrite uses a single `jq` pipeline that reads the old object, sets the new fields, and emits the result. Unknown fields (e.g. a tool-specific `myTool_metadata` key) pass through untouched — this is the same fault-tolerance contract that change 2's schema established.

### D5. Hook fire is the last side effect before the Progress Signal

Order in the implementation:

```
1. validate args
2. write phases/<name>/goals.md         (atomic)
3. write phases/<name>/progress.json    (atomic)
4. update current-waypoint.json         (atomic via tmp+mv)
5. update project.json                  (atomic via tmp+mv; warn-only if absent)
6. fire phase:before                    (best-effort; warn-only if hooks unsourceable)
7. emit "Completed kbd-new-phase — <name> ready for /kbd-assess"
8. print confirmation banner
```

Rationale: any hook that reads waypoint or project state sees the *new* phase as authoritative. The hook reaches "after the world is updated" rather than "in the middle of the transition". This matches the design convention used elsewhere (the report-progress hook reads `KBD_HOOK_*` env, but third-party hooks might read the on-disk state, so the on-disk state must be settled first).

### D6. The "first waypoint write" path uses a documented default skeleton

When `current-waypoint.json` does not exist (very-first-phase case), the skill writes a complete waypoint object using the documented field set (matching `references/schemas/current-waypoint.template.json` introduced in change 2). `previousPhase` is `null`. No interactive prompt; no inference from elsewhere.

### D7. Malformed waypoint aborts; absent project.json warns

The spec is asymmetric here, and that's intentional:

- A **malformed waypoint** is a corruption — the operator must fix it. Continuing risks compounding the corruption (write a new waypoint that loses fields the original had). Abort and surface the path.
- An **absent `project.json`** is a "you haven't run `/kbd-init` yet" condition — the system still works, but cross-tool routing is degraded. Warn and continue.

This asymmetry is captured in the spec's two distinct scenarios.

### D8. Cross-repo commit shape

This change adds at most three files in `prometheus-skill-system`:

- `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-new-phase/kbd-new-phase.sh`
- A one-line edit to `skills/process/kbd-process-orchestrator/SKILL.md` confirming `/kbd-new-phase` is implemented.

This folds naturally into the topic-branch commit accumulating changes 2 + 3 + this one (and likely change 5 too, since `/kbd-new-child`/`/kbd-next-child` will land in the same skill subdirectory).

### D9. Validation rejects names containing `..`

Beyond the canonical regex, an extra explicit check refuses any name containing `..` even if it would technically match the regex (it won't, but defense-in-depth). This is the same rule used by `scripts/worktree-new.sh` in change 1.

## Implementation Sketch

### `SKILL.md` skeleton (mirrors `kbd-next-phase`)

```
---
license: MIT
name: kbd-new-phase
version: '1.0.0'
description: >
  Manually create a new top-level KBD phase. Accepts <name> [goals…] and
  initialises the phase directory, waypoint, project.json activePhase, and
  fires phase:before. Use this when no prior reflection exists or when
  pivoting to a phase that the reflection didn't suggest.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-new-phase

## What this does
1. Parses <name> + [goals…] from arguments.
2. Validates the name (kebab-case, no traversal, no collision).
3. Creates .kbd-orchestrator/phases/<name>/ with goals.md + progress.json.
4. Flips current-waypoint.json (previousPhase ← prior phase, phase ← <name>, …).
5. Flips project.json activePhase (warn if absent).
6. Fires phase:before via shared/lib/hooks.sh.
7. Emits "Completed kbd-new-phase — <name> ready for /kbd-assess" + a banner.

## When to use
Run when (a) no prior reflection exists (e.g. very first phase),
(b) you're pivoting away from /kbd-next-phase's suggestion, or
(c) you're initialising state manually.

Compare to /kbd-next-phase, which auto-seeds from reflection.md.

## Progress Signals (MANDATORY)
Starting kbd-new-phase — <name>
Completed kbd-new-phase — <name> ready for /kbd-assess

## Prerequisites
- The proposed name must not already exist as a phase directory.
- current-waypoint.json must be valid JSON (or absent for the first-phase case).

## How to invoke
1. Parse args ⇒ name, goals[]
2. Validate name (regex ^[a-z0-9][a-z0-9._-]*$, refuse ..)
3. Refuse if phases/<name>/ exists
4. mkdir -p phases/<name>
5. Write phases/<name>/goals.md (atomic)
6. Write phases/<name>/progress.json (atomic)
7. Update current-waypoint.json (atomic)
8. Update project.json activePhase (warn if absent)
9. Source shared/lib/hooks.sh; fire phase:before "$name" 1 1
10. Print banner: phase, goals.md path, "Next: /kbd-assess <name>"

## Hook integration
Same stanza as /kbd-next-phase — fires phase:before exactly once after
the waypoint flip.

## Examples
/kbd-new-phase phase-1-foundation
/kbd-new-phase ux-refresh "polish dashboard" "ship dark mode" "audit a11y"
```

### `kbd-new-phase.sh` skeleton

```bash
#!/usr/bin/env bash
set -euo pipefail

die() { printf 'kbd-new-phase: %s\n' "$*" >&2; exit 1; }

name="${1:-}"
[[ -n "$name" ]] || die "usage: kbd-new-phase.sh <name> [goal-1] [goal-2] …"
shift
goals=("$@")

# Validation
case "$name" in
  *..*) die "invalid name: parent traversal not allowed" ;;
  */*)  die "invalid name: slashes not allowed" ;;
esac
[[ "$name" =~ ^[a-z0-9][a-z0-9._-]*$ ]] \
  || die "invalid name: must match ^[a-z0-9][a-z0-9._-]*$"

phase_dir=".kbd-orchestrator/phases/$name"
[[ -e "$phase_dir" ]] && die "phase already exists: $phase_dir (try /kbd-next-phase or pick another name)"

# Required deps
command -v jq >/dev/null 2>&1 || die "jq is required"

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$phase_dir"

# goals.md (atomic)
{
  printf '# Goals\n\n'
  if [[ ${#goals[@]} -gt 0 ]]; then
    for g in "${goals[@]}"; do printf -- '- %s\n' "$g"; done
  else
    printf -- '<!-- TBD: enumerate goals before /kbd-assess -->\n'
  fi
} > "$phase_dir/goals.md.tmp"
mv -f "$phase_dir/goals.md.tmp" "$phase_dir/goals.md"

# progress.json (atomic)
wp=".kbd-orchestrator/current-waypoint.json"
prior_phase="$(jq -r '.phase // ""'      "$wp" 2>/dev/null || true)"
source_tool="$(jq -r '.sourceTool // ""' "$wp" 2>/dev/null || true)"
[[ -n "$source_tool" ]] || source_tool="unknown"

jq -n \
  --arg phase "$name" --arg src "$source_tool" --arg now "$now" '
{
  phase: $phase, parentPhase: null, childPhases: [], childPointer: null,
  assessment_complete: false, plan_complete: false, execute_complete: false,
  reflect_complete: false, changes_total: 0, changes_completed: 0,
  completed_changes: [], active_change: null, blocked_changes: [],
  sourceTool: $src, createdBy: "kbd-new-phase", updatedAt: $now
}' > "$phase_dir/progress.json.tmp"
mv -f "$phase_dir/progress.json.tmp" "$phase_dir/progress.json"

# Waypoint flip (atomic)
if [[ -f "$wp" ]]; then
  jq -e . "$wp" >/dev/null \
    || die "malformed waypoint: $wp — fix by hand before retrying"
  jq --arg phase "$name" --arg prev "$prior_phase" --arg now "$now" '
    .previousPhase    = (if $prev == "" then null else $prev end) |
    .phase            = $phase |
    .change           = null |
    .status           = "assessment_ready" |
    .currentTask      = ("run kbd-assess for " + $phase) |
    .nextPendingChange= null |
    .exactNextCommand = ("/kbd-assess " + $phase) |
    .parentPhase      = null |
    .childPhases      = [] |
    .childPointer     = null |
    .updatedAt        = $now
  ' "$wp" > "$wp.tmp"
else
  jq -n --arg phase "$name" --arg now "$now" '
    {phase: $phase, previousPhase: null, change: null,
     status: "assessment_ready",
     currentTask: ("run kbd-assess for " + $phase),
     nextPendingChange: null, sourceTool: "unknown",
     exactNextCommand: ("/kbd-assess " + $phase),
     parentPhase: null, childPhases: [], childPointer: null,
     updatedAt: $now}
  ' > "$wp.tmp"
fi
mkdir -p "$(dirname "$wp")"
mv -f "$wp.tmp" "$wp"

# project.json flip (atomic; warn if absent)
pj=".kbd-orchestrator/project.json"
if [[ -f "$pj" ]]; then
  jq --arg phase "$name" --arg now "$now" '.activePhase = $phase | .updatedAt = $now' "$pj" > "$pj.tmp"
  mv -f "$pj.tmp" "$pj"
else
  printf 'kbd-new-phase: warn: %s missing — run /kbd-init\n' "$pj" >&2
fi

# Hook fire (best-effort)
hooks_lib="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}/shared/lib/hooks.sh"
waypoint_lib="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}/shared/lib/waypoint.sh"
if [[ -f "$hooks_lib" && -f "$waypoint_lib" ]]; then
  export KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
  # shellcheck source=/dev/null
  . "$waypoint_lib"; . "$hooks_lib"
  kbd_hooks_fire phase before "$name" 1 1 || \
    printf 'kbd-new-phase: warn: phase:before hook fire failed (continuing)\n' >&2
else
  printf 'kbd-new-phase: warn: hooks subsystem unavailable (continuing)\n' >&2
fi

# Banner
printf '\n%s\n' "Completed kbd-new-phase — $name ready for /kbd-assess"
printf '  phase:     %s\n' "$name"
printf '  goals:     %s\n' "$phase_dir/goals.md"
printf '  Next:      /kbd-assess %s\n' "$name"
```

### Orchestrator `SKILL.md` edit

Single one-line change: in the "Ongoing workflow" list under "Quick Start Commands", the entry currently reads `/kbd-new-phase <name> [goals...] — Start a new named phase with goals`. After this change lands, no edit is strictly required — the entry was already accurate; it was just orphaned. We add a tiny note: "(implemented in `skills/kbd-new-phase/`)" inline, to make the implementation visible to anyone reading the doc.

## Risks

1. **Two skills writing waypoint concurrently.** Roo and Codex could both invoke `/kbd-new-phase` in different worktrees. Mitigated by atomic `mv` — the loser's write replaces the winner's, but neither sees a half-written file. The race is *waypoint divergence*, not corruption, and is detected by the next `/kbd-status` invocation.
2. **`KBD_ORCHESTRATOR_ROOT` not exported in the calling shell.** The script falls back to `~/.claude/skills/kbd-process-orchestrator` per the existing default; this matches every other skill's expectation.
3. **`jq` absent.** Hard-fail with `die`. Consistent with `waypoint.sh` and `hooks.sh` from change 2 / 3.
4. **`progress.json` field drift over time.** If future changes add fields to the canonical phase progress shape, this skill's initial-write needs to be updated to stay in sync. Filed as a known-maintenance item; the canonical reference is the same template living in `references/schemas/`.
5. **Tension between spec's `.sh` requirement and the no-script convention.** Resolved via D2 (ship the script as a convenience entry point; SKILL.md remains canonical). The spec is satisfied; the convention is preserved.

## Alternatives Considered

- **Make the script transactional with rollback.** Rejected — adds substantial complexity for a low-likelihood failure mode. The spec's "Idempotency Boundary" requirement makes the non-rollback contract explicit; operators can re-trigger the hook later if it failed.
- **Ship no script; only SKILL.md.** Rejected per D2 — the script is a convenience entry point that matches the spec's "Skill Surface" requirement and gives any tool a single command to run.
- **Use a templating engine for `progress.json`.** Rejected — `jq -n` is sufficient, no new dependency.
- **Merge with `/kbd-next-phase` into one parametrised skill.** Rejected — the two have meaningfully different prerequisites (next-phase requires a reflection; new-phase doesn't) and merging would obscure each one's contract.
