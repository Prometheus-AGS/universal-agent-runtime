# Specification Set: fix-broken-session-configuration-ui

## Backend and ZeeSpec gate

- Spec backend: OpenSpec (`.kbd-orchestrator/project.json`)
- ZeeSpec: n/a; no `.zeespec/` subject exists for this phase
- UAR source baseline: `b8c4fde214e250dc39080330ee4c130d102c78f7`
- Published dependency baseline: signed Entity Management `v3.0.2` at `f29a701649799df3ff64f5f986e3c016246d34b6`
- Upstream repair baseline: current Entity Management `origin/main` at `e25210010a8eb4e575f7e4fc6e04be598a8c8213`

## Track A — UAR critical path (serial)

1. `openspec/changes/adopt-entity-management-3-0-2`
   - Owns only `frontend/package.json`, root/frontend pnpm lockfiles, dependency evidence, and its OpenSpec/KBD artifacts.
   - Moves the product immediately from workspace `3.0.0-rc.1` to exact registry `3.0.2` for both React and core packages.
2. `openspec/changes/repair-session-configuration-entity-flow`
   - Owns the entity platform facade/contracts/transports/domain hooks, configured model selector path, chat session editor, typed session API, effective model resolution, focused tests, and its artifacts.
   - Must follow change 1 so implementation targets the released API.
3. `openspec/changes/prevent-session-configuration-regressions`
   - Owns project instructions outside managed regions, scoped frontend boundary checks/fixtures, the single installed-browser proof, evidence, and its artifacts.
   - Runs after change 2 is code-complete so the functional gate certifies the finished UAR slice.

Track A can close on exact 3.0.2 because the Session Configuration path ingests only the configured set (observed: 12 models), never the 7,248-row catalog.

## Track B — Entity Management upstream (parallel)

- Repository: `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`
- Clean worktree: `/Users/gqadonis/.claude/worktrees/entity-management-fix-atomic-fetched-list-ingestion`
- Branch: `codex/fix-atomic-fetched-list-ingestion`, based on updated `origin/main`
- Change: `openspec/changes/fix-atomic-fetched-list-ingestion`
- Owns the core graph action/engine, affected list bindings/adapters, focused tests, negative control, changeset/version evidence, upstream verification, commit, push, and PR.
- It MUST NOT edit or reset the dirty primary upstream checkout.

Track B is a required phase deliverable but is not a blocking dependency of Track A. UAR remains pinned to the reviewed exact 3.0.2 release for this phase even if the upstream patch is published before closeout. Adopting the later patch requires a separate explicit dependency change after its release artifacts are reviewed; this phase does not silently amend its 3.0.2 contract.

## Shared acceptance boundary

- The installed sheet opens within two seconds on the local certification host and remains interactive.
- Opening it issues no `/api/models` request and shows every configured provider/model, including catalog-unknown entries.
- Saved session configuration reloads and a genuine inference turn uses the saved model; cancel does not alter committed state.
- Retained controls have a typed runtime contract; unsupported decorative controls are removed.
- Compact and desktop computed styles prove body insets no smaller than the header inset.
- Entity Management 3.0.2 is preserved as the observed N+2 negative control; the upstream positive control publishes one success update for 7,248 rows.
- All product checks run locally after the owning code is complete. GitHub Actions remain deployment-only. No soak or broad unrelated suite is added.
- Each OpenSpec change passes `openspec validate <change> --strict` and has row-form verification evidence with source SHA, command, observed output, profile, and limit.

## Stop conditions

Stop and report instead of guessing if any of the following occurs:

1. Registry 3.0.2 cannot resolve as one core singleton from both UAR workspace roots.
2. The 3.0.2 tarball differs from the analyzed registry integrity metadata.
3. Fixing effective model precedence would break an existing explicit-request precedence contract.
4. A retained Session Configuration control has no discoverable runtime owner and removing it would conflict with an active product requirement.
5. Configured providers/models cannot be normalized without another full-catalog request.
6. Draft isolation cannot be implemented without exposing unsaved values through canonical shared patches.
7. The functional inference proof lacks a usable configured provider or records a different effective model than the saved session.
8. The upstream atomic action changes merge, lifecycle, sync, list, append, retry, cancellation, or error semantics beyond the specified replacement of incidental per-row clock samples with one batch timestamp.
9. Upstream fixed-group versioning would publish an unreviewed artifact or require moving `latest` during implementation.
10. The dirty upstream primary checkout would need to be modified, reset, or used for the fix.
11. Required work expands outside the permitted surfaces above.

## Uncomfortable constraint

The upstream 3.0.2 defect is real, but making its publication a prerequisite for the twelve-row Session Configuration path would delay the user-visible repair without reducing its risk. Conversely, treating the bounded UAR workaround as a general cure would leave every large-list consumer exposed. Both tracks are required, and neither may be used to erase the other's acceptance criteria.

## Analyze evidence inherited by Plan and Execute

- `.kbd-orchestrator/phases/fix-broken-session-configuration-ui/analysis.md` records the inspected UAR paths and the exact backend/frontend contract mismatch.
- `.prometheus/research/fix-broken-session-configuration-ui-entity-state-architecture.research/` records the registry tarball hashes, signed tag/current upstream SHAs, source excerpts, contradictions, and deep-research report.
- UAR source inspected during Analyze includes `frontend/src/platform/entities/index.ts`, `frontend/src/features/chat/session-config-panel.tsx`, `frontend/src/features/models/model/models-store.ts`, `frontend/src/features/models/model/use-model-selector.ts`, `frontend/src/stores/chat-session-config-store.ts`, `frontend/src/services/session-config-api.ts`, `frontend/src/features/chat/use-chat-runtime.ts`, `src/uar/api/discovery.rs`, `src/server.rs`, and `scripts/check-frontend-boundaries.mjs`.
- Upstream source inspected at both `v3.0.2` and `origin/main` includes `packages/entity-graph-core/src/graph.ts`, `packages/entity-graph-core/src/engine.ts`, `packages/entity-graph-react/src/hooks/use-entities.ts`, `packages/entity-graph-react/src/hooks/use-entity-query.ts`, `packages/entity-graph-react/src/view/use-entity-view.ts`, and `packages/entity-graph-react/src/adapters/electricsql-react.ts`.
- If any cited behavior differs when its owning change begins, that change stops and reconciles the spec instead of coding against the stale assumption.
