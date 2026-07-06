# fix-root-frontend-script-invocation

## Why

`package.json`'s `build`, `dev`, `test`, `test:e2e`, `lint`,
`typecheck` scripts all used `pnpm --filter ./frontend <cmd>`. This
requires the repo root to be a registered pnpm workspace containing
`frontend` as a member — but the already-landed
`frontend-pnpm-workspace-migration` change made `frontend/` its own
independent workspace root (`frontend/pnpm-workspace.yaml`: `packages:
[".", "packages/*"]`, so it can consume the
`@prometheus-ags/prometheus-entity-management` submodule via
`workspace:*`), and never registered the repo root as a workspace at
all. Result: every one of those 6 scripts failed immediately with
`No projects matched the filters in "<repo root>"` — none of them ever
reached their real tool (`tsc`, `vite`, `eslint`, `vitest`,
`playwright`). `format`'s `pnpm --filter ./frontend exec prettier
--write src/` failed the same way.

This went undetected because `cargo build`'s own frontend build
(`build.rs::build_frontend()`) uses a different, already-correct
invocation — `Command::new("pnpm").current_dir(&frontend_dir)` — so
every Rust-side verification checkpoint in recent phases exercised a
working code path, while the root `package.json` scripts CLAUDE.md
documents as the canonical dev commands (`bun run build`, `bun run
dev`, `bun run check`, `bun run lint`, `bun run format`) sat broken,
unexercised, since the migration landed.

## What changed

Replaced `pnpm --filter ./frontend <cmd>` with `pnpm -C frontend <cmd>`
in `build`, `dev`, `test`, `test:e2e`, `lint`, `typecheck` —
`-C`/`--dir` runs pnpm as if invoked from that directory directly,
without requiring any workspace registration. Matches `build.rs`'s
already-working pattern exactly.

`format` needed a different fix: `pnpm -C frontend exec prettier`
fails to find `prettier` (it's a *root* devDependency, never hoisted
into `frontend/node_modules`, and `-C` doesn't do workspace-aware
dependency resolution the way `--filter`'s `exec` did). Changed to
`pnpm exec prettier --write frontend/src/` — no `-C`/`--filter` needed
at all, since `pnpm exec` at the root naturally resolves the root's
own installed `prettier` binary; only the target path changed from
`src/` (relative to frontend) to `frontend/src/` (relative to root).

**Explicitly rejected**: adding a root-level `pnpm-workspace.yaml`
`packages: [frontend]` entry. Tested during this phase's assessment —
it does make `--filter ./frontend` match again, but breaks
`frontend/packages/prometheus-entity-management`'s own resolution as a
workspace member (`ERR_PNPM_WORKSPACE_PKG_NOT_FOUND`). Nesting a pnpm
workspace root inside another isn't supported the way this repo's
layout needs; this change is `package.json`-only, no
`pnpm-workspace.yaml` touched.

## Verification

- `bun run typecheck`: now reaches `tsc -b` and reports the real 17
  errors (was: `No projects matched the filters`).
- `bun run lint`: now reaches `eslint`, reports 215 real problems (140
  errors, 75 warnings) — pre-existing lint debt, unrelated to and out
  of scope for this change; confirms the command runs, not that lint
  is clean.
- `bun run build`: full Vite build succeeds (`✓ built in 15.29s`),
  `static/` assets produced.
- `format`'s new invocation verified via `pnpm exec prettier --check
  frontend/src/` (non-mutating) — resolves correctly, reports 174
  pre-existing formatting issues (separate, unrelated debt; this repo
  has never been run through `prettier --write` consistently). **Did
  not** run the real `--write` as part of verification after an
  initial mistake reformatted all 174 files repo-wide during this
  session — reverted via `git checkout -- frontend/` before staging
  anything, confirmed 0 unintended changes remain.
