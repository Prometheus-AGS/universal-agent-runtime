## Why

5 of 7 GitHub Actions workflows fail on `main` (`CI`, `Tests (Quick)`,
`Comprehensive Test Suite`, `live-integration`, `template-cleanup`) while the
README displays status badges — a customer evaluating the repo sees red.
Under `uar-final-production-hardening-2026-07`'s 100%-customer-ready mandate,
every workflow must be green or explicitly advisory-and-labeled. The `ci.yml`
fix already exists as a reviewed, well-rationalized uncommitted working-tree
diff (feature scoping + dropping the `-D warnings` blanket escalation) that
was never committed.

## What Changes

- Adopt the working-tree `ci.yml` fix (scope to working features
  `postgres-backend,tauri,wasm-runtime`; let `Cargo.toml`'s `[lints]` govern
  clippy severity instead of a CLI `-D warnings` blanket).
- Align `quick-tests.yml`'s clippy step with the same policy.
- Fix `comprehensive-tests.yml`'s three documented failures: inline
  `cargo audit` without `security-audit.yml`'s ignore list; `bun install
  --frozen-lockfile` with no `bun.lockb` (repo moved to pnpm); Docker Compose
  test-service health-check timeouts.
- Delete the leftover `template-cleanup.yml` (repo-template artifact).
- Diagnose and fix `live-integration.yml`'s failing conclusion despite
  advisory `continue-on-error` steps.
- Real-dispatch verification of every touched workflow (repo standing rule).

## Capabilities

### New Capabilities
- `ci-pipeline-health`: every workflow on `main` concludes green or is
  explicitly advisory and labeled as such; README badges reflect reality.

### Modified Capabilities
(none)

## Impact

- `.github/workflows/*` only; no runtime code. KBD: change 2/9 of the phase.
