## Why

`pglite-oxide` (the embedded-PostgreSQL crate proposed as an optional
desktop data-layer enhancement in `desktop-data-layer-pglite-oxide`) ships
no `x86_64-apple-darwin` AOT release asset as of 0.5.1 — only
`aarch64-apple-darwin`, Linux (x86_64/aarch64), Windows x86_64, and a
`portable-wasix` fallback. This project's primary dev machine is an Intel
Mac, so whether pglite-oxide is usable here at all is currently unverified.
Before any production code depends on it, we need empirical evidence —
does the portable-WASIX build actually boot and perform acceptably on
`x86_64-apple-darwin` — not an assumption either way.

## What Changes

- Add a standalone spike (a throwaway crate or `examples/` binary, not
  wired into any runtime path) that boots `PgliteServer` via
  pglite-oxide's `portable-wasix` asset on `x86_64-apple-darwin`.
- Measure and record: cold-start time, and basic query latency (a trivial
  `SELECT` round-trip) against the booted instance.
- Record a PASS/FAIL verdict plus the measured numbers in this change's
  own design doc, so `desktop-data-layer-pglite-oxide` can read the result
  when it decides whether to wire pglite-oxide in as an enhancement.
- No production code changes. No dependency is added to the root
  workspace `Cargo.toml` — the spike lives entirely inside this change's
  own scratch scope and is discarded (or kept as a documented reference)
  after the verdict is recorded.

## Capabilities

### New Capabilities
- `pglite-oxide-portability`: records the platform-support verification
  evidence (PASS/FAIL + cold-start/query-latency numbers) for running
  pglite-oxide's portable-WASIX build on `x86_64-apple-darwin`, gating
  whether `desktop-data-layer-pglite-oxide` wires it in as an enhancement.

### Modified Capabilities
(none — this is a standalone verification change with no existing
capability's runtime behavior changing)

## Impact

- **Runtime UX**: none — this is a pre-implementation spike with no
  runtime wiring; it does not touch `desktop-shell` or any served surface.
- **Provider compatibility**: none.
- **Realtime state**: none.
- **KBD workflow state**: on completion, updates
  `.kbd-orchestrator/phases/uar-hybrid-app-architecture/library-candidates.json`
  (`cand-001`'s `open_questions`) and `decision-log.md` with the verdict,
  and clears the gating note on `desktop-data-layer-pglite-oxide` (change
  11) so it can proceed with a resolved (not merely deferred) enhancement
  decision.
- **Dependencies**: `pglite-oxide` + `pglite-oxide-assets` are added only
  to the spike's own isolated `Cargo.toml`, never to the workspace root.
