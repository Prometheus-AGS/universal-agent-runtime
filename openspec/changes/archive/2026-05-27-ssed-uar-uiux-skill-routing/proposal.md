## Why

The user's phase-defining message asked for a discipline in this repo's `CLAUDE.md` / `AGENTS.md`: any UI/UX work must consult a specific roster of skills + memory + web search **before** writing code. The roster is:

- **UI/UX Pro Max** (nextlevelbuilder/ui-ux-pro-max-skill) — design system + audit; 71k★, 161 palettes, 57 font pairings, 99 UX guidelines, 25 chart types across 10 stacks.
- **Impeccable** (pbakaus/impeccable) — 1 skill, 23 commands under `/impeccable/*`: `/audit`, `/critique`, `/polish`, `/distill`, `/bolder`, `/quieter`, `/animate`, `/colorize`, `/normalize`, `/harden`, plus brand/product mode commands `typeset`, `layout`, `delight` and more (full set in the cache).
- **Anthropic frontend-design** and **ux-designer** skills.
- **Vercel React Best Practices**, **Composition Patterns**, **React Native Skills**, and **Web Design Guidelines**.
- **Surreal-memory recall** (now default-on per change 6).

The user also asked that before *any* UI/UX work the agent (a) consult surreal-memory for prior decisions, (b) run the roster's analysis, (c) summarise best practices, (d) only then write code. And specifically for change 10/11 (entity explorer + extension): research "best practices for runtime dev tools on a web page and as a Chrome extension" before planning.

This change adds two artifacts to this UAR repo:

1. A **UI/UX skill roster cache** at `.kbd-orchestrator/references/uiux-skill-roster.md` (this repo, not skill-system — the roster is project-scoped configuration).
2. A **fenced "UI/UX work routing" block** appended to this repo's `CLAUDE.md` and `AGENTS.md` (just below the agent-rules region from change 7) that any AI tool reading the file will see.

Both pieces are documented as the canonical pre-work discipline for change 10 and 11.

## What Changes

### Skill roster cache (this repo)

`.kbd-orchestrator/references/uiux-skill-roster.md` — verbatim list of skills + commands + source URLs + fetched-on date. Sections:

- **Tier 1 — Always consult**: UI/UX Pro Max, Impeccable.
- **Tier 2 — Stack-specific**: Vercel React Best Practices, Vercel Composition Patterns, Vercel React Native, Vercel Web Design Guidelines, Anthropic frontend-design, Anthropic ux-designer.
- **Tier 3 — Pre-work research**: surreal-memory recall via `/kbd-memory-recall`, plus web search for "runtime dev tools on a web page" and "Chrome extension devtools panel best practices" (the latter two specifically required for the entity explorer changes).

For each entry: name, source URL, one-line capability summary, command examples where applicable.

### Fenced routing region in `CLAUDE.md` and `AGENTS.md`

A separate fenced region using marker pair `<!-- uiux-routing:start v1 -->` / `<!-- uiux-routing:end -->`. The region documents the discipline:

```
## UI/UX work routing

Before writing or modifying any UI/UX code in this repo, the AI agent
MUST follow these steps in order:

1. Consult surreal-memory for prior UI/UX decisions on this project
   via `/kbd-memory-recall`.
2. Run UI/UX Pro Max analysis on the target component(s) / page(s).
3. Run Impeccable commands relevant to the work: `/impeccable audit`,
   `/impeccable critique`, and the work-specific commands (e.g.
   `/impeccable polish` before shipping, `/impeccable distill` when
   simplifying, `/impeccable animate` when adding motion).
4. Consult Anthropic frontend-design + ux-designer skills.
5. Consult Vercel React Best Practices and Composition Patterns; for
   the entity-explorer panel and Chrome extension work specifically,
   also web-search "runtime devtools page best practices" and
   "Chrome MV3 devtools panel patterns".
6. Summarise best practices applicable to this specific task in one
   paragraph.
7. Only then write code.

Skill roster, command list, and source URLs are cached in
`.kbd-orchestrator/references/uiux-skill-roster.md` (refreshable).

This routing block is auto-managed; see /kbd-inject-agent-rules for
the fenced-region machinery.
```

### Re-use change 7's machinery, parameterised

Change 7's `kbd-inject-agent-rules` is hard-coded to inject the *agent rules* block. Two paths considered (design will pick one):

**Option A**: extend that skill to accept a `--pack <name>` flag where `<name>` ∈ `agent-rules` (default) | `uiux-routing` | future packs. Each pack has its own `references/template-<pack>.md` and `references/cache-<pack>.md` and its own marker pair.

**Option B**: ship a thin sibling skill `kbd-inject-uiux-routing` that mirrors the rules-injector structure but with the uiux content.

Design pick: **Option A** is preferred because the user signalled multiple injection use cases ("when activated" implies a parameterised skill, not a one-off). The injector becomes the generic fenced-region writer; the agent-rules pack is just its first customer.

### Orchestrator documentation

- Update `kbd-process-orchestrator/skills/kbd-inject-agent-rules/SKILL.md` to document the `--pack` flag and the two built-in packs.
- Add `/kbd-inject-agent-rules --pack uiux-routing` to the example block.

### Non-changes

- **No code changes to UI components in this repo.** Discipline only; code follows in change 10.
- **No installation of the cited skills.** The roster file documents them and points at source URLs; installing them is the operator's decision.
- **No automation that forces the discipline** at a build-time gate. The discipline is documented; agents are expected to follow it because the fenced region is read on session start.

## Capabilities

### New Capabilities

- `uar-uiux-skill-routing`: A documented, fenced-region routing discipline in this repo's `CLAUDE.md` / `AGENTS.md` listing the canonical UI/UX skill roster + the pre-work steps (memory consult, audit/critique runs, summarisation) any AI tool must follow before writing UI/UX code. Backed by a cached skill roster at `.kbd-orchestrator/references/uiux-skill-roster.md`.

### Modified Capabilities

- `kbd-agent-rules-injector`: Extended with a `--pack <name>` flag so the same machinery handles both the `agent-rules` pack (default — change 7) and the new `uiux-routing` pack (change 8). Marker pair becomes parameterised by pack name. Existing behavior preserved when `--pack` is omitted.

## Impact

- **Risk**: Low. Discipline-only; no behavioural code change to the running app.
- **Affected files**:
  - This repo: `.kbd-orchestrator/references/uiux-skill-roster.md` (new), `CLAUDE.md` + `AGENTS.md` (uiux-routing fenced region appended).
  - Skill-system: `skills/kbd-inject-agent-rules/SKILL.md` + `.sh` (add `--pack`), two new template files (`template-agent-rules.md` rename of existing, `template-uiux-routing.md` new), corresponding cache files.
- **Cross-repo**: Yes — skill-system gets the parameterised injector; UAR gets the rendered output + the roster cache.
- **Reversibility**: Trivial — delete the uiux-routing fenced region by hand; the `--pack` flag is additive on the injector side.
- **Unblocks**: change 10 (entity explorer UI work) MUST follow this routing discipline. The roster includes the explicit "runtime dev tools" research target the user named.

Source captures (date 2026-05-27):
- https://github.com/pbakaus/impeccable
- https://www.claudepluginhub.com/plugins/pbakaus-impeccable
- https://github.com/nextlevelbuilder/ui-ux-pro-max-skill
- https://ui-ux-pro-max-skill.com/
- https://snyk.io/articles/top-claude-skills-ui-ux-engineers/
- https://pasqualepillitteri.it/en/news/576/claude-code-skills-design-uiux-guide
- https://deepwiki.com/pbakaus/impeccable/2.3-skills-and-commands-catalog
