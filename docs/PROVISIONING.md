# Runtime Tool Provisioning

This document describes how UAR resolves external executables it needs at
runtime — MCP stdio server commands today, and the machinery a future
compile-a-skill-from-source feature can reuse. It is distinct from
[`DEPENDENCY_MANAGEMENT.md`](DEPENDENCY_MANAGEMENT.md), which covers
compile-time Rust crate pinning in `Cargo.toml`.

## Why this exists

MCP servers configured with a `command` in `mcp.json` (e.g. `kreuzberg`) are
spawned via a plain child-process call. If the configured binary isn't on
`PATH`, the tool call used to just fail with no recovery. `src/uar/orchestrator/provisioning.rs`
adds a pluggable resolver so a missing-but-obtainable tool can be provisioned
automatically instead.

This is a deliberately different problem from
[`process_supervisor.rs`](../src/uar/orchestrator/process_supervisor.rs), which
detects and reuses an already-running **TCP-listening service** (a "probe the
port, adopt if it answers, else spawn" pattern). Nothing in this codebase
currently spawns a TCP service UAR needs to adopt in that sense — `provisioning.rs`
solves "does this executable exist yet," not "is this service already running."

## The 4-strategy precedence

`ToolProvisioner::resolve(spec, opts)` tries, in order, stopping at the first
strategy that succeeds:

1. **Adopt** — a `which`/`where`-style `PATH` lookup. Always tried first and
   free of side effects.
2. **NativePackageManager** — detects `apt-get`/`dnf` (Linux), `brew`
   (macOS), or `winget`/`choco` (Windows), and installs the tool's declared
   package name for that manager.
3. **GitInstall** — clones the tool's declared repo and runs its declared
   build command, matching the same clone-and-build pattern
   `prometheus-skill-system` and `prometheus-entity-management` already use
   as submodules.
4. **PrebuiltBinary** — downloads and extracts a release archive for the
   detected OS/architecture (via the OS's native `tar`, not a new
   archive-handling crate).

Each strategy is optional per `ToolSpec` — a spec can decline to declare a
native package, a git repo, or a prebuilt URL, in which case that strategy is
skipped.

## `ProvisionOptions::allow_install`

Strategies 2–4 are real, host-modifying actions: installing a system
package, cloning and building third-party source, or downloading and
extracting an archive. `ProvisionOptions::allow_install` defaults to `false`,
which means: if Adopt fails and a later strategy *would* apply, `resolve()`
returns an error naming which strategy would be used, rather than either
silently modifying the host or silently giving up. Callers that want
self-healing behavior set `allow_install: true` explicitly.

## Adding a new `ToolSpec`

Curated specs live next to the strategies they configure:

- `known_tool_spec(name)` — MCP server command names. Add a match arm for a
  new curated tool; anything not matched falls back to an Adopt-only spec
  (never invents an installation strategy for an uncurated name).
- `skill_toolchain_specs()` — the 5 skill-compilation toolchains
  (Rust/Node/Python/Go/wasmtime), mirroring the Dockerfile's own install
  choices where practical so the two don't drift.

## Non-goals

- **Database engines are out of scope.** SurrealDB and Postgres are assumed
  pre-provisioned by whoever deploys UAR; this module does not manage them.
- **Not currently wired to skill compilation.** UAR has no code path today
  that compiles a skill from source — the Dockerfile's own comment confirms
  its 5 toolchains are kept resident "for user builds" (a human manually
  building their own skill inside the running container), not something
  UAR's runtime invokes automatically. `skill_toolchain_specs()` exists as a
  ready-to-use, tested recipe for whenever that feature is built, not as an
  active integration.
- **`surreal-memory` and `liter-llm` are not provisioned targets.** Both are
  linked Rust libraries used in-process (`surreal_memory::Memory`,
  `liter_llm::DefaultClient`), not separate spawnable processes.

## Test coverage and a disclosed gap

Unit tests cover: the Adopt strategy (success and failure), native-package
selection logic, the full resolution fallthrough, both error paths (no
strategy declared; a strategy exists but `allow_install` is `false`), the
curated `kreuzberg` spec, the Adopt-only fallback for uncurated names, and a
real (non-mocked) Adopt-path resolution of `rustc` — this crate's own build
guarantees `rustc` is present, so that test exercises a genuine case, not a
synthetic fixture.

**Not covered by an automated test:** actually installing something via
strategies 2–4 (native package manager, git-install, prebuilt-binary) end to
end. Doing so would modify whatever host runs the test suite — not
acceptable for CI. If this module gains a real caller with `allow_install:
true` in production, add an explicitly `#[ignore]`d, manually-run test
exercising at least one real install path (e.g. in a disposable container),
rather than silently leaving this gap undocumented.
