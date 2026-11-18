# Implementation inventory

Recorded 2026-11-18 at the implementation boundary for `console-agent-teams`. Tasks 1 to 4 are implemented. Nothing listed here has been compiled, type-checked, linted, tested or rendered. Verification (tasks 5.x) had not started when this was written.

Writers: domain (`platform/entities/teams`), console A (`features/teams` except `ui/run`), console B (`features/teams/ui/run`). The lead added the error `diagnostic` field (`api/teams-errors.ts`, `teams/index.ts`, `ui/runtime-notice.tsx`) after writer B reported it missing.

Handoffs: `handoff-domain.md`, `handoff-console-a.md`, `handoff-console-b.md`. Recorded service payloads: `observations/`.

## New files (71)

| File | sha256 (first 16) |
|---|---|
| `frontend/src/features/teams/index.ts` | `bf06ef14f5803d03` |
| `frontend/src/features/teams/model/binding-diagnostics.ts` | `fd78bc54c1b479aa` |
| `frontend/src/features/teams/model/binding-draft.test.ts` | `e2fcb5017c4704b1` |
| `frontend/src/features/teams/model/binding-draft.ts` | `ee65df260c396813` |
| `frontend/src/features/teams/model/errors.ts` | `edbe155bfecc1767` |
| `frontend/src/features/teams/model/format.ts` | `ead8439f95337512` |
| `frontend/src/features/teams/model/launch-input.test.ts` | `0a4c64956e6287c2` |
| `frontend/src/features/teams/model/launch-input.ts` | `af97d74593ebc7a3` |
| `frontend/src/features/teams/model/package-analysis.test.ts` | `747f082674c52e63` |
| `frontend/src/features/teams/model/package-analysis.ts` | `9cfc0e912fe50916` |
| `frontend/src/features/teams/model/readiness.test.ts` | `1a566b39126221e4` |
| `frontend/src/features/teams/model/readiness.ts` | `85a1983e09698f55` |
| `frontend/src/features/teams/model/teams-location.ts` | `c58a4e9bce573a34` |
| `frontend/src/features/teams/model/use-definition.ts` | `a58d8fd1585db67a` |
| `frontend/src/features/teams/model/use-readiness.ts` | `8a14877ba185271d` |
| `frontend/src/features/teams/model/use-sheet-helpers.ts` | `b0ed8dc0478e5bb0` |
| `frontend/src/features/teams/model/use-teams-bootstrap.ts` | `13381cf1c0e9b053` |
| `frontend/src/features/teams/ui/binding-model-step.tsx` | `50307d78d8f30894` |
| `frontend/src/features/teams/ui/binding-reference-fields.tsx` | `768304fb765c7292` |
| `frontend/src/features/teams/ui/binding-sheet.tsx` | `e456f1f242e54839` |
| `frontend/src/features/teams/ui/definition-detail.tsx` | `525e2695c13e7a31` |
| `frontend/src/features/teams/ui/gated-action.tsx` | `77a94c07d4a09d17` |
| `frontend/src/features/teams/ui/install-package-sheet.tsx` | `eeb2bb88f5f9ba28` |
| `frontend/src/features/teams/ui/launch-sheet.tsx` | `2969fa8bec8afb77` |
| `frontend/src/features/teams/ui/readiness-strip.tsx` | `57628c4c38a822a5` |
| `frontend/src/features/teams/ui/run/attempt-card.tsx` | `72766d9d64c81d36` |
| `frontend/src/features/teams/ui/run/attempt-parts.tsx` | `1ebd6f73a12a2d1f` |
| `frontend/src/features/teams/ui/run/cancel-attempt.tsx` | `55bef804a7897edc` |
| `frontend/src/features/teams/ui/run/collect.tsx` | `dadcc9c569ac082a` |
| `frontend/src/features/teams/ui/run/disclosure.tsx` | `a71fab8cdd078528` |
| `frontend/src/features/teams/ui/run/member-lane.tsx` | `7e41bb723cf572a3` |
| `frontend/src/features/teams/ui/run/member-lanes.tsx` | `ee1d0150eb73ce46` |
| `frontend/src/features/teams/ui/run/member-records.tsx` | `4d9e0bd933590297` |
| `frontend/src/features/teams/ui/run/output-view.tsx` | `9148b2d31e05ad25` |
| `frontend/src/features/teams/ui/run/refresh-status.tsx` | `8895f4c24caeb8d9` |
| `frontend/src/features/teams/ui/run/routing-progress.tsx` | `9c63f25cd89a0916` |
| `frontend/src/features/teams/ui/run/routing-reservation.tsx` | `35a71b89f1de945e` |
| `frontend/src/features/teams/ui/run/routing-test.tsx` | `345235e5230ce19f` |
| `frontend/src/features/teams/ui/run/run-hooks.ts` | `da7080d89d97f54d` |
| `frontend/src/features/teams/ui/run/run-model.test.ts` | `81c1aaa5c0b07d47` |
| `frontend/src/features/teams/ui/run/run-model.ts` | `e8522e19fe34b8cd` |
| `frontend/src/features/teams/ui/run/status-label.tsx` | `b7eafef87b21d70b` |
| `frontend/src/features/teams/ui/run/task-parts.tsx` | `a9e9e1a92e6050e6` |
| `frontend/src/features/teams/ui/run/team-run-view.tsx` | `6f6d9d74f6a4ca13` |
| `frontend/src/features/teams/ui/run/wait-detail.tsx` | `ee30d6b614f5669c` |
| `frontend/src/features/teams/ui/runtime-notice.tsx` | `2afc3624f7c2b694` |
| `frontend/src/features/teams/ui/team-detail.tsx` | `dddbf429ab3da4a2` |
| `frontend/src/features/teams/ui/teams-list.tsx` | `4aca7cbd63254338` |
| `frontend/src/features/teams/ui/teams-page.tsx` | `969d052695e8db66` |
| `frontend/src/features/teams/ui/teams-welcome.tsx` | `47a5eaad9664c275` |
| `frontend/src/features/teams/ui/workspace-control.tsx` | `88954cfed8400def` |
| `frontend/src/platform/entities/teams/api/teams-errors.ts` | `66ea259eada1f242` |
| `frontend/src/platform/entities/teams/api/teams-transport.ts` | `ea0e7e7a2e1a687c` |
| `frontend/src/platform/entities/teams/binding-builder.ts` | `2cbcc71207284c90` |
| `frontend/src/platform/entities/teams/commands.ts` | `db0bb92f876725c2` |
| `frontend/src/platform/entities/teams/contracts.ts` | `40b643314dba567b` |
| `frontend/src/platform/entities/teams/digest.test.ts` | `a9a8f1687254331c` |
| `frontend/src/platform/entities/teams/digest.ts` | `c47856821b800603` |
| `frontend/src/platform/entities/teams/domain.ts` | `e8c32a4d1c7775b3` |
| `frontend/src/platform/entities/teams/entities.ts` | `1c8094a2e3bd42e2` |
| `frontend/src/platform/entities/teams/export-model.ts` | `a2363e343b6c5d8b` |
| `frontend/src/platform/entities/teams/index.ts` | `39a437a077230511` |
| `frontend/src/platform/entities/teams/ingest.ts` | `a5039e826694f957` |
| `frontend/src/platform/entities/teams/keys.ts` | `34b4ca3a94bf6201` |
| `frontend/src/platform/entities/teams/loads.ts` | `6d9c451e9318c79e` |
| `frontend/src/platform/entities/teams/registration.ts` | `9b0c44ba4bf8bec0` |
| `frontend/src/platform/entities/teams/selectors.ts` | `d88fee0a55e3672c` |
| `frontend/src/platform/entities/teams/store.ts` | `de2b48bd3ab56925` |
| `frontend/src/platform/entities/teams/use-team-run.ts` | `a58b39697cf2a164` |
| `frontend/src/platform/entities/teams/use-teams.ts` | `b96c09bdb553bbfa` |
| `frontend/src/platform/entities/teams/watch.ts` | `743cf46c34df5d03` |

## Edited files (5)

| File | sha256 (first 16) |
|---|---|
| `frontend/src/app/shell/nav-destinations.ts` | `e297d26e8ab296b8` |
| `frontend/src/app/shell/nav-destinations.test.ts` | `aa0f9464229aff27` |
| `frontend/src/entities/bootstrap.ts` | `11ba99b09c05be2d` |
| `frontend/src/pages/admin-page.tsx` | `37f03e0bd508b68d` |
| `frontend/src/platform/entities/index.ts` | `b1eba97939374907` |

## Known open items carried into verification

- The run view's waiting, handed-off, delegated, cancelling and uncertain states have no recorded payload; only queued-to-running, succeeded and failed were observed.
- `collect.tsx` folds cross-record data by calling one narrow hook per id; it is the least conventional piece.
- The watcher does not restart by itself after stopping; the routing console triggers a refresh after each submit and retry.
- One credential reference is applied to every model binding.
- Launch is not gated on execution ownership.
- Authored, never run: `digest.test.ts`, `readiness.test.ts`, `package-analysis.test.ts`, `binding-draft.test.ts`, `launch-input.test.ts`, `run-model.test.ts`, and the edit to `nav-destinations.test.ts`.

## Verification authorization

Granted by the operator on 2026-11-18, after this inventory was recorded, for the `console-agent-teams` change only.

Covered: two isolated design critiques and one adversarial review of the diff; `pnpm -C frontend typecheck`; `pnpm -C frontend lint`; the teams test files and `nav-destinations.test.ts` under vitest; one frontend bundle build with the fixed `entity-graph-core` dist swapped in and restored afterwards; a browser pass against the dev instance through `http://127.0.0.1:8090` covering install, bind, launch, routing test and member lanes; repairs and reruns of whatever fails.

Excluded: commit, push, pull request, any change to the installed service on port 1906, any Cargo command, and any other change or phase.
