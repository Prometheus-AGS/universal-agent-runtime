# SDK distribution verification summary

Scope: Rust SDK default (`http-client`) on macOS, Python SDK on CPython 3.13,
TypeScript SDK on Node.js, and UAR `server-full` Tier 0 on macOS. Results
transfer to no other SDK feature set, interpreter/runtime, profile, or platform.

- Release metadata: PASS locally. Rust, Python, and TypeScript remain 1.0.0,
  MIT-licensed, and attributed to Prometheus AGS. Website install commands match
  each manifest's package name.
- Rust SDK default: PASS locally. Three unit tests and one doctest passed;
  examples compiled; rustdoc generated; metadata and package contents were
  inspected. The runtime compiled as a test-only dependency with five warnings
  in this profile; no warning-free claim is made.
- Rust publication order: CONTROL OBSERVED. Package preparation before the
  runtime exists on crates.io exited 101 with `no matching package named
  universal-agent-runtime found`. Root metadata then exposed four path-only
  normal dependencies. The recorded blocking order is internal crates, verified
  runtime registry versions/package, runtime 1.0.0, then SDK 1.0.0. Neither Rust
  crate is reported as registry-publishable today.
- Python SDK: PASS locally. Four tests passed; Ruff, strict mypy, wheel/sdist,
  Sphinx `-W`, and wheel-content inspection passed.
- TypeScript SDK: PASS locally. Four tests passed; lint, typecheck, CJS/ESM/DTS
  build, TypeDoc, and dry-run package inspection passed. Production-only audit
  found zero vulnerabilities; the development install reported one
  high-severity finding, so no full-graph vulnerability-free claim is made.
- Workflow policy: PASS locally. The legacy all-routine `ci.yml` workflow is
  retired and no deployment workflow changed.
- UAR Tier 0: PASS within the named baseline. `server-full` check exits 0 with
  three existing warnings; scoped Clippy exits 0 with 571 existing warnings.
- Tier timing: full phase Tier 2 remains deferred until all active-phase changes
  are complete.

Iteration 1 was blocked on workflow, lockfile, and incomplete-state findings.
Iteration 2 closed those but exposed an incomplete runtime-first publication
claim. Iteration 3 records the full prerequisite chain and makes no
publishable-now claim. Independent critic and judge both PASS this exact
candidate.
