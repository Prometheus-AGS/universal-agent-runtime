# frontend-build-tooling Specification

## Purpose
TBD - created by archiving change migrate-vite-rolldown-codesplitting. Update Purpose after archive.
## Requirements
### Requirement: Vendor Chunk Splitting Uses a Non-Deprecated Rolldown API

The frontend production build SHALL configure vendor chunk splitting via
Rolldown's `build.rolldownOptions.output.codeSplitting.groups` API rather
than the deprecated `manualChunks` function form, and MUST preserve the
same vendor chunk groupings (`vendor-react`, `vendor-assistant`,
`vendor-query`, `vendor-hljs`) that existed before the migration.

#### Scenario: Production build emits the expected vendor chunks

- **Given** `frontend/vite.config.ts` configures chunk splitting via
  `codeSplitting.groups`
- **When** `pnpm run build` runs
- **Then** the output MUST include separate chunks matching
  `vendor-react`, `vendor-assistant`, `vendor-query`, and `vendor-hljs`,
  with the same package-matching logic as the prior `manualChunks`
  function

#### Scenario: Build config uses no deprecated chunk-splitting API

- **Given** `frontend/vite.config.ts` is inspected for chunk-splitting
  configuration
- **When** checking for the deprecated `manualChunks` function form or
  the removed object form
- **Then** neither MUST be present — only `codeSplitting.groups` MUST
  configure vendor chunk grouping

