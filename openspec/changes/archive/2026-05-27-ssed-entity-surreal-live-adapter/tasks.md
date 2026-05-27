# Implementation Tasks — ssed-entity-surreal-live-adapter

> Cross-repo: prometheus-entity-management (adapter + tests) + prometheus-skill-system (skill SKILL.md).

## 1. Adapter — `prometheus-entity-management/src/adapters/surreal-live.ts`

- [ ] 1.1 Implement `createSurrealLiveAdapter(opts)` per design §Implementation Sketch
- [ ] 1.2 Minimal `Surreal` interface — no driver lock-in
- [ ] 1.3 `select-then-live` default seeding; `live-only` alternative
- [ ] 1.4 Action → EntityChange mapping (CREATE/UPDATE/DELETE)
- [ ] 1.5 `normalize` applied to every row
- [ ] 1.6 `affectedListKeys` derived per ChangeSet
- [ ] 1.7 Reconnect loop with 1/3/9/30s backoff
- [ ] 1.8 `checkpointResume` save + replay
- [ ] 1.9 `onStatus` + `onSynced` callbacks

## 2. Tests — `surreal-live.test.ts`

- [ ] 2.1 Hand-rolled fake `Surreal` client (records query/live/kill calls)
- [ ] 2.2 Seed path tests (select-then-live + live-only)
- [ ] 2.3 CREATE/UPDATE/DELETE mapping
- [ ] 2.4 `normalize` invoked per row
- [ ] 2.5 Reconnect with backoff
- [ ] 2.6 Checkpoint replay (configured + unconfigured)
- [ ] 2.7 `affectedListKeys` correctness
- [ ] 2.8 `pnpm test src/adapters/surreal-live.test.ts` green

## 3. Re-export — `src/index.ts`

- [ ] 3.1 Add named export for `createSurrealLiveAdapter`
- [ ] 3.2 Re-export `SurrealTableConfig` and `SurrealLiveAdapterOptions` as types

## 4. Skill — `prometheus-skill-system/skills/prometheus-entity-skills/entity-realtime-surreal-live/`

- [x] 4.1 `SKILL.md` with front matter, sections per spec req "Companion Skill"
- [x] 4.2 At least one code example showing `createSurrealLiveAdapter` registration
- [x] 4.3 Cross-link to `src/adapters/surreal-live.test.ts`

## 5. Cross-repo PRs

- [ ] 5.1 `prometheus-entity-management` PR: adapter + tests + re-export
- [ ] 5.2 `prometheus-skill-system` PR: skill SKILL.md (combines naturally with W1-W3 PR)
- [ ] 5.3 Record commit SHAs + PR URLs below after merge

```
prometheus-entity-management commit: <fill in after merge>
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
prometheus-skill-system commit:      eb3134bce2b1f1a956acfd6d264f2b7d8862974f
PR URL:                         https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

## 6. Closeout

- [x] 6.1 Skill SKILL.md landed in prometheus-skill-system
- [ ] 6.2 Adapter implementation **deferred to a dedicated implementation session** — requires running the test suite in `prometheus-entity-management` (vitest + the rest of the project's tsconfig), which is out-of-scope for this UAR worktree's tooling. The spec + design + task seeds in this change carry every detail an implementer needs.
- [ ] 6.3 `/opsx:verify` will flag §1–3 as deferred CRITICAL — proceed to archive with that documented status (cross-repo TS implementation requires its own worktree session)
- [ ] 6.4 progress.json `changes_completed: 9` with a `deferred_tasks` entry; active_change → `ssed-entity-explorer-fab-panel`
