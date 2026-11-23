# Open decisions from Spec — uar-uiux-architecture-2026-11

Collected by the lead from the ten spec writers' handoffs (2026-11-19). Each item names the
change that raised it and the writer's default. Operator decisions only; single-writer file
assignments and constants are Plan's job and are listed separately at the end.

## Operator decisions

| # | Decision | Raised by | Writer's default |
|---|---|---|---|
| 1 | Sub-agent model rule: an agent or sub-agent's explicit model wins over conversation and system defaults. Confirm? | system-model | yes (SCN-SYS-008 non-mandatory until confirmed) |
| 2 | Retiring `run_policy.global.model` conflicts with open change `embedded-run-policy-and-admin-surface` ("Global default model is applied"). Supersede that scenario? | system-model | supersede |
| 3 | Non-null Global `model` on write: reject or ignore; stored values: delete or leave inert | system-model | ignore on write, leave inert |
| 4 | Retire the YAML `llm_config.model` fallback in `resolve_default_model` (`runtime/manager.rs:11323-11331`), so the runtime cannot route a turn while the console says "no usable default"? | system-model | open |
| 5 | Team: a binding that explicitly pins a profile or settings revision is honoured, so a stale pin fails visibly (D5) | team | yes (SCN-TEAM-007 non-mandatory until confirmed) |
| 6 | Team: a wait continuation counts as a task boundary (D7) | team | yes (SCN-TEAM-006 non-mandatory until confirmed) |
| 7 | Team: disabling a route does not stop a running task; cancel is the stop (D10) | team | yes |
| 8 | Team: `credentialRef` changes count as part of a model change (D1) | team | yes |
| 9 | Stored rows of removed settings namespaces: leave inert, or one-time delete touching no other key | settings | leave inert |
| 10 | API writes to removed keys: accept and report `not_applied`, or reject | settings | accept, report `not_applied` |
| 11 | **Shell redesign gap:** no change owns replacing the shell with the `docs/design/` single sidebar plus tabbed inspector. The design change documents it in DESIGN.md and regroups Settings inside the current shell. Add a change for the shell, or defer it? | design | document now, regroup inside current shell |
| 12 | Snapshot owner for the PEM graph across tabs: the PGliteWorker leader, or the foreground tab | persistence | leader |
| 13 | Keep the optional model-catalog worker (build item 21)? Virtualizing the Models list is the real fix and nobody owns it today | workers | defer the worker |
| 14 | PGlite bump 0.5.12 → 0.5.17 | persistence | optional |
| 15 | Embedding-model option source: the catalog has no embedding flag. Server-declared option list (state/runtime owners), embedding fields read-only until then | pickers | server-declared list |
| 16 | Skill import: `POST /api/uar/skills/import-content` has no capability-catalog entry; `skills.import` is host-controlled. Trust-tools must confirm before the browser file chooser replaces the typed path | pickers | keep typed path as registered exception until confirmed |
| 17 | Picker open and catalog-load budget 200 ms | pickers | accept |
| 18 | Offered-set storage so operator-removed models do not return on reseed | config | open |
| 19 | Embedded realtime: no `entity.snapshot`/`entity.change` frame is emitted anywhere, yet the embedded adapter waits for `entity.snapshot` after each reset. Fix in this phase, or accept list invalidation only? | realtime | open |
| 20 | Desktop shell (The Boss) arm64 packaging gate: this host is x86_64. Include only with an arm64 host? | lead | exclude unless an arm64 host is provided |

## For Plan (not operator decisions)

- Single-writer assignments for shared files: `frontend/src/features/settings/ui/generic-schema-panel.tsx` (settings + pickers), `providers-store.ts` (identity before system-model), `frontend/src/lib/realtime/topics.ts` (identity + realtime), `server.rs`, `config*.rs`, `settings/**`, `llm/registry.rs`, `api/**`, `frontend/vite.config.ts`, `scripts/check-frontend-budgets.mjs`, `frontend/performance-budgets.json`, `scripts/build-pglite-seed.mjs`, `frontend/public/sw.js`, `scripts/check-frontend-boundaries.mjs` and its allowlist, `models-page.tsx`, new Playwright configs, team console tasks (team change 5.x).
- Ordering: identity before realtime `models` fan-out and before system-model; settings markers before pickers.
- Constants: snapshot retention, persistence max-wait, retry, compute budget.
- Fill `## Purpose` of `frontend-local-first-persistence` at sync.
- Known pre-existing defect: the seed builder expects `pglite-seed-v4.tar.gz`, which does not exist (`scripts/build-pglite-seed.mjs:122-129`).
- Pin note: `versions.toml` and `Cargo.toml` pin SurrealDB 4.0.0 (patched SDK); live-query behavior across replicas on 4.0.0 is unverified.
