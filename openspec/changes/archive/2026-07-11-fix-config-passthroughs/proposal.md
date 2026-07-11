## Why

The `Cli` struct exposes `--port` / `PORT` and `--jwt-required` / `JWT_REQUIRED`
flags, but unlike the other 17 CLI overrides they were never applied to the
config builder — they parsed into `Cli` and were silently dropped. `PORT` was a
no-op (only `UAR_SERVER__PORT` worked), and `JWT_REQUIRED` was a
silently-ignored security flag, which is a real trap for a first-time deployer.

## What Changes

- Apply `cli.port -> server.port` and `cli.jwt_required -> security.jwt_required`
  in `AppConfig::load_with_cli`'s manual CLI-override block (matching the 17
  working passthroughs).
- Regression tests: `--port` overrides `server.port`; `--jwt-required=false`
  overrides `security.jwt_required` (default stays true).
- `.env.example`: document that the short `PORT`/`JWT_REQUIRED` forms are honored
  alongside the `UAR_*__*` forms.

## Capabilities

### New Capabilities
- `config-cli-surface`: every declared CLI/env config flag is actually applied
  to the runtime configuration (no silently-dropped passthroughs).

## Impact

`src/config.rs` (+2 override lines, +2 regression tests), `.env.example` doc
note. No behavior change for existing `UAR_*__*` users. KBD: change 6/9.
