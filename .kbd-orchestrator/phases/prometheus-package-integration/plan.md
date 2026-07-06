PLAN: prometheus-package-integration (resumed, redesigned scope)
Project: universal-agent-runtime
Date: 2026-07-06
OpenSpec available: YES
Changes to implement: 4 (of the original 14, only #4's replacement work remains — see assessment.md)

## Framing

`assessment.md` confirmed 12 of the original 14 changes are done; #13 is
confirmed done too (initially mischecked) and the user confirmed it
should not be touched. Only #4 (`binary-instance-discovery`) is real
remaining work, and its actual shape changed twice during assessment:

- `surreal-memory-server`/`liter-llm` aren't spawnable processes at all
  (in-process libraries) — dropped from scope entirely.
- MCP stdio servers (the one real, currently-spawned dependency) are
  **stdio-piped, not TCP-listening** — `process_supervisor.rs`'s
  existing probe-port-then-adopt-or-spawn logic is built for a
  different shape of dependency (a TCP service you can detect and
  reuse) and doesn't apply to a stdio child process (each connection
  is inherently a fresh process; there's nothing to "adopt"). What
  stdio spawning and toolchain resolution actually both need is
  simpler: **"does this executable exist on this machine — if not,
  install it via the best available strategy."**

Given that, this plan does **not** modify `process_supervisor.rs`
(`Supervisor`/`ManagedBinary`/`AdoptionResult`) at all — it remains
valid, unmodified infrastructure for a TCP-listening-service shape of
dependency that simply has no current call site. Instead, a new,
focused module handles the "ensure this executable/toolchain exists"
problem, used by both real integration points (MCP stdio spawn,
skill-compilation toolchain resolution).

## CHANGE LIST (ordered)

1. **provisioning-strategy-core**: the pluggable resolution engine
   - Scope: new module `src/uar/orchestrator/provisioning.rs`
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
   - Customer value: MEDIUM (foundational — no user-visible behavior
     change alone, but everything else depends on it)
   - Details: `ToolSpec { name: &str, native_pkg: PerOsPackageName,
     git_repo: Option<GitInstallSpec>, prebuilt: Option<PrebuiltSpec> }`
     describing how to obtain one tool. `ToolProvisioner::resolve(spec)
     -> Result<PathBuf>` tries, in order: (1) **Adopt** — `which`-style
     PATH lookup (reuse the existing `which()` helper pattern from
     `build.rs:495`, promoted to a shared location rather than
     duplicated); (2) **NativePackageManager** — detect the current OS
     (`cfg!(target_os = "...")`) and probe for `apt-get`/`dnf` (Linux),
     `brew` (macOS), `winget`/`choco` (Windows) via a version-check
     `Command`, install if found; (3) **GitInstall** — clone the
     tool's repo (a temp or persistent cache dir under
     `dirs::cache_dir()`) and run its declared build command; (4)
     **PrebuiltBinary** — download + extract a release archive for the
     detected OS/arch. Each strategy is independently unit-testable
     (mock the `Command`/HTTP layer); the resolution order itself gets
     a table-driven test proving fallthrough behavior (first strategy
     that succeeds wins, remaining ones are skipped).
   - Verify: unit tests per strategy + resolution-order fallthrough;
     `cargo check`/`cargo clippy` clean.

2. **wire-mcp-server-provisioning**: use the provisioner before
   spawning MCP stdio servers
   - Scope: `src/mcp/registry.rs` (the `McpServerEntry::Stdio` arm in
     `from_config`)
   - Depends on: 1
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: MEDIUM (today, if a configured MCP command isn't
     on `PATH`, the tool call just fails with no recovery — this makes
     it self-healing)
   - Details: before `resolve_mcp_command(command)` /
     `Command::new(&command_path)`, call
     `ToolProvisioner::resolve(ToolSpec::for_mcp_command(command))` —
     only invoked when the plain `command` string doesn't already
     resolve via a direct PATH check (preserve today's fast path for
     the common case where it's already installed). Failure to
     provision surfaces the same `set_mcp_server_status(name, false)` +
     error path that already exists for a failed connection, not a new
     error type.
   - Verify: an integration test with an intentionally-missing/fake MCP
     command name, confirming the provisioning attempt is made and its
     failure surfaces through the existing error path; existing MCP
     integration tests (`test_mcp_optional`, `tests/integration`)
     unaffected.

3. **wire-toolchain-provisioning**: resolve skill-compilation
   toolchains for local dev
   - Scope: `src/uar/runtime/skills/wasm_runtime.rs` and/or
     `builtin_loader.rs` (wherever a skill's declared language
     requirement is checked before compilation is attempted)
   - Depends on: 1
   - Recommended agent: Claude Code
   - Est. complexity: M · Complexity score: Medium · Model class: mid
   - Customer value: LOW-MEDIUM (only matters for local dev outside the
     container — inside the shipped image, #13's polyglot Dockerfile
     already has every toolchain, so `Adopt` is the expected outcome
     there every time)
   - Details: `ToolSpec` entries for `rustc`, `node`, `python3`, `go`,
     `wasmtime` (matching #13's Dockerfile's own toolchain list, so the
     two stay in sync). Resolved once, cached for the process lifetime
     (not re-resolved per skill load).
   - Verify: unit test confirming `Adopt` succeeds trivially when the
     container's baked-in toolchains are present (the common case in
     CI); explicit test (behind a feature flag or `#[ignore]`, since
     it would actually invoke a package manager) for the fallback
     chain on a machine missing a toolchain, documented as manually run.

4. **provisioning-tests-and-docs**: close out the original #14 for
   this reduced scope
   - Scope: new `docs/PROVISIONING.md` (or a section in
     `docs/DEPENDENCY_MANAGEMENT.md`), plus whatever test gaps remain
     after changes 1–3's own verification
   - Depends on: 1, 2, 3
   - Recommended agent: Claude Code
   - Est. complexity: S · Complexity score: Low · Model class: small
   - Customer value: LOW (documentation + coverage closure) but
     directly addresses the Spec Gap Summary this assessment surfaced —
     no canonical doc currently explains this contract
   - Details: document the 4-strategy precedence, how to add a new
     `ToolSpec`, and the explicit non-goal (this system does not
     provision database engines).
   - Verify: `cargo test --lib` full suite green; doc reviewed for
     accuracy against the actual shipped code (not aspirational).

## EXECUTION ROUND ORDER

- **Round 1** (foundational): `provisioning-strategy-core`
- **Round 2** (parallel, one shared checkpoint): `wire-mcp-server-provisioning`,
  `wire-toolchain-provisioning`
- **Round 3** (closes out): `provisioning-tests-and-docs`

Round 2's shared checkpoint: `cargo test --lib` full suite green,
`cargo clippy` zero new warnings, plus a manual confirmation that
`mcp.json`'s existing `kreuzberg`/`time` entries still connect
successfully (the fast-path-when-already-installed case must not
regress).

## COMMANDS TO RUN

```
/opsx:new provisioning-strategy-core
/opsx:new wire-mcp-server-provisioning
/opsx:new wire-toolchain-provisioning
/opsx:new provisioning-tests-and-docs
```

Per this project's established practice, these
`openspec/changes/<id>/proposal.md` + `tasks.md` pairs are written
per-change at execute time.

## Sycophancy self-check

- S-02: this plan explicitly does NOT claim to "fix" or "improve"
  `process_supervisor.rs` — it correctly identifies that module as
  solving a different problem shape with no current call site, and
  leaves it untouched rather than force-fitting new logic into it for
  the sake of reusing the original change's name.
- S-03: explicit trade-off named — the toolchain-provisioning fallback
  chain (package-manager/git-install/prebuilt-binary) is real but low
  in expected day-to-day value, since the shipped container already
  has every toolchain baked in; it matters for local dev only. Sized
  accordingly (M complexity, LOW-MEDIUM customer value), not oversold.
- S-07: no scope creep — scope is exactly what survived two rounds of
  correction and explicit user confirmation: MCP stdio spawning +
  5 toolchains, no database engines, no invented sidecar processes for
  `surreal-memory-server`/`liter-llm`.

PLAN COMPLETE
