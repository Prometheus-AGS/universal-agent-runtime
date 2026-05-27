# Implementation Tasks — ssed-kbd-memory-first-execution

> Target repo: `prometheus-skill-system`. Folds into the same topic-branch commit.

## 1. Detection helper — `shared/lib/memory.sh`

- [x] 1.1 Create with `kbd_memory_available` + `kbd_memory_url` + cached probe (per design D4)
- [x] 1.2 Probe order: `KBD_AVAILABLE_TOOLS` substring match → `$UAR_MEMORY_MCP_URL` curl → `$KBD_MEMORY_MCP_URL` curl → `.kbd-orchestrator/memory.config.json` field → return 1
- [x] 1.3 2s curl timeout; soft-fail on any error

## 2. Memory log hook — `shared/lib/memory-log.sh`

- [x] 2.1 Wrapper script invoked by the `kbd-memory-log` hook entry
- [x] 2.2 Sources `memory.sh`; exit 0 when unavailable (no-op)
- [x] 2.3 Builds the canonical `kbd_lifecycle_event` payload via `jq -n`
- [x] 2.4 POST to `<url>/api/entities`; ignore any response (best-effort)
- [x] 2.5 Captures ONLY structured KBD_HOOK_* values (spec req 2 — no stderr leakage)

## 3. /kbd-memory-recall skill

- [x] 3.1 `skills/kbd-memory-recall/SKILL.md` (front matter, when-to-use, progress signals, prerequisites, how-to-invoke, examples)
- [x] 3.2 `skills/kbd-memory-recall/kbd-memory-recall.sh`:
  - [x] 3.2.1 Resolves phase from arg or waypoint
  - [x] 3.2.2 Loads `goals.md` + `assessment.md` (if present) as the query text
  - [x] 3.2.3 POSTs to `<url>/api/find_relevant` with entityType filter
  - [x] 3.2.4 Writes markdown digest to `phases/<phase>/prior-context.md`
  - [x] 3.2.5 Stub digest when memory unavailable (`<!-- memory endpoint unreachable … -->`)
  - [x] 3.2.6 Always exits 0 (composes with `on_failure: ignore`)

## 4. hooks.json — two new built-in entries

- [x] 4.1 `kbd-memory-log` augment hook covering `*:*`
- [x] 4.2 `auto-memory-recall` augment hook covering `assess:before`
- [x] 4.3 `jq .` validates

## 5. Documentation

- [x] 5.1 `shared/references/memory-retention.md`: entity schema, retention window, relevance ordering
- [x] 5.2 Orchestrator `SKILL.md` "Surreal-Memory Integration" rewritten — default-on when reachable

## 6. Smoke tests

- [x] 6.1 `shared/lib/tests/test-memory.sh`:
  - [x] 6.1.1 `kbd_memory_available` returns 1 in empty env, returns 0 with `KBD_AVAILABLE_TOOLS="create_entity"`
  - [x] 6.1.2 `kbd-memory-recall.sh` writes stub digest when no endpoint
  - [x] 6.1.3 `memory-log.sh` no-ops cleanly when no endpoint

## 7. Cross-repo commit + closeout

- [ ] 7.1 Stage in topic branch (combines with prior changes)
- [ ] 7.2 `/opsx:verify` + `/opsx:archive`
- [ ] 7.3 progress.json `changes_completed: 6`; active_change → `ssed-kbd-agent-rules-injector`

```
prometheus-skill-system commit: eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```
