# pglite-oxide-portability Specification

## Purpose
TBD - created by archiving change pglite-oxide-intel-mac-spike. Update Purpose after archive.
## Requirements
### Requirement: Evidence-based portability verdict for pglite-oxide on x86_64-apple-darwin
The system SHALL produce a recorded PASS/FAIL verdict, backed by a live
execution on `x86_64-apple-darwin`, for whether `pglite-oxide`'s
`portable-wasix` asset can boot a working `PgliteServer` on that platform,
before `desktop-data-layer-pglite-oxide` treats pglite-oxide as an
available enhancement on Intel Mac.

#### Scenario: portable-wasix boots and serves a query successfully
- **WHEN** the spike starts `PgliteServer` via the `portable-wasix` asset
  on `x86_64-apple-darwin` and issues a `CREATE TABLE` / `INSERT` /
  `SELECT` round-trip over the PG-wire connection
- **THEN** the server boots without error, the `SELECT` returns the
  inserted row, and a PASS verdict with cold-start time and query
  round-trip latency is recorded in the change's `design.md`

#### Scenario: portable-wasix fails to boot or serve correctly
- **WHEN** the spike attempts the same boot-and-query sequence and the
  server fails to start, or the query round-trip does not return the
  expected row
- **THEN** a FAIL verdict is recorded in the change's `design.md`,
  including the exact error output, so
  `desktop-data-layer-pglite-oxide` treats pglite-oxide as unavailable on
  `x86_64-apple-darwin` and relies solely on the embedded-SurrealDB
  baseline for that platform

### Requirement: Spike isolation from production code
The verification SHALL run in a standalone crate that is never referenced
as a dependency by any production crate in this repository, so its result
carries no build-time or runtime coupling regardless of the verdict.

#### Scenario: Spike crate has no inbound or outbound production dependency
- **WHEN** the spike crate at `spikes/pglite-oxide-intel-mac-spike/` is
  built or removed after the verdict is recorded
- **THEN** no production crate's build (`cargo check`/`build` at the
  workspace root, or `src-tauri`) is affected in either direction

