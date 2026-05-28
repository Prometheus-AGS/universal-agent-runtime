## Context

Two simultaneous goals: (1) ship the UI/UX routing discipline into this UAR repo, and (2) parameterise the injector skill so the same code path serves both the agent-rules pack (change 7) and this new pack — and any future pack.

The skill roster content is sourced via web search performed during this change. URLs + anchor keywords cached in `cache-uiux-routing.md` so a future `--refresh` can validate.

## Goals / Non-Goals

**Goals**
- Roster cache + fenced region landed in this UAR repo.
- Injector gains `--pack` flag, backward-compatible default.
- Refactored injector internals are clean enough to add a third pack later (e.g. memory-policy, threat-model) with two new files only.
- Spec scenarios mapped 1:1 to implementation steps so verify is trivial.

**Non-Goals**
- No automated enforcement (no pre-commit hook refusing UI code without the prior memory-recall). Discipline is doc-only.
- No installation of the cited external skills (UI/UX Pro Max, Impeccable, etc.). Roster names them; install is operator-driven.
- No multi-version migration of existing `agent-rules:start v1` regions. v1 stays v1.

## Decisions

### D1. Parameterise via `--pack`, not rename the skill

The skill stays `kbd-inject-agent-rules`. Rationale: the user's phase-defining request frames it as a single skill that handles multiple injection use cases ("when activated"). Renaming would break the just-landed change 7. The `--pack` flag opens the door to many packs while preserving the default behavior.

### D2. Pack-specific files live next to the existing ones

```
skills/kbd-inject-agent-rules/references/
├── template-agent-rules.md     (renamed from template.md)
├── cache-agent-rules.md        (renamed from rules-cache.md)
├── template-uiux-routing.md    (new — fenced region body)
└── cache-uiux-routing.md       (new — roster + source URLs)
```

Backward compatibility: the renamed files keep a symlink or copy at the old name for one release (or, more simply, the renamed targets are the new canonical paths and the script reads from them).

### D3. Marker name derives from pack name

Pack `agent-rules` → markers `agent-rules:start v1` / `agent-rules:end` (unchanged).
Pack `uiux-routing` → markers `uiux-routing:start v1` / `uiux-routing:end`.

This makes the markers self-describing in the file. A regex like `<!-- (\w[\w-]*):start v\d+ -->` discriminates them.

### D4. Pack version is per-pack, not per-skill

Each pack carries its own `vN` in the marker. The agent-rules pack starts at `v1`; the uiux-routing pack starts at `v1`. They're independent — bumping one doesn't bump the other.

### D5. The fenced-region detection logic is parameterised on START_MARK + END_MARK

The existing script hard-codes the marker text. Refactor: derive `START_MARK` and `END_MARK` from `--pack` value before the per-target loop. Everything else is unchanged.

### D6. Roster cache lives in *this repo*, not skill-system

The roster is project-scoped — different projects have different UI/UX skill rosters. Living in `.kbd-orchestrator/references/` (this repo) makes that explicit. The skill-system ships the *template* (a default roster) at `references/cache-uiux-routing.md`; the injector copies it into the target repo on first invocation if no project-local roster exists. (For this change, we ship the roster directly into UAR.)

### D7. The injector reads roster content from the target repo when available

When writing the `uiux-routing` region into a project, the injector prefers `.kbd-orchestrator/references/uiux-skill-roster.md` from the target repo over its own cache. This lets each project customise the roster while keeping the skill-system template as a fallback.

### D8. The discipline document mentions surreal-memory as already default-on

Because change 6 landed first, the discipline's step 1 ("consult surreal-memory via `/kbd-memory-recall`") is operational — there's a real skill behind it. We don't need to qualify it with "if available".

## Implementation Sketch

### Renames in skill-system

```
git mv references/template.md     references/template-agent-rules.md
git mv references/rules-cache.md  references/cache-agent-rules.md
```

(Done logically through the existing change 7 files — for simplicity we update the script and ship both old and new names with the old being a forwarder symlink. Or, simplest: just check for both names in the script. We pick: ship both names, prefer the new.)

### Script diff highlights

```sh
# After argument parsing, add:
pack="${pack:-agent-rules}"
case "$pack" in
  agent-rules)    template="$HERE/references/template-agent-rules.md"
                  cache="$HERE/references/cache-agent-rules.md"
                  marker_prefix="agent-rules" ;;
  uiux-routing)   template="$HERE/references/template-uiux-routing.md"
                  cache="$HERE/references/cache-uiux-routing.md"
                  marker_prefix="uiux-routing" ;;
  *)              die "--pack must be agent-rules or uiux-routing (got: $pack)" ;;
esac

# Back-compat:
[[ -f "$template" ]] || template="$HERE/references/template.md"
[[ -f "$cache"    ]] || cache="$HERE/references/rules-cache.md"

START_MARK="<!-- ${marker_prefix}:start v1 -->"
END_MARK="<!-- ${marker_prefix}:end -->"

# uiux-routing pack: prefer roster from target repo if present (D7)
if [[ "$pack" == "uiux-routing" && -f "$project_path/.kbd-orchestrator/references/uiux-skill-roster.md" ]]; then
  cache="$project_path/.kbd-orchestrator/references/uiux-skill-roster.md"
fi
```

The rest of the script (marker detection, awk replacement, dry-run, idempotency) is unchanged.

### `template-uiux-routing.md`

Mirrors the doc block in the proposal verbatim, wrapped in the marker pair.

### `cache-uiux-routing.md` (template version, shipped in skill-system)

Same content as the UAR-side `.kbd-orchestrator/references/uiux-skill-roster.md`. The injector prefers the project-local copy per D7.

## Risks

1. **Roster drift over time.** Source pages change (skills get renamed, deprecated). Mitigation: cache file carries source URLs + anchor keywords; future `--refresh` flow (already implemented for agent-rules pack) covers this pack too.
2. **Two fenced regions interleaved.** Spec covers it — replacing one MUST not touch the other. Implementation derives markers from `--pack` and only matches its own pair.
3. **`--pack` flag confusion.** Default behavior is unchanged when omitted; existing users see no surprise. Documentation in SKILL.md explains the flag.
4. **Discipline ignored.** Doc-only enforcement is weak; an inattentive agent could skip the routing block. Mitigation: the region is short, prominent, and starts with `MUST`. Future change could add a hook that posts a reminder line on UI-file open events.

## Alternatives Considered

- **Separate skill `kbd-inject-uiux-routing`.** Rejected — duplicates 90% of the injector. `--pack` is cleaner.
- **One mega-pack covering everything (rules + routing + future).** Rejected — couples two unrelated concerns; users may want one without the other.
- **Ship the roster only as a skill-system asset (no UAR copy).** Rejected — per D6, the roster is project-scoped.
- **Enforce the discipline at lint time.** Rejected for this change — out of scope, and the discipline is process-level (consult memory, etc.) not lintable.
