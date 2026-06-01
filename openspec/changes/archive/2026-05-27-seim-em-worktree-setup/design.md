## Context

`prometheus-entity-management` lives at `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`, hosting `main` plus assorted workstream branches. The prior phase's worktree convention (`uar-worktree-convention`) requires that any new worktree be placed under `~/.claude/worktrees/`, never inside a target repo's working tree. This change provisions exactly one such worktree, against a brand-new topic branch, for this phase's W3+ TypeScript work.

This is the smallest non-trivial change in the phase. It's mostly procedural; the design exists to nail down four small but consequential decisions and to capture the rollback story.

## Goals / Non-Goals

**Goals**
- One persistent worktree at `~/.claude/worktrees/seim-entity-management` checked out on a fresh topic branch from `origin/main`.
- A machine-readable record (`worktrees.json` in the phase dir) so subsequent changes know exactly where to write.
- Idempotent provisioning: re-running the change script after success errors out cleanly without corrupting state.
- The verification step proves the worktree is usable for code work (`pnpm install` succeeds — not just `git rev-parse`).

**Non-Goals**
- No code committed into the worktree by this change.
- No port of UAR's `scripts/worktree-new.sh` to entity-management (UAR-specific guards).
- No worktree for skill-system (fresh branches per workstream there).
- No multi-worktree strategy in this phase.

## Decisions

### D1. Branch name: `feat/seim-entity-management-impl`

Mirrors the phase's `seim-` prefix so commit logs are searchable: a single `git log feat/seim-entity-management-impl..origin/main` against entity-management later in the phase shows exactly the phase's contribution.

Alternative considered: per-change branches (`feat/seim-em-surreal-live-adapter-impl`, etc.). Rejected — the OpenSpec archive already provides per-change traceability; per-change branches multiply review surface for no audit win.

### D2. Worktree state: dedicated `worktrees.json` sidecar (Option B)

Three options were on the table in the proposal:

- **A. Phase `execution.md`** — central but mutates a file `/kbd-execute` also writes to; race risk + cognitive overhead.
- **B. Dedicated `worktrees.json`** — sidecar to `progress.json`, only ever read/written by worktree-related changes. **Chosen.**
- **C. New `worktrees: {}` field in `progress.json`** — works, but bloats progress.json which is already growing.

The sidecar is the cleanest separation: `worktrees.json` documents *where code lives*; `progress.json` documents *what work has been done*. Different concerns, different files. Schema:

```jsonc
{
  "$comment": "Per-repo worktrees provisioned by this phase. Read by changes that need to know where to write code.",
  "worktrees": {
    "prometheus-entity-management": {
      "path": "~/.claude/worktrees/seim-entity-management",
      "expandedPath": "/Users/<user>/.claude/worktrees/seim-entity-management",
      "branch": "feat/seim-entity-management-impl",
      "baseCommit": "<sha from origin/main at provisioning>",
      "baseSha": "<same>",
      "provisionedAt": "<ISO-8601 UTC>",
      "provisionedBy": "seim-em-worktree-setup"
    }
  }
}
```

Both `path` (with `${HOME}` left literal-ish for portability) and `expandedPath` (concrete) are written — consumers can pick which they want without re-implementing the expansion.

### D3. `scripts/worktree-new.sh` is NOT ported here

The UAR helper is UAR-specific: its `is_descendant` check resolves `git rev-parse --show-toplevel` against UAR's repo root to reject paths that would land inside UAR's tree. Porting it to entity-management means parameterising the helper, lifting the repo-specific assumption, and dropping it into a "common scripts" location — which doesn't exist yet across these repos.

Concretely: this change uses **bare `git worktree add -b`** invoked from inside the entity-management checkout. Three lines, no abstraction, no helper. A future change can port the helper if and when the pattern repeats often enough to justify the indirection.

### D4. Rollback strategy

If the worktree gets corrupted mid-phase (rare but possible — disk pressure, accidental `rm -rf` of `.claude/worktrees/`, etc.):

```sh
# In prometheus-entity-management:
git worktree prune                                    # cleans stale refs
git worktree remove -f ~/.claude/worktrees/seim-entity-management 2>/dev/null || true
rm -rf ~/.claude/worktrees/seim-entity-management

# Re-provision per §1.1-1.4 of tasks.md.
# If the topic branch had unpushed commits, recover from reflog:
git reflog show feat/seim-entity-management-impl
git branch feat/seim-entity-management-impl-recovered <reflog-sha>
```

Branch contents are git-tracked, so recovery is bounded by reflog retention (90 days default). Per-change OpenSpec archives in this UAR repo preserve the *intent* of the lost work; reconstructing it from the spec + design is feasible even with total worktree loss.

### D5. Idempotency

The provisioning command is run-once. If repeated:

- `git worktree add ...` exits non-zero with "directory already exists" — caught and reported.
- `worktrees.json` is overwritten with current state (no-op if already correct).
- `execution.md` note is idempotent if the operator uses `grep -q` before appending; otherwise duplicates.

Not strictly idempotent end-to-end, but the only state that materially matters (the on-disk worktree) is checked first and errors cleanly. Acceptable.

### D6. `pnpm install` in the worktree

Mentioned in the proposal as a verification step. Decision: **required**, not optional. Reasons:

1. The verification proves the worktree is *workable* — `git rev-parse` only proves it exists.
2. Cached node_modules behavior with pnpm + worktrees can be subtle (symlinked store). Catching install errors here saves debugging time at change 4.
3. `pnpm install` against a committed lockfile is fast (~10 s warm cache) — negligible cost for the verification value.

The install happens in the new worktree, not in the original entity-management checkout — those are independent dependency graphs even though they share a pnpm store.

### D7. `worktrees.json` is read by every downstream change

Establishing the convention here means subsequent changes' `kbd-new-phase`-style scripts (or the agent driving them) must:

1. Look up the relevant repo's entry in `worktrees.json` to know where to write code.
2. Refuse to operate if the entry is absent — fail loudly rather than write into the wrong checkout.

The "refuse if absent" check belongs in each consuming change, not in this provisioning change. Documented as a convention; enforcement is per-consumer.

## Implementation Sketch

```sh
# Pre-flight
cd /Users/gqadonis/Projects/prometheus/prometheus-entity-management
git fetch origin
git checkout main
git pull --ff-only origin main
base_sha="$(git rev-parse origin/main)"

# Provision (atomic branch + worktree)
git worktree add -b feat/seim-entity-management-impl \
  ~/.claude/worktrees/seim-entity-management origin/main

# Verify
cd ~/.claude/worktrees/seim-entity-management
test "$(git rev-parse --show-toplevel)" = "$HOME/.claude/worktrees/seim-entity-management"
test "$(git rev-parse HEAD)" = "$base_sha"
git status --short | grep -q . && exit 1 || true  # clean
pnpm install --frozen-lockfile

# Record state (back in UAR worktree)
cd /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/worktrees/adoring-booth-312094
mkdir -p .kbd-orchestrator/phases/submodule-entity-management-implementation
cat > .kbd-orchestrator/phases/submodule-entity-management-implementation/worktrees.json <<JSON
{
  "$comment": "Per-repo worktrees provisioned by this phase.",
  "worktrees": {
    "prometheus-entity-management": {
      "path": "\${HOME}/.claude/worktrees/seim-entity-management",
      "expandedPath": "$HOME/.claude/worktrees/seim-entity-management",
      "branch": "feat/seim-entity-management-impl",
      "baseCommit": "$base_sha",
      "baseSha": "$base_sha",
      "provisionedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
      "provisionedBy": "seim-em-worktree-setup"
    }
  }
}
JSON

# execution.md note (append; do NOT clobber)
if ! grep -q '^## Worktree provisioning' \
     .kbd-orchestrator/phases/submodule-entity-management-implementation/execution.md \
     2>/dev/null; then
  cat >> .kbd-orchestrator/phases/submodule-entity-management-implementation/execution.md <<MD

## Worktree provisioning

- Repo: \`prometheus-entity-management\`
- Worktree: \`~/.claude/worktrees/seim-entity-management\`
- Branch: \`feat/seim-entity-management-impl\`
- Base commit: \`$base_sha\` (from \`origin/main\` at provisioning time)
- Provisioned by: \`seim-em-worktree-setup\`

Subsequent changes that write code to entity-management MUST resolve the
worktree path from \`worktrees.json\` (sibling to \`progress.json\`) and
refuse to operate if the entry is absent.
MD
fi
```

## Risks

1. **Operator runs the provisioning twice.** Mitigated by D5 — `git worktree add` will error on the second run; `worktrees.json` overwrite is harmless.
2. **`pnpm install` fails on the verification step.** Surfaces a real problem (corrupted lockfile, network issue) before any actual work depends on it. Rare; rerun after fixing root cause.
3. **`origin/main` moves between fetch and the worktree-add.** Improbable in a single-operator setup; standard git race. If it bites, the recorded `baseCommit` reflects the *fetched* SHA, not the current `origin/main`. Acceptable — the worktree is consistent with what was fetched.
4. **Downstream change skips reading `worktrees.json` and writes to the wrong checkout.** Convention enforcement is per-consumer (D7). Each downstream change's design should call this out explicitly; the orchestrator's `SKILL.md` may eventually need a "Phase sidecar files" section.
5. **`execution.md` may not yet exist if `/kbd-execute` hasn't run for this phase.** The append script uses `>>` which creates the file if needed; if `/kbd-execute` later writes the same file, the prior content survives (appends compose).

## Alternatives Considered

- **Provision into `prometheus-entity-management`'s own `.worktrees/`** — rejected (violates `uar-worktree-convention`).
- **Use `git clone --shared` into a sibling directory** — rejected (loses the git-worktree semantics: ref isolation, branch atomicity).
- **One worktree per change in W3+** — rejected (massive overhead, no audit win over per-change OpenSpec archives).
- **Record worktree state in `progress.json`** — rejected per D2.
