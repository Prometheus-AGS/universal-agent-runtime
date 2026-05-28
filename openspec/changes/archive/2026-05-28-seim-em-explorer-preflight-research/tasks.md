## 1. Preparation

- [x] 1.1 `cd ~/.claude/worktrees/seim-entity-management` — verify branch is `feat/seim-entity-management-impl` and working tree is clean (`git status`)
- [x] 1.2 `mkdir -p docs` — create the docs directory if it does not exist
- [x] 1.3 Read `prior-context.md` (populated by the `assess:before` / `/kbd-memory-recall` hook) — note any prior UI/UX decisions relevant to a devtools panel surface; these become §1 content

## 2. UI/UX Pro Max analysis (§2)

- [x] 2.1 Invoke the `nextlevelbuilder/ui-ux-pro-max-skill` via the Skill tool (or Agent tool if not installed) with the prompt: "Recommend a palette, font pairing, spacing scale, and a11y guidelines for a **dark** devtools panel that displays entity graph data (entity lists, patch diffs, live events). Favour monospace / code-adjacent fonts. Target React + Tailwind."
- [x] 2.2 Capture: palette name + hex swatches, font pair, spacing scale, at least one a11y observation (contrast ratio, focus state, or touch target). Attribute to `ui-ux-pro-max-skill`.

## 3. Impeccable commands (§3)

- [x] 3.1 Run `/impeccable audit` on the Entity Explorer concept (describe a 5-tab FAB panel: Entities / Patches / Events / Subscriptions / Performance) — capture accessibility, performance, and responsive findings
- [x] 3.2 Run `/impeccable critique` on the same concept — capture UX-hierarchy, clarity, and emotional-resonance findings
- [x] 3.3 Run `/impeccable distill` — capture complexity-reduction recommendations (devtools panels are especially prone to feature creep)

## 4. Anthropic skill consultation (§4)

- [x] 4.1 Invoke the `frontend-design` Anthropic skill with the Entity Explorer panel brief — capture intentional-design recommendations; attribute to `frontend-design`
- [x] 4.2 Invoke the `ux-designer` Anthropic skill with the same brief — capture UX-engineer review findings; attribute to `ux-designer`

## 5. Vercel React skills (§5)

- [x] 5.1 Consult **React Best Practices** (Vercel skill) — capture: client/server component boundary guidance relevant to a devtools panel, performance defaults (lazy loading tabs, virtualized entity list)
- [x] 5.2 Consult **Composition Patterns** (Vercel skill) — capture: tab-panel composition pattern (state lifting vs. per-tab local state), whether any tab needs a React context boundary, deferred rendering strategy for inactive tabs

## 6. Web search — runtime devtools best practices (§6)

- [x] 6.1 Search: `"runtime devtools page best practices 2026"` — record top 2-3 results in citation format: `- <URL> | anchor: <keywords> | fetched: YYYY-MM-DD`
- [x] 6.2 Scan each result for: panel layout patterns, tab organisation, data refresh strategies, performance profiling UI conventions

## 7. Web search — Chrome MV3 devtools panel patterns (§7)

- [x] 7.1 Search: `"Chrome MV3 devtools panel patterns"` — record top 2-3 results in citation format
- [x] 7.2 Search: `"react-devtools bridge architecture"` — record top 1-2 results
- [x] 7.3 Scan results for: devtools page lifecycle, background service worker bridge, panel injection, message-passing between content-script and panel

## 8. Write docs/devtools-design-notes.md

- [x] 8.1 Create `~/.claude/worktrees/seim-entity-management/docs/devtools-design-notes.md` with eleven sections:
  - `## §1 Memory recall digest` — findings from task 1.3 (or "No prior UI/UX decisions found")
  - `## §2 UI/UX Pro Max summary` — palette + font + spacing + a11y from task 2.2
  - `## §3 Impeccable audit · critique · distill` — findings from tasks 3.1–3.3
  - `## §4 Anthropic frontend-design + ux-designer` — findings from tasks 4.1–4.2
  - `## §5 Vercel React Best Practices + Composition Patterns` — findings from tasks 5.1–5.2
  - `## §6 Web search: runtime devtools best practices` — citations from task 6.1 + key findings from 6.2
  - `## §7 Web search: Chrome MV3 devtools panel patterns` — citations from tasks 7.1–7.2 + key findings from 7.3
  - `## §8 Distillation` — **one paragraph, 3-5 sentences**, written as forward-looking prompt context for W6 ("When implementing the panel, prefer…")
  - `## §9 Implications for W5 — event-bus / multi-store registry` — 1-2 concrete bullets
  - `## §10 Implications for W6 — 5-tab panel components` — 1-2 concrete bullets
  - `## §11 Implications for W8 — Chrome extension scaffold` — 1-2 concrete bullets (bonus)
- [x] 8.2 Verify §8 is exactly one paragraph (3-5 sentences), not a bullet list
- [x] 8.3 Verify §6 and §7 each have at least one citation entry in `URL | anchor: … | fetched: …` format

## 9. Commit

- [x] 9.1 `cd ~/.claude/worktrees/seim-entity-management && git add docs/devtools-design-notes.md` — stage the file explicitly (do NOT use `git add .`)
- [x] 9.2 `git diff --cached --name-only` — confirm only `docs/devtools-design-notes.md` is staged
- [x] 9.3 Commit with message:
  ```
  docs(explorer): preflight research notes — UI/UX routing discipline W4
  ```
- [x] 9.4 `git diff --name-only HEAD~1 HEAD` — verify exactly one file path in the diff

## 10. Progress update

- [x] 10.1 Update `.kbd-orchestrator/phases/submodule-entity-management-implementation/progress.json`:
  - Move `"seim-em-explorer-preflight-research"` from `active_change` to `completed_changes[]`
  - Set `changes_completed` to 6
  - Set `active_change` to `"seim-em-explorer-event-bus-registry"` (W5)
  - Set `active_change_state` to `"pending"`
  - Add the new commit sha to `entity_mgmt_worktree_local_commits[]`
  - Update `updatedAt` to current ISO-8601 timestamp
