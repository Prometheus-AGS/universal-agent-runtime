## Why

Release automation uses stale Node/Bun/all-features assumptions and packages platforms not routinely validated.

## What Changes

- Derive release validation from ordinary CI using Node 22, pnpm and supported Cargo bundles.
- Validate current asset/config paths.
- Test build/install/startup/health/archive on advertised platforms.

## Capabilities
### New Capabilities
- `release-workflow-platform-certification`

## Impact
GitHub Actions, packaging scripts, release artifacts and platform support docs.
