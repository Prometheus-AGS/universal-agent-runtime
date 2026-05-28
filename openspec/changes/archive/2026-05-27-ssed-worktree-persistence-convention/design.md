## Context

This repository, and the multi-tool AI workflow that surrounds it (Claude Code, Roo, Cursor, Codex, OpenCode), all read configuration from a checked-in `.claude/` directory at the repo root: `.claude/settings.local.json`, `.claude/commands/`, `.claude/skills/`. That directory is part of the working tree and is committed.

A separate, unrelated convention has crept in: git worktrees being created under `.claude/worktrees/<name>/`. Two of those are currently live (`.claude/worktrees/adoring-booth-312094` is the one this change is being authored in). Putting worktrees inside `.claude/` is wrong on three independent axes:

1. **Namespace collision** — a casual rm-rf of "worktrees" near the `.claude/` config makes it easy to delete real configuration; a sloppy glob in tooling could include worktree files in the index.
2. **Tool confusion** — every tool that reads `.claude/` for config now has to know to skip a sibling subdirectory that contains *complete checkouts of the repo it's already in*.
3. **Path semantics** — worktrees represent *peers* of the current checkout, not children of the repo's configuration. Nesting them inside the repo expresses the wrong relationship.

`~/.claude/worktrees/` already has two siblings (`confident-wilbur-c27abe`, `musing-sinoussi-09cea6`), proving the developer's instinct to use that location — what's missing is a convention, tooling, and guard rails to make it the default for *this* repo.

The change is intentionally narrow. It produces convention + tooling + guard rails, and explicitly carves out the currently active in-repo worktree so we don't destabilize work in flight.

## Goals / Non-Goals

**Goals**
- Every git worktree for this repo created **after this change lands** lives under `~/.claude/worktrees/`.
- A single helper script (`scripts/worktree-new.sh`) is the documented, idiomatic way to create one. It refuses bad paths and bad names.
- The convention is announced to every AI tool that reads this repo via `CLAUDE.md` and `AGENTS.md`.
- A `.gitignore` rule and an advisory pre-commit hook make the *wrong* location actively painful to use.
- KBD orchestrator surfaces the active worktree's path in `/kbd-status` so a developer who lands in the wrong tree sees it immediately.

**Non-Goals**
- **No relocation of existing worktrees.** The active worktree (`.claude/worktrees/adoring-booth-312094`) and any other live in-repo worktrees stay exactly where they are. Migration risks losing uncommitted state mid-phase and offers no upside that justifies it.
- **No enforcement via blocking hook.** The pre-commit hook is *advisory* — a warning, not a refusal. We don't want to make this repo's commit story brittle for a convention that the helper script already enforces at creation time.
- **No language switch.** The helpers are POSIX shell, not Node/TS/Python, because they wrap `git worktree` and need to run on a fresh checkout before any toolchain bootstrap has happened.
- **No new dependency on `realpath`/`coreutils`.** macOS ships BSD utilities; the scripts use `cd … && pwd -P` for canonicalization rather than `realpath -e`.

## Decisions

### D1. Shell, not Node/TypeScript

The helpers run before `bun install`, before `cargo build`, and on the cleanest possible checkout. A shell script with no runtime prereqs is the right substrate. The trade-off is parsing/UX limitations vs. a TS CLI, but the scripts have a tiny surface area (three commands, ~30 LOC each) and don't merit a toolchain.

### D2. `~` expansion is the script's job, not git's

`git worktree add` does **not** expand `~` — it takes the path literally. The scripts therefore expand `${HOME}/.claude/worktrees/<name>` themselves before invoking git. Tests must cover the case where `$HOME` is unset (CI containers do this).

### D3. Path canonicalization without `realpath`

To reject paths that resolve inside the repo, the script computes:

```sh
repo_root="$(git rev-parse --show-toplevel)"
canon_target="$(cd "$(dirname "$resolved")" 2>/dev/null && pwd -P)/$(basename "$resolved")"
case "$canon_target" in
  "$repo_root"/*) die "refusing to create worktree inside the repo tree" ;;
esac
```

This works on macOS/Linux without GNU `coreutils`.

### D4. Seeding `.claude/settings.local.json` into the new worktree

A new worktree starts with no `.claude/settings.local.json` of its own (the file is `.gitignore`'d in most setups). To avoid surprising the developer with empty permissions, the script copies the current worktree's `settings.local.json` into the new one *if it exists*. We **do not** copy `.claude/skills/` or `.claude/commands/` — those are repo-tracked and `git worktree add` already gives the new tree the right versions via the working copy.

### D5. Advisory hook, not blocking hook

Implementation: add a single line to `.git/hooks/pre-commit` (or `core.hooksPath` if the repo uses one) that runs `git diff --cached --name-only | grep -q '^\.claude/worktrees/'` and prints a warning when matched. It does not `exit 1`. The repository's `scripts/install-hooks.sh` (if/when it exists) becomes the install vehicle; until then the convention is documented in `CONTRIBUTING.md` as a manual one-liner. (Out of scope for this change to add a hook installer — track as a follow-up.)

### D6. KBD `worktreeRoot` field is optional and additive

`.kbd-orchestrator/project.json` gains `worktreeRoot` as an *optional* field. Tools that don't know about it ignore it (forward compatibility). Tools that do know about it use it to validate or to render a friendly status line. Default value is the string `${HOME}/.claude/worktrees` (literal, with the env expansion left to the consumer) so the file remains portable across users.

### D7. CLAUDE.md ↔ AGENTS.md sync via the rule-injector skill (deferred)

A later change in this phase (`ssed-kbd-agent-rules-injector`) introduces a fenced-region rewriter for `CLAUDE.md` and `AGENTS.md`. **This change does not depend on that skill.** It writes the "Worktree convention" section once, by hand, into both files. When the injector lands, the convention block can be migrated into a managed fenced region; that migration is part of the injector change, not this one.

### D8. Naming the helpers

`worktree-new.sh` / `worktree-list.sh` / `worktree-rm.sh` mirror the `git worktree add|list|remove` verbs. The `.sh` extension is kept so a casual `ls scripts/` makes the language obvious; the shebang is `#!/usr/bin/env bash` (not `sh`) to allow `set -euo pipefail` and `[[ ]]`. Bash 3.2 (macOS default) is the minimum target.

## Implementation Sketch

```
scripts/
  worktree-new.sh        # 30–40 LOC: validate, mkdir -p, git worktree add, seed settings
  worktree-list.sh       # 10 LOC: git worktree list --porcelain | filter prefix
  worktree-rm.sh         # 20 LOC: validate prefix, git worktree remove

CLAUDE.md                # add "Worktree convention" section
AGENTS.md                # add identical section (kept in sync by hand for now)
.gitignore               # add /.claude/worktrees/
.kbd-orchestrator/project.json   # add "worktreeRoot": "${HOME}/.claude/worktrees"
```

The `worktree-new.sh` happy path:

```sh
#!/usr/bin/env bash
set -euo pipefail

name="${1:?usage: worktree-new.sh <name> [--base <ref>]}"
shift || true
base="HEAD"
if [[ "${1:-}" == "--base" ]]; then base="${2:?--base requires a ref}"; fi

root="${HOME:?HOME not set}/.claude/worktrees"
target="$root/$name"
repo_root="$(git rev-parse --show-toplevel)"

[[ -e "$target" ]] && { echo "exists: $target" >&2; exit 1; }
case "$target" in "$repo_root"/*) echo "target inside repo: $target" >&2; exit 1;; esac

mkdir -p "$root"
git worktree add "$target" "$base"

src="$repo_root/.claude/settings.local.json"
[[ -f "$src" ]] && cp "$src" "$target/.claude/settings.local.json"

echo "created: $target"
```

## Migration

Two existing in-repo worktrees are left in place (per goals/non-goals). No data migration. Developers may, at their leisure, `git worktree remove` an old in-repo worktree and `scripts/worktree-new.sh` a fresh one — that's a user-driven action, not part of this change.

## Risks

1. **`$HOME` unset in CI** — the script `die`s with a clear message; CI jobs that need worktrees will have to set `HOME` (they do today, almost universally).
2. **`pwd -P` resolves symlinks differently than user expectation** — if a developer keeps `~/.claude` itself as a symlink, the canonicalized target may end up outside `~`. Documented; the in-repo check still works because it canonicalizes both sides.
3. **Advisory hook ignored** — by design. The combination of `.gitignore` + helper-script-as-only-entry-point makes the wrong path hard to reach; the hook is a third line of defense, not the first.
4. **Two siblings already at `~/.claude/worktrees/`** — created by other repos. The helper's collision check is per-name, so cross-repo namespace collisions are possible. Out of scope for this change; resolution would be a `<repo>/<name>` two-level layout, deferred until it actually bites.
5. **Bash 3.2 on macOS** — avoids `mapfile`, associative arrays, `&>>`. Scripts kept simple enough that this is a non-issue.

## Alternatives considered

- **Just document it; no scripts** — rejected. Conventions without tooling decay.
- **Block the bad path with a real pre-commit refusal** — rejected (D5). Brittle, and unnecessary given creation-time enforcement.
- **Use `git worktree add --lock` to mark the new worktree** — orthogonal; doesn't help with path discipline.
- **Make `worktreeRoot` mandatory in `project.json`** — rejected (D6). Optional + additive preserves backward compatibility with every cross-tool consumer.
