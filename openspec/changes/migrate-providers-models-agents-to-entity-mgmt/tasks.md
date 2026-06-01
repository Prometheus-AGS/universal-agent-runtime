## Status — 2026-05-26

**Deferred to next session.** This change is the broad rollout of the entity-mgmt migration across Admin surfaces (providers, LLM models, agents, skills, settings) plus every consumer that reads those entities outside the matching Admin page (chat header model/agent badge, agent selector, capability toggles, session config panel).

All architectural prerequisites have shipped in this session — the change is mechanical at this point, but invasive enough that it should be broken into one PR per entity, each verified in-browser before moving to the next.

## 1. Provider entity

- [ ] 1.1 `services/entities/providers.ts`.
- [ ] 1.2 Migrate `admin/pages/providers-page.tsx`.

## 2. Model entity

- [ ] 2.1 `services/entities/models.ts`.
- [ ] 2.2 Migrate models page + capability toggles model picker + chat header.

## 3. Agent entity

- [ ] 3.1 `services/entities/agents.ts`.
- [ ] 3.2 Migrate agents page + `AgentSelector` + chat header label.

## 4. Skill entity

- [ ] 4.1 `services/entities/skills.ts`.
- [ ] 4.2 Migrate skills page; respect `kind` + `origin` (change 3 + 12 already ship the domain support).

## 5. Settings entity

- [ ] 5.1 `services/entities/settings.ts`.
- [ ] 5.2 Migrate `SessionConfigPanel`.

## 6. Audit

- [ ] 6.1 `docs/migration-stale-data-audit.md` enumerating render paths.
- [ ] 6.2 `git grep "fetch\\(" frontend/src` shows no direct fetches for migrated entities outside `services/entities/`.

## 7. Tests

- [ ] 7.1 Cross-component propagation snapshot.
