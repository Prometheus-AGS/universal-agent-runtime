## Context

This change is pure documentation. The implementation IS the research process: consult each
skill/source in the prescribed order, synthesise findings, and write them to
`docs/devtools-design-notes.md` in `~/.claude/worktrees/seim-entity-management`. No TypeScript,
no JSX, no CSS. The commit must touch exactly one file path.

## Goals / Non-Goals

**Goals:**
- Execute all seven UI/UX routing discipline steps for the Entity Explorer surface
- Produce `docs/devtools-design-notes.md` with all eleven sections (§1–§11)
- Satisfy the gate so W5 and W6 can proceed
- Capture web-search citations in cache-discipline format (URL + anchor + fetch date)

**Non-Goals:**
- Implementing any component code (that is W6)
- Designing the event-bus shape (that is W5)
- Exhaustive coverage of every possible devtools pattern — one paragraph distillation is the target

## Decisions

**D1. Execution order — strictly sequential per discipline**
Steps 1 → 7 are run in the order the routing block mandates. Step 1 (memory recall) is
already seeded by the `assess:before` hook; the implementor reads `prior-context.md` before
consulting any external source.

**D2. Web-search citation format**
Every cited source entry MUST follow:
```
- <URL> | anchor: <keyword(s) that locate the relevant section on the page> | fetched: YYYY-MM-DD
```
This mirrors the `rules-cache.md` pattern established in `kbd-inject-agent-rules` and is the
format the spec requires.

**D3. Impeccable command selection for a devtools panel**
Three commands are required by the spec: `audit`, `critique`, `distill`. The `distill` command
is the highest-value choice for a devtools panel because devtools surfaces suffer most from
complexity creep (too many controls, poor information hierarchy). `polish` and `harden` are
skipped — they are pre-ship checks applicable to W6, not to this documentation pass.

**D4. Palette target — dark devtools surface**
The Entity Explorer panel will render inside a browser devtools pane or a floating panel
overlaid on a running app. Both contexts are typically dark-themed. The UI/UX Pro Max analysis
should be asked for a **dark palette** with high-contrast neutral + one accent. Font pairing
should favour monospace / code-adjacent because most of the content is entity data.

**D5. Vercel skill scope — panel component tree**
The React Best Practices + Composition Patterns skills are consulted specifically for:
- Tab panel composition (how to split a 5-tab surface cleanly)
- State lifting vs. local state per-tab
- Whether any tab needs a React context boundary
Context-sensitive rendering (e.g. only mount the Events tab when the event-bus is active) is
the primary composition question.

**D6. Distilled paragraph placement**
§8 is the paragraph that W6's implementor will read first. It is written as a direct prompt
context, not a summary of the sections above — phrasing like "When implementing the panel,
prefer…" rather than "We found that…".

**D7. Downstream implications sections (§9–§11)**
These are written as bullets with enough concrete detail to inform the design.md of W5/W6/W8
without duplicating every finding from §1–§8. One or two bullets per wave is sufficient.

**D8. Docs directory creation**
The `docs/` directory may not exist yet in `~/.claude/worktrees/seim-entity-management`.
The implementor creates it with `mkdir -p` before writing the file. The `docs/` dir is not
independently tracked (no `.gitkeep`); the file itself is the tracked artefact.

**D9. Commit scope**
The commit message is:
```
docs(explorer): preflight research notes — UI/UX routing discipline W4
```
The commit lands on `feat/seim-entity-management-impl` in the entity-management worktree.
`git diff --name-only HEAD~1 HEAD` must show only `docs/devtools-design-notes.md`.

**D10. No push until PR #1 review window**
The commit is made locally; it accumulates with the existing W3 commits. The implementor does
NOT force-push or open a new PR — the next push will be done as part of the W5/W6/W7 PR
bundle or when the user requests it.

## Risks

- **Memory recall returns no prior UI/UX decisions** — acceptable. §1 records "no prior
  UI/UX decisions found for devtools panel surfaces" as its content, which is still a valid
  section. The discipline does not require prior decisions to exist.
- **Web searches return low-signal results** — capture the best 2-3 sources found, note
  the search date, and move on. The spec requires at least one citation per section; more
  is better but not required.
- **Impeccable / UI/UX Pro Max skill not installed** — use the skill roster at
  `.kbd-orchestrator/references/uiux-skill-roster.md` to invoke them via the Agent tool
  or direct skill invocation. If a skill produces no output, document "skill consulted;
  no additional recommendations beyond §N" rather than omitting the section.
- **`docs/` commit accidentally including other files** — always use
  `git add docs/devtools-design-notes.md` (explicit path, not `git add .`) before committing.
