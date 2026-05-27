# Implementation Tasks — ssed-uar-uiux-skill-routing

## 1. Roster cache (this repo)

- [x] 1.1 Create `.kbd-orchestrator/references/uiux-skill-roster.md` with Tier 1/2/3 organisation, source URLs + anchor keywords + fetch date

## 2. Skill-system: parameterised injector

- [x] 2.1 Rename / dual-name `references/template.md` → `references/template-agent-rules.md` (keep both names readable via the script's fallback chain)
- [x] 2.2 Rename / dual-name `references/rules-cache.md` → `references/cache-agent-rules.md` (same fallback)
- [x] 2.3 Add `references/template-uiux-routing.md` (fenced region body)
- [x] 2.4 Add `references/cache-uiux-routing.md` (skill-system-shipped roster — used as fallback when no project-local roster exists)
- [x] 2.5 Update `kbd-inject-agent-rules.sh`:
  - [x] 2.5.1 Parse `--pack <name>` (default `agent-rules`)
  - [x] 2.5.2 Validate value ∈ {agent-rules, uiux-routing}
  - [x] 2.5.3 Resolve template + cache paths per pack with back-compat fallback to old names
  - [x] 2.5.4 Derive `START_MARK` / `END_MARK` from pack name
  - [x] 2.5.5 For `uiux-routing` pack, prefer project-local roster at `<path>/.kbd-orchestrator/references/uiux-skill-roster.md` when present
- [x] 2.6 Update `SKILL.md` with `--pack` flag, examples, and the two built-in packs

## 3. Render uiux-routing into this repo

- [x] 3.1 Run `kbd-inject-agent-rules --pack uiux-routing` against this UAR worktree's CLAUDE.md + AGENTS.md
- [x] 3.2 Verify the existing agent-rules region is byte-preserved (spec scenario: "Routing block separate from agent-rules block")
- [x] 3.3 Verify second run is bit-identical (idempotency)

## 4. Smoke tests

- [x] 4.1 Update `test-agent-rules-injector.sh` to add cases:
  - [x] 4.1.1 `--pack agent-rules` (explicit) == default
  - [x] 4.1.2 `--pack uiux-routing` writes uiux-routing markers
  - [x] 4.1.3 Both packs co-exist in same file without interference
  - [x] 4.1.4 Invalid pack value rejected

## 5. Closeout

- [ ] 5.1 `/opsx:verify` + `/opsx:archive`
- [ ] 5.2 progress.json `changes_completed: 8`; active_change → `ssed-entity-surreal-live-adapter`
