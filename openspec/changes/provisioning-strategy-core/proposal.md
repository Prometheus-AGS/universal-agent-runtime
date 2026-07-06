# provisioning-strategy-core

## Why

Closes the redesigned `binary-instance-discovery` (#4) from
`prometheus-package-integration`. Two real gaps existed: MCP stdio
server commands (`kreuzberg`, `npx`-based entries, etc.) are spawned
via bare `Command::new()` in `mcp/registry.rs` with zero provisioning —
if the configured command isn't on `PATH`, the tool call just fails,
with no recovery. Skill-compilation toolchains (Rust/Node/Python/Go/
wasmtime) have no resolution logic at all for local dev outside the
container (inside the shipped image, `#13`'s polyglot Dockerfile
already bakes them all in).

`process_supervisor.rs` (the module the original plan named for this
work) solves a different problem — detecting and reusing an
already-running **TCP-listening service** — and has no current call
site (nothing in this codebase spawns a TCP service UAR would need to
adopt; `surreal-memory-server` and `liter-llm`, the two other
"binaries" the original plan named, turned out to be linked Rust
libraries used in-process, not separate processes at all). It is left
untouched.

## What changed

New module `src/uar/orchestrator/provisioning.rs`:

- `ToolSpec` — declares one tool: its `PerOsPackageName` (native package
  manager names per OS), an optional `GitInstallSpec` (repo URL + build
  command + resulting binary path), an optional `PrebuiltSpec` (release
  archive URL template with `{os}`/`{arch}` placeholders + binary path
  inside the archive).
- `ToolProvisioner::resolve(spec, opts) -> Result<ProvisionOutcome>` —
  tries, in order: **Adopt** (a `which`-style `PATH` lookup, mirroring
  `build.rs:495`'s existing helper — can't literally share code with a
  build script, so re-implemented identically rather than duplicated
  ad hoc); **NativePackageManager** (detects `apt-get`/`dnf` on Linux,
  `brew` on macOS, `winget`/`choco` on Windows, and installs if found);
  **GitInstall** (clone + build from source, same pattern
  `prometheus-skill-system`/`prometheus-entity-management` already use
  as submodules); **PrebuiltBinary** (download + extract via the OS's
  native `tar`, not a new archive-handling crate).
- `ProvisionOptions::allow_install` (default `false`) gates strategies
  2–4 from actually modifying the host — installing a system package,
  cloning+building third-party source, or downloading and extracting
  an archive are all real, host-modifying actions this module will not
  take silently. With it `false`, a missing tool with a viable strategy
  reports *which* strategy would be used and why it didn't run, rather
  than either silently installing something or silently failing.

## Verification

- 7 new unit tests: `which()` finds a known-present binary (`git`,
  already a hard dependency of this project's submodule workflow) and
  correctly returns `None` for a fabricated nonexistent name;
  `native_package_for_this_os` picks `apt` over `dnf` on Linux and
  returns `None` when nothing is declared; `resolve()` adopts an
  already-installed tool without needing `allow_install`; `resolve()`
  errors (without installing anything) when `allow_install` is `false`
  and a strategy would otherwise apply; `resolve()` errors when no
  strategy is declared at all.
- `cargo test --lib`: 379/379 green (372 baseline + 7 new).
- `cargo clippy --lib`: zero new warnings (502 before and after,
  matching this session's established baseline).
- Native-package-manager installation, git-install, and prebuilt-binary
  download/extract are exercised by the fallthrough/error-path tests
  above but not by an end-to-end "actually installs something" test —
  doing so would modify this CI/dev environment's host. Disclosed as a
  real gap, not silently skipped: `provisioning-tests-and-docs` (a
  later change in this same round) should add an explicitly-`#[ignore]`d,
  manually-run test for at least one real install path.
