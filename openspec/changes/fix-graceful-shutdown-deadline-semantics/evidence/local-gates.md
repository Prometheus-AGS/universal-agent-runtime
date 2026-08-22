# Local build and scope gates

Date: 2026-08-22

Profile: `server-full` only. No result transfers to another profile.

Base SHA: `32afa53d510c8b840b3e98b2be9d9f5dee149531`

## Tier 0 check

```bash
cargo check --locked --no-default-features --features server-full
```

Observed exit `0`:

```text
Checking universal-agent-runtime v1.0.0
warning: constant `MAX_BODY_BYTES` is never used
warning: constant `MAX_REDIRECTS` is never used
warning: type does not implement `std::fmt::Debug` (`WasmHostState`)
warning: `universal-agent-runtime` (lib) generated 3 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.35s
```

The three warnings predate and are outside this child surface. They are
reported, not relabeled as clean and not changed.

## Scoped Clippy

```bash
cargo clippy --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib --no-deps
```

Observed exit `0`:

```text
warning: `universal-agent-runtime` (lib) generated 571 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.83s
```

This repository enables pedantic warnings without denying them for this
command. The output contains the known existing warning inventory; this child
does not claim warning-free Clippy.

## OpenSpec, shell, dependency, and visibility gates

Commands:

```bash
openspec validate fix-graceful-shutdown-deadline-semantics --strict --no-interactive
bash -n scripts/certify-release-candidate.sh
git diff --check -- scripts/certify-release-candidate.sh \
  src/mcp/registry.rs src/server.rs src/uar/realtime/surreal_bus.rs \
  tests/integration/live/harness.rs tests/integration/live/capability_cases.rs \
  openspec/changes/fix-graceful-shutdown-deadline-semantics \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-graceful-shutdown-deadline-semantics
git diff --exit-code -- Cargo.toml Cargo.lock
git diff -U0 -- src/mcp/registry.rs src/server.rs \
  src/uar/realtime/surreal_bus.rs | rg '^\\+[^+].*\\bpub(\\(|\\s)' || true
```

Observed output, combined exit `0`:

```text
Change 'fix-graceful-shutdown-deadline-semantics' is valid
ADDED_RUST_VISIBILITY
+    pub(crate) async fn shutdown(&self) {
+    pub(crate) async fn shutdown(&self) {
```

The shell and scoped diff checks emitted no errors. `Cargo.toml` and
`Cargo.lock` have no diff. The only added product visibility is the two
crate-private shutdown methods; no public Rust API was introduced.
