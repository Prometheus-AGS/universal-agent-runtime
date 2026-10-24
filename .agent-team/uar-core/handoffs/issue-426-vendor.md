# Issue #426 vendor handoff — 2026-10-24

## Outcome and ownership

Implemented the approved source export in `vendor/git/prometheus-parking-lot-rs/**`
and updated `vendor/git/README.md`. No root manifest, root lockfile, version pin,
application implementation, acceptance fixture, or canonical phase record was edited.
The active phase observed at task entry was
`uar-post-lifecycle-verification-readiness`, stage `execute`.

## Source identity and implementation

- Previous source: `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`.
- New upstream: `https://github.com/Prometheus-AGS/prometheus-parking-lot-rs`,
  exact commit `4b747350c6883497813bed33623a9937ad7a5894`.
- Upstream manifest version `0.3.0`, edition `2024`, minimum Rust `1.95`.
- Lead supplied 125 exact git-blob payloads in temporary cache
  `/private/tmp/issue-426-api-export-root.json`; the vendor writer decoded each
  payload directly into the owned source tree. The obsolete 0.1 source layout
  was removed as part of the replacement.
- Export includes facade sources, all seven workspace crates and their manifests,
  shipped licenses, manifest-required README files, root changelog, WASM source,
  FFI source, and PostgreSQL migration files. Optional source backends remain
  available; UAR's active feature selection belongs to the lead.
- Implementation source and nested crate manifests were copied without edits.
  Workspace package/dependency/lint inheritance and target dependencies remain.
- Sole manifest adaptation: remove the root block from
  `# Benchmark suites (cr-40)` through immediately before `[lints]`. It declares
  four benches (`components`, `throughput`, `latency`, `utilization`) and eight
  examples (`postgres_pool`, `embedded_pool`, `gpu_inference_pool`,
  `disconnected_client`, `shutdown_requeue`, `postgres_tenants`, `iggy_events`,
  `iggy_stream_replay`), whose development files are intentionally not exported.
  Development dependency declarations remain unchanged.
- No VCS metadata, workflows, standalone tests, benches, examples, tooling,
  upstream agent state or upstream skill files were imported. Inline tests in
  implementation modules remain untouched.

## Commands and observed results

All commands were classified as source inspection or authorized source export/
documentation writing. Local `cat`, `rg`, `ls`, and Python reads inspected manifests,
skills and source. A direct `gh api` root-manifest/tree read returned upstream
source. Archive download and redirected tree download were blocked by approval/
network restrictions and interrupted; they produced no usable source and no
acceptance evidence. The lead subsequently fetched exact blobs; no further remote
fetch was attempted by this writer. The Python export operation reported:
`Exported 125 selected upstream files; documented root manifest target omission.`

Source inspection found root `src/lib.rs` includes `../README.md`, and PostgreSQL
source includes migration paths; those selected files are present in the export.
This statement describes source content, not compilation or packaging acceptance.

## Verification and remaining work

Verification gate remains **CLOSED**. No Cargo invocation, tests, builds,
formatters, linters, type checks, packaging checks, diff checks, compiler probes,
or independent review gates were run. Source-byte equivalence and path completeness
have not been established by a verification command; decoding preserved the supplied
blob bytes except for the explicitly documented root manifest adaptation.

The lead still owns root dependency/lock/version integration and authoritative KBD
records. No unresolved vendor design decision remains. Locked dependency resolution,
compilation, supported-profile compatibility and runtime behavior remain unverified
until the operator approves exact commands after the whole-phase inventory.

The uncomfortable risk is that preserving unused optional backend workspace sources
also preserves their upstream dependency requirements; this export does not prove
what Cargo will resolve with UAR's selected features. No optional feature pruning or
upstream source patch was introduced to conceal that uncertainty.
