# Verification — `resolve-sdk-distribution`

Date: 2026-08-18

Scope: Rust SDK default (`http-client`) on macOS, Python SDK on CPython 3.13,
TypeScript SDK on Node.js, and UAR `server-full` Tier 0 on macOS. Results transfer
to no other SDK feature set, interpreter/runtime, profile, or platform.

| requirement | assertion observed | negative control observed | command | result |
|---|---|---|---|---|
| Selected SDKs are licensed, locally verified, and release-ordered. | Rust, Python, and TypeScript report version `1.0.0`, MIT, and `Prometheus AGS`; their focused tests, examples/type checks, package contents, and generated docs passed. Customer docs use the exact package names. Rust metadata retains `universal-agent-runtime = =1.0.0` with its local path. | Rust package preparation before its sibling runtime exists on crates.io failed with `no matching package named universal-agent-runtime found`. Root metadata then exposed four path-only normal dependencies, proving that runtime-first alone is insufficient. | Exact commands and observed output are recorded in `evidence/positive-verification.md` and `evidence/negative-controls.md`. | All three SDKs are selected and locally verified. Rust registry publication remains blocked on the recorded four-step internal-crates → runtime → SDK chain; no publishable-now claim is made. |
| Routine SDK verification stays local. | The legacy all-routine `.github/workflows/ci.yml` is retired and all SDK verification commands are retained in local evidence. No deployment workflow changed. | A file-existence control fails if the routine workflow remains; the observed branch printed `legacy routine CI workflow absent`. | `if test -e .github/workflows/ci.yml; then echo 'unexpected routine CI workflow remains'; exit 1; else echo 'legacy routine CI workflow absent'; fi`<br>`git diff --name-only -- .github/workflows` | Control: exit 0 with `legacy routine CI workflow absent`. Scoped diff names only `.github/workflows/ci.yml`. |
| Change-level gates pass. | Strict OpenSpec, scoped diff checking, UAR `server-full` Tier 0, and artifact-refiner schema/reference gates are required. | Not a fail-closed product assertion. The independent artifact critic and judge review the corrected final candidate without generation history. | Exact commands and outputs are recorded in `evidence/positive-verification.md` and `evidence/artifact-refiner-validation.md`. | OpenSpec and diff checks pass. UAR check exits 0 with 3 existing warnings; scoped Clippy exits 0 with 571 existing warnings. No warning-free claim. Phase Tier 2 remains deferred. |
