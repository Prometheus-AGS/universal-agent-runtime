## 1. Audit upstream packages
- [x] 1.1 Confirm `@a2ui/web_core` exists on npm, and record version/license/maintainer. Result: v0.10.4, Apache-2.0, `a2ui-team <a2ui-owners@google.com>`, published 2026-07-08.
- [x] 1.2 Confirm `@a2ui/react` exists on npm, and record version/license/maintainer. Result: v0.10.1, Apache-2.0, same org, published ~2026-06-23.
- [x] 1.3 Cross-check both against the upstream GitHub org/repo (`a2ui-project/a2ui`, directories `renderers/web_core` and `renderers/react`) via each package's `homepage`/`repository` fields. Confirmed consistent; no naming correction needed to the plan.
- [x] 1.4 Inspect tarball contents (`npm pack`) to determine whether npm ships original TypeScript source or compiled output. Result: compiled JS + `.d.ts` only (no `.ts` source) — informs the vendoring-approach decision in `proposal.md`.

## 2. `frontend/packages/a2ui-core/`
- [x] 2.1 `package.json` — exact-pinned `"@a2ui/web_core": "0.10.4"` dependency; `exports` map for `.`, `./v0_8`, `./v0_9`, `./v0_9/basic_catalog`.
- [x] 2.2 `src/index.ts`, `src/v0_8.ts`, `src/v0_9.ts`, `src/v0_9-basic-catalog.ts` — thin re-export modules, each carrying the upstream Apache-2.0 header.
- [x] 2.3 `LICENSE` — Apache-2.0 text copied verbatim from the upstream package.
- [x] 2.4 `UPSTREAM.md` — pinned version table, pinning-vs-copying rationale, update procedure.
- [x] 2.5 `README.md` — usage + scope (consumed by Changes 17 and 22).
- [x] 2.6 `tsconfig.json` — standalone strict TS config; `typecheck` script (`tsc --noEmit`).

## 3. `frontend/packages/a2ui-react/`
- [x] 3.1 `package.json` — exact-pinned `"@a2ui/react": "0.10.1"` dependency; `react`/`react-dom` peer deps matching upstream's `^19.2.7` requirement; `"private": true`; description explicitly states reference-implementation-only.
- [x] 3.2 `src/index.ts`, `src/styles.ts` — thin re-export modules with a REFERENCE-IMPLEMENTATION-ONLY banner comment and the upstream Apache-2.0 header.
- [x] 3.3 `LICENSE` — Apache-2.0 text copied verbatim.
- [x] 3.4 `README.md` — prominent "do not import from product code" banner, pinned-version table, and the Change 17 cross-testing rationale for keeping this package at all.
- [x] 3.5 `tsconfig.json` — standalone strict TS config (JSX enabled); `typecheck` script.

## 4. Workspace integration
- [x] 4.1 Confirm `frontend/pnpm-workspace.yaml`'s existing `packages/*` glob picks up both new packages without config changes.
- [x] 4.2 `pnpm -C frontend install` — regenerate `pnpm-lock.yaml` (not `--frozen-lockfile`, since this change adds new workspace packages/dependencies). Pre-existing blocker found and resolved: the `prometheus-entity-management` git submodule was uninitialized in this worktree, which fails `pnpm install` for the whole workspace regardless of this change; initialized it (`git submodule update --init`) and built it (`pnpm build` in that package) so the full-workspace `typecheck`/`lint` could run clean. Neither action touches this change's own files.

## 5. Verification
- [x] 5.1 `pnpm --filter @prometheus-ags/a2ui-core typecheck` — pass.
- [x] 5.2 `pnpm --filter @prometheus-ags/a2ui-react typecheck` — pass.
- [x] 5.3 `pnpm -C frontend typecheck` (full workspace, `tsc -b`) — pass.
- [x] 5.4 `pnpm -C frontend lint` (full workspace, `eslint .`) — pass (root config ignores `packages/**` by existing convention).
- [x] 5.5 `openspec validate a2ui-vendor-google-core-react --strict` — pass.

## 6. Operator follow-up
- [ ] 6.1 Operator/coordinating session reviews the pinned-dependency-vs-source-copy decision in `proposal.md` before merge. **Not auto-approved** — this is a judgment call disclosed for review, not unilaterally finalized.
- [ ] 6.2 Coordinating session decides on merge (per task instructions, this session does not merge its own branch).
