## Why

`prometheus-entity-management` ships its own pnpm-workspaces tooling (and includes `examples/` as pnpm sub-workspaces). To embed it as a git submodule under `frontend/packages/prometheus-entity-management/` and consume it via `workspace:*`, the host `frontend/` directory must itself be a pnpm workspace root. UAR's current frontend is a single bun-managed package; converting to pnpm workspaces unblocks the submodule consumption and aligns with the entity-mgmt repo's conventions.

## What Changes

- New `frontend/pnpm-workspace.yaml`:
  ```yaml
  packages:
    - "."
    - "packages/*"
  ```
- Convert `frontend/package.json` `packageManager` field to `pnpm@10.x`; remove bun lockfile (`bun.lockb`).
- Run `pnpm import` from existing lockfile if possible, otherwise `pnpm install` cold.
- Update `build.rs` to invoke pnpm:
  ```rust
  // before: bun run build
  // after:
  let status = Command::new("pnpm")
      .args(["--filter", "./frontend", "install", "--frozen-lockfile"])
      .status()?;
  // then:
  let status = Command::new("pnpm")
      .args(["--filter", "./frontend", "build"])
      .status()?;
  ```
- Keep `bun` available in the container (per the Dockerfile change) for ad-hoc scripts and faster cold installs in dev — pnpm is the canonical project tool.
- Update `package.json` scripts: replace any `bun` invocations with `pnpm`/`pnpm exec`.
- CI workflows updated: `pnpm/action-setup` + `actions/setup-node@v4`.

## Acceptance

- `pnpm install` in repo root succeeds, populating `frontend/node_modules` and (after the next change) `frontend/packages/prometheus-entity-management/node_modules`.
- `cargo build` triggers `pnpm` via `build.rs` and produces `frontend/dist/` (or `static/`) successfully.
- CI green.
