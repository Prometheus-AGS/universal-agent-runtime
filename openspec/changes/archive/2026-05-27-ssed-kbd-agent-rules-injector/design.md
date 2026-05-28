## Context

Skill ships in the same orchestrator skill set as everything else in W1/W2. The two rule sets were sourced via web search performed during this change's design step — content is cached in `references/rules-cache.md` with URLs and fetch dates so a later `--refresh` can validate sources without re-running the search.

The fenced-region pattern is similar to git merge markers or doc-injection tools (e.g. `gh-action-readme`). It must survive arbitrary surrounding content because `CLAUDE.md` files are user-edited.

## Goals / Non-Goals

**Goals**
- Idempotent fenced-region rewrites in `CLAUDE.md` / `AGENTS.md`.
- Both rule sets present with explicit attribution.
- Source URLs cached locally so refresh is auditable.
- `--dry-run` shows the diff without modifying.
- Graceful handling of corrupt marker states.

**Non-Goals**
- Auto-invocation hook. Future change could wire this to `phase:before`; not here.
- Per-rule edits via CLI flags (e.g. "skip Karpathy rule 3"). The rules-pack is monolithic; users edit the rule body in `rules-cache.md` + the template.
- Multi-pack support (different rule packs per project). Single pack, single fenced region.

## Decisions

### D1. Marker format includes version

`<!-- agent-rules:start v1 -->` lets future packs introduce a v2 without colliding with v1 regions. The injector only ever manages markers at its own current version; an older v1 region stays untouched unless the operator runs the v2 injector explicitly. Crude but works.

### D2. Sed-style replacement, not jq

`CLAUDE.md` is markdown, not JSON. The injector uses a small awk pipeline to split the file at the markers and re-glue with the new content. Awk is in every POSIX environment; no new dependency.

### D3. Refresh validates by anchor keyword, not full-text diff

A live HTML page can change formatting without changing semantics; a strict diff would fire false positives. Each source URL has a documented anchor keyword (e.g. "Think Before Coding" for the Karpathy AIBuilderClub page) — refresh confirms the keyword still appears in the fetched body, otherwise warns.

### D4. Cache file is the source of truth for content

`rules-cache.md` holds the canonical rule text. The injector reads from there to build the fenced region. `--refresh` updates the fetch dates in the cache but does NOT auto-update the rule text — the human reviewer does that based on the warning. Rationale: web pages change wording subtly; we don't want silent drift.

### D5. Combined output in a single fenced region

One fenced block, two sub-sections. Simpler than two regions; the user reads them together because they're complementary (think-first + workflow).

### D6. Dry-run uses unified-diff output

`diff -u <existing> <new>` — operators recognise this format from git. No need for a custom renderer.

### D7. Cross-repo first-customer

This UAR repo's `CLAUDE.md` / `AGENTS.md` are the first target. Running the injector against them mid-phase exercises the skill end-to-end and produces a visible artifact — also closes part of change 8 (the rules block is what change 8's UI/UX routing rules will sit *near*, not inside).

## Implementation Sketch

### `references/rules-cache.md` (verbatim)

```
# Agent rules cache

Source of truth for `/kbd-inject-agent-rules`. Edit here if a source page
changes wording.

## Karpathy — Think-first principles

Last fetched: 2026-05-27

1. **Think Before Coding** — State assumptions explicitly, surface
   ambiguity, present tradeoffs, ask for clarification rather than
   guessing silently.

2. **Simplicity First** — Write the minimum code that solves the
   problem; no speculative features, abstractions, or over-engineering.

3. **Surgical Changes** — Touch only what the request requires; do not
   "improve" adjacent code, comments, or formatting.

4. **Goal-Driven Execution** — Operate against concrete success
   criteria, not step-by-step micro-instructions. LLMs loop well toward
   goals.

Sources:
- https://github.com/forrestchang/andrej-karpathy-skills (anchor: "Think Before Coding")
- https://www.aibuilderclub.com/blog/karpathy-claude-md-rules (anchor: "Karpathy")
- https://lucaberton.com/blog/karpathy-claude-md-llm-coding-principles-2026/ (anchor: "CLAUDE.md")

## Boris Cherny — Claude Code workflow principles

Last fetched: 2026-05-27

1. **Plan Mode First** — Iterate the plan in Plan Mode until it's right;
   only then switch to auto-accept edits.

2. **CLAUDE.md as accumulated knowledge** — Use CLAUDE.md as a long-lived
   project-level instruction file; accumulate rules, constraints, and
   lessons over time.

3. **Verification + feedback loops** — Give the agent a way to verify
   its work; closing the loop is reportedly worth a 2-3x quality bump.

4. **Code quality matters for AI too** — Partially-migrated codebases
   confuse models as much as humans; finish migrations completely.

Sources:
- https://howborisusesclaudecode.com/ (anchor: "Plan Mode")
- https://newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny (anchor: "Claude Code")
- https://www.lennysnewsletter.com/p/head-of-claude-code-what-happens (anchor: "Boris")
```

### `references/template.md`

```
<!-- agent-rules:start v1 -->
## Agent rules

> Auto-managed by `/kbd-inject-agent-rules`. Re-running the skill
> overwrites everything between the markers. Edit the cache at
> `kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/rules-cache.md`
> if you need to change the content.

### Think-first principles (Karpathy)

1. **Think Before Coding** — State assumptions explicitly, surface
   ambiguity, present tradeoffs, ask for clarification rather than
   guessing silently.
2. **Simplicity First** — Write the minimum code that solves the
   problem; no speculative features.
3. **Surgical Changes** — Touch only what the request requires.
4. **Goal-Driven Execution** — Operate against concrete success criteria.

### Workflow principles (Claude Code, Boris Cherny)

1. **Plan Mode First** — Iterate the plan until it's right; only then
   auto-accept edits.
2. **CLAUDE.md as accumulated knowledge** — Long-lived project rules,
   not chat scratch.
3. **Verification + feedback loops** — Give the agent a way to verify
   its work (2-3× quality bump).
4. **Code quality matters for AI too** — Partially-migrated codebases
   confuse models the same way they confuse humans.

Verbatim sources + fetch dates in
`kbd-process-orchestrator/shared/references/rules-cache.md` (managed by
the same skill).
<!-- agent-rules:end -->
```

### `kbd-inject-agent-rules.sh` (high level)

```sh
#!/usr/bin/env bash
set -euo pipefail

# Parse --target, --path, --refresh, --dry-run
# Resolve target file paths
# Optional: refresh by curl-ing source URLs and validating anchor keywords
# Build new fenced region content from references/template.md
# For each target:
#   detect markers, validate consistency
#   produce new file via awk pipeline
#   on --dry-run: print `diff -u` and skip mv
#   else: atomic mv
```

## Risks

1. **Stale cached content over time.** Source pages drift; refresh validation only checks anchor keywords, not full wording. Mitigation: anchor keywords cover the core phrasing; the human reviewer reads the warning and updates `rules-cache.md` deliberately.
2. **Multiple fenced regions in the wild.** Spec req covers the case (multi-start markers → refuse). Operators have to reconcile by hand.
3. **`CLAUDE.md` deletion in the middle.** If an operator accidentally deletes the end marker, the skill refuses to write. Repair instructions are in the spec.
4. **`awk` portability.** Used only POSIX features (no GNU-isms).
5. **`curl` absence for `--refresh`.** Refresh degrades to a single warning; non-refresh mode still works.

## Alternatives Considered

- **Edit-in-place with sed.** Rejected — fragile with markers and easy to break with shell-metachar content.
- **Use a templating engine (`mustache`, `gomplate`).** Rejected — new dependency for one template.
- **Per-file fenced regions tied to each source.** Rejected — combined region is more readable and easier to audit.
