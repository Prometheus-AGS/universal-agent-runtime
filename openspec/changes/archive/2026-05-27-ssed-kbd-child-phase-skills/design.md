## Context

Building on changes 2 + 3 + 4:

- Change 2 added `parentPhase`, `childPhases[]`, `childPointer` to `current-waypoint.json` with documented invariants but no writers.
- Change 3 added the hook dispatcher with `child:before` / `child:after` event kinds documented but no skill emitting them.
- Change 4 added `/kbd-new-phase` as the top-level phase creator template — `/kbd-new-child` is the structural sibling.

This change is straightforward composition: mirror `/kbd-new-phase`'s shape with two scripts that operate inside an *existing* parent phase's directory, with hook fires for the child bracket.

## Goals / Non-Goals

**Goals**
- Two skills (`/kbd-new-child`, `/kbd-next-child`) that round out the phase lifecycle.
- Cross-field invariants from change 2 are enforced on write.
- Hook bracket from change 3 is fully wired.
- Same atomic-write discipline as `/kbd-new-phase`.

**Non-Goals**
- No grand-children. `parentPhase` is single-valued.
- No automatic child suggestion from reflection (future skill).
- No retroactive nesting of existing flat phases.

## Decisions

### D1. Children live under their parent's phase directory

`.kbd-orchestrator/phases/<parent>/children/<child>/{goals.md, progress.json}` — same shape as a top-level phase but nested one level. Makes the relationship self-describing on disk; tools that don't know about nesting can ignore the `children/` subdir without harm.

### D2. The project-level waypoint represents the *active focus*, not the active row

Only one `.kbd-orchestrator/current-waypoint.json` exists. When a child is active:
- `phase` = parent's name (unchanged from when the parent was selected)
- `childPointer` = active child's name
- `parentPhase` = `null` (this row IS the parent, not a child row)
- `currentTask` / `exactNextCommand` updated to scope to the child

Rationale: tools that don't know about nesting see "phase: parent" and run `/kbd-assess parent` — that's wrong but harmless (assessment lands in the parent's dir, not in /dev/null). Tools that do know about nesting see `childPointer` and scope their work into the child. We avoid having two competing waypoint files.

(An alternative considered and rejected: write a separate `phases/<parent>/children/<child>/current-waypoint.json`. Rejected because it doubles the source-of-truth surface; one waypoint with conditional rendering is simpler.)

### D3. `/kbd-next-child` accepts both implicit and explicit advance

Implicit: no args → step to the next entry in `childPhases[]` after the current pointer. Explicit: `<name>` → jump to that name. Rationale: a long-running phase with many children may want random access, not just linear traversal.

### D4. Invariants enforced *before* write

Spec req 4 requires the writer-side invariant check. Both scripts compute the proposed waypoint object in memory (via `jq`), then validate `childPointer ∈ childPhases` and `childPhases is unique` against the proposed object, and abort before `mv`-renaming the temp file into place. This is the same defense-in-depth pattern used in `/kbd-new-phase`'s malformed-waypoint check.

### D5. Hook fire ordering

`/kbd-new-child`:
1. Validate name + preconditions
2. Create child dir + write files
3. Update waypoint atomically (with the new child appended and pointer set)
4. Fire `child:before` for the new child

`/kbd-next-child`:
1. Validate args + preconditions
2. Compute new pointer
3. If prior pointer was non-null, fire `child:after` for the prior child *with the old waypoint still on disk* (so hooks reading state see the closing context)
4. Update waypoint atomically
5. Fire `child:before` for the new active child

This ordering makes `child:after` hooks "look back at the closing child" and `child:before` hooks "look at the new active child" — symmetric with `phase:after` (kbd-reflect) and `phase:before` (kbd-new-phase / kbd-next-phase).

### D6. Index and total semantics for child hook fires

`KBD_HOOK_INDEX` = 1-based position of the *new* child in `childPhases[]`. `KBD_HOOK_TOTAL` = length of `childPhases[]` *after* the operation. For `/kbd-new-child`, total grows by 1. For `/kbd-next-child`, total is unchanged. This lets the default report-progress hook render `child <name> [3/5]` meaningfully.

### D7. No deep nesting fallback

If a future writer ever populates `parentPhase` on a row that is itself reached via the project-level waypoint's `childPointer`, this skill's scripts will recognise it but refuse to descend further (`/kbd-new-child` refuses if the active context is itself a child). One level of nesting only.

## Implementation Sketch

Each script ~80 LOC. Both source `shared/lib/waypoint.sh` + `shared/lib/hooks.sh`. Both use `jq` for atomic waypoint rewrites preserving unknown fields.

### `kbd-new-child.sh` shape

```sh
#!/usr/bin/env bash
set -euo pipefail
die() { printf 'kbd-new-child: %s\n' "$*" >&2; exit 1; }

name="${1:?usage: kbd-new-child <name> [goals…]}"; shift; goals=("$@")
validate_name "$name"
wp=".kbd-orchestrator/current-waypoint.json"
[[ -f "$wp" ]] || die "no active phase — run /kbd-new-phase first"
jq -e . "$wp" >/dev/null || die "malformed waypoint"

parent="$(jq -r '.phase // ""' "$wp")"
[[ -n "$parent" ]] || die "no active phase"
[[ -z "$(jq -r '.parentPhase // ""' "$wp")" ]] || die "currently inside a child phase; only one nesting level supported"

children="$(jq -c '.childPhases // []' "$wp")"
echo "$children" | jq -e --arg n "$name" 'any(. == $n)' >/dev/null \
  && die "child '$name' already exists; try /kbd-next-child $name"

child_dir=".kbd-orchestrator/phases/$parent/children/$name"
[[ -e "$child_dir" ]] && die "$child_dir already exists on disk"
mkdir -p "$child_dir"

# Write goals.md + progress.json atomically (mirrors kbd-new-phase.sh)
write_goals "$child_dir/goals.md" "${goals[@]}"
write_progress "$child_dir/progress.json" "$name" "$parent"

# Compute new childPhases + childPointer, validate invariants, atomic mv
new_children="$(echo "$children" | jq --arg n "$name" '. + [$n]')"
echo "$new_children" | jq -e 'length == (unique | length)' >/dev/null \
  || die "internal: would write duplicate children"

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq --argjson cp "$new_children" --arg ptr "$name" --arg now "$now" --arg parent "$parent" '
  .childPhases      = $cp |
  .childPointer     = $ptr |
  .currentTask      = ("run kbd-assess for " + $parent + "/" + $ptr) |
  .exactNextCommand = ("/kbd-assess " + $parent + "/" + $ptr) |
  .updatedAt        = $now
' "$wp" > "$wp.tmp"
mv -f "$wp.tmp" "$wp"

# Hook fire (after waypoint is settled)
total="$(echo "$new_children" | jq 'length')"
index="$total"     # new child is always last
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
kbd_hooks_fire child before "$name" "$index" "$total"

printf '\nCompleted kbd-new-child — %s/%s ready for /kbd-assess\n' "$parent" "$name"
```

### `kbd-next-child.sh` shape

```sh
#!/usr/bin/env bash
set -euo pipefail
die() { printf 'kbd-next-child: %s\n' "$*" >&2; exit 1; }

target="${1:-}"
wp=".kbd-orchestrator/current-waypoint.json"
[[ -f "$wp" ]] || die "no active phase"
jq -e . "$wp" >/dev/null || die "malformed waypoint"

parent="$(jq -r '.phase' "$wp")"
children="$(jq -r '.childPhases[]' "$wp")"
[[ -n "$children" ]] || die "no children defined — run /kbd-new-child first"
prior="$(jq -r '.childPointer // ""' "$wp")"

# Compute next
if [[ -n "$target" ]]; then
  echo "$children" | grep -qFx "$target" \
    || die "no such child: $target (available: $(echo "$children" | tr '\n' ' '))"
  next="$target"
else
  # Implicit: step after prior in $children
  if [[ -z "$prior" ]]; then
    next="$(echo "$children" | head -n 1)"
  else
    after="$(echo "$children" | awk -v p="$prior" 'p_seen{print;exit} $0==p{p_seen=1}')"
    [[ -n "$after" ]] || die "already on last child — run /kbd-reflect, then /kbd-next-phase"
    next="$after"
  fi
fi

# Fire child:after for closing child while old waypoint is still on disk
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
total="$(echo "$children" | wc -l | tr -d ' ')"
if [[ -n "$prior" ]]; then
  prior_index="$(echo "$children" | grep -nFx "$prior" | cut -d: -f1)"
  kbd_hooks_fire child after "$prior" "$prior_index" "$total"
fi

# Update waypoint atomically
now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq --arg ptr "$next" --arg now "$now" --arg parent "$parent" '
  .childPointer     = $ptr |
  .currentTask      = ("run kbd-assess for " + $parent + "/" + $ptr) |
  .exactNextCommand = ("/kbd-assess " + $parent + "/" + $ptr) |
  .updatedAt        = $now
' "$wp" > "$wp.tmp"
mv -f "$wp.tmp" "$wp"

# Fire child:before for new active child
next_index="$(echo "$children" | grep -nFx "$next" | cut -d: -f1)"
kbd_hooks_fire child before "$next" "$next_index" "$total"

printf '\nCompleted kbd-next-child — now on %s/%s\n' "$parent" "$next"
```

## Risks

1. **`childPointer` invariant under concurrent writes.** Two skills updating childPhases at the same time can race; one's write wins, the loser's child is lost. Same model as `/kbd-new-phase`'s atomic mv — acceptable, detected by `/kbd-status`.
2. **`/kbd-status` rendering of children.** Already specified in `kbd-status-worktree-awareness` (change 2's spec). This change writes the fields; `/kbd-status` reads them — no new work required here.
3. **`/kbd-assess` etc. don't yet know about child scope.** Today they assess the active *phase*. After this change, they SHOULD descend into `phases/<parent>/children/<active-child>/` when `childPointer` is non-null. Filed as a follow-up — child-scoped assessment is a behavior change in those skills, not in these two. The skill output still works (the parent-scoped assessment goes to the parent's dir); the user simply doesn't get child-scoped assess yet.

## Alternatives Considered

- **Separate child waypoint files.** Rejected per D2 — doubles source-of-truth surface.
- **Allow arbitrary nesting depth.** Rejected — adds complexity for an unclear use case. Single-level nesting matches the user's "sub-process" framing.
- **Auto-advance after `/kbd-reflect` on a child.** Tempting but creates implicit behavior; explicit `/kbd-next-child` is more predictable.
