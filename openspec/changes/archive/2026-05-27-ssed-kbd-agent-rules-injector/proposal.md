## Why

The user's phase request: "create a skill that automatically inserts the karpathy rules and the rules from the author of claude code itself … both encourage AI tools to think before they start writing code". The skill must support `CLAUDE.md` and `AGENTS.md` and be idempotent.

Web search performed during this change confirms both rule sets (sources captured in `rules-cache.md`):

**Karpathy (4 rules)** — sourced from Karpathy's public observations distilled into the andrej-karpathy-skills repository (forrestchang/andrej-karpathy-skills) and discussed in multiple 2026 write-ups:
1. **Think Before Coding** — state assumptions explicitly, surface ambiguity, present tradeoffs, ask for clarification rather than guessing.
2. **Simplicity First** — write the minimum code that solves the problem; no speculative features.
3. **Surgical Changes** — touch only what the request requires; do not "improve" adjacent code.
4. **Goal-Driven Execution** — operate against success criteria, not step-by-step micro-instructions.

**Boris Cherny (Claude Code author) — 4 principles** distilled from interviews + how-boris-uses-claude-code material:
1. **Plan Mode First** — iterate the plan until it's right; only then auto-accept edits.
2. **CLAUDE.md as accumulated knowledge** — long-lived project rules/constraints, not chat scratch.
3. **Verification + feedback loops** — give the agent a way to verify its work (2-3× quality bump).
4. **Code quality matters for AI too** — partially-migrated codebases confuse models the same way they confuse humans; finish migrations.

Both sets have the same spirit: think + verify before mutating; resist sprawl. Combining them is high-leverage for every downstream project.

## What Changes

### New skill `kbd-inject-agent-rules`

Path: `skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/`. Components:

- `SKILL.md` — invocation contract, prerequisites, examples.
- `kbd-inject-agent-rules.sh` — POSIX bash, atomic writes, idempotent.
- `references/rules-cache.md` — verbatim rule text + source URLs + fetch date. Refreshed by `--refresh` flag.
- `references/template.md` — the fenced-region template that gets written into target files.

### Fenced-region rewrite

The injector manages a region delimited by:

```
<!-- agent-rules:start v1 -->
… content …
<!-- agent-rules:end -->
```

Where `v1` is the rules-pack version. Operations:

- First write (no markers present) → append the block at the end of the file.
- Subsequent writes (markers present) → replace the region in place. Content outside the markers is never touched.
- `--refresh` mode → re-fetches the rules from cached source URLs, updates `rules-cache.md` with new fetch date, then rewrites the fenced region.

### Argument shape

```
/kbd-inject-agent-rules [--target CLAUDE.md|AGENTS.md|both]
                        [--path <project-root>]
                        [--refresh]
                        [--dry-run]
```

Default: `--target both`, `--path .`, no refresh, write through.

### Rule pack contents

The fenced-region body merges both sets into a single "Agent rules" section. Format:

```
## Agent rules

> Auto-managed by /kbd-inject-agent-rules. Edit by re-running the skill;
> changes inside the fenced region will be overwritten on the next refresh.

### Think-first principles (Karpathy)
1. Think Before Coding — state assumptions, …
2. Simplicity First — …
3. Surgical Changes — …
4. Goal-Driven Execution — …

### Workflow principles (Claude Code, Boris Cherny)
1. Plan Mode First — …
2. CLAUDE.md as accumulated knowledge — …
3. Verification + feedback loops — …
4. Code quality matters for AI too — …

> Sources cached in shared/references/rules-cache.md
```

### `--refresh` flow

Hits the source URLs via `curl`, parses the page through a minimal regex/grep filter to confirm the rule keywords are still present, and updates the fetch date stamp in `rules-cache.md`. Refusing to update silently when a source URL no longer resolves — emits a warning so a future change can repair the source.

### Orchestrator documentation

Add the new skill to `kbd-process-orchestrator/SKILL.md` "Quick Start Commands" list.

### Non-changes

- **No automatic invocation.** The skill must be explicitly run; we don't auto-inject into every CLAUDE.md the orchestrator touches.
- **No editing of `.kbd-orchestrator/` files.** Out of scope; the agent-rules skill is for project-root context files.
- **No deletion of existing rules in target files.** Anything outside the fenced region is preserved verbatim.

## Capabilities

### New Capabilities

- `kbd-agent-rules-injector`: A `/kbd-inject-agent-rules` skill that idempotently writes a fenced-region block of 8 rules (Karpathy's 4 + Boris Cherny's 4) into `CLAUDE.md` and/or `AGENTS.md`, refreshable from cached source URLs, with `--dry-run` and `--target` flags.

### Modified Capabilities

- None.

## Impact

- **Risk**: Very low. The injector touches only its own fenced region in user-controlled files; everything else is byte-preserved. Atomic writes prevent partial state.
- **Affected files**: skill-system only. This UAR repo is a *consumer* — running the skill against this repo's `CLAUDE.md` / `AGENTS.md` is the canonical first invocation (also closes part of change 8).
- **Cross-repo**: Yes — same `prometheus-skill-system`.
- **Reversibility**: Trivial — delete the fenced region by hand if undesired; remove the skill directory.
- **Unblocks**: change 8 (`ssed-uar-uiux-skill-routing`) can re-use the same fenced-region machinery to inject UI/UX routing rules.

Source captures (date 2026-05-27):
- https://github.com/forrestchang/andrej-karpathy-skills
- https://www.aibuilderclub.com/blog/karpathy-claude-md-rules
- https://lucaberton.com/blog/karpathy-claude-md-llm-coding-principles-2026/
- https://howborisusesclaudecode.com/
- https://newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny
- https://www.lennysnewsletter.com/p/head-of-claude-code-what-happens
