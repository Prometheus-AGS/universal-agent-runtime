# #462 local safeguard implementation handoff

Phase: `production-readiness-476-baseline`, Execute. Verification gate: CLOSED
throughout this implementation. This handoff claims authored implementation,
not accepted behavior. Parent owns waypoint, full-phase inventory, and gates.

## Files changed

- `scripts/verify-rust-test-baseline.sh`: new Bash entrypoint; serial locked,
  no-default-features, library compile-only commands for `minimal`, `server-full`,
  and `server-full,postgres-backend`. Default mode explicitly labels working-tree
  scope. `--acceptance` requires clean committed source and recursively clean,
  initialized submodules matching Git pins; prints commit/tree, submodule,
  toolchain, target, profile, and command identities; compares source/toolchain
  identities after compilation. Directory lock spans the three commands. Cargo
  failures propagate via pipefail; full output is streamed, color disabled, and
  `warning:`/`warning[code]:` diagnostics cause failure without lint suppression.
- `lefthook.yml`: added local `rust-test-baseline` command, selected by root and
  nested Rust/Cargo files, root Rust config/toolchain files, and the script.
  Existing commands preserved. No CI/workflow changes.
- `docs/development/rust-test-baseline.md`: new invocation, acceptance evidence,
  scope, failure/refusal, lock, phase-lock, and limitations documentation.
- This assigned handoff file.

## Source inspection and execution record

Read AGENTS, authoritative waypoint and phase plan, root Cargo features/profiles,
`.cargo/config.toml`, `rust-toolchain.toml`, `.gitmodules`, Lefthook config,
existing local certification/evidence scripts, relevant gotchas and Rust rules.
Loaded Prometheus Rust Workspace, Rust Best Practices, and phase-gated reference.
Workspace MCP orientation was read-only. Read official Lefthook glob documentation
to preserve root/nested matching semantics:
https://lefthook.dev/configuration/glob/ .

All tool commands were source inspection or authorized implementation/document
writes. No verification commands ran: no shell syntax execution, script run,
mock fixture, Cargo/toolchain probe, hooks, tests, build, lint, formatter,
`git diff --check`, or review-driven verification. No pass evidence exists yet.

## Guard rationale and remaining limits

Clean/pinned source requirements and the post-run comparison address #462's
temporary-source-patch evidence defect. Serial execution and directory ownership
implement the existing single-writer boundary. Warning rejection implements the
operator's zero-warning requirement; failure propagation prevents false passes.
No speculative guard or unrelated implementation was added.

Acceptance snapshots are not continuous file monitoring or a hermetic environment
receipt. Other Cargo writers still need scheduling; tools ignoring this script's
directory lock are outside its exclusion guarantee. `CARGO_TARGET_DIR` is honored;
otherwise the inspected repository Cargo config selects `target`. Interrupted
processes can leave a directory lock requiring deliberate removal after the owner
stops. Compilation does not execute tests or establish runtime behavior.

No dependency policy decision is made here: recursive identities follow committed
Git pins. The lead owns the reported `versions.toml` versus liter gitlink drift.
No sources/dependencies were reset or patched by this implementation.

## Pending lead gate

After the complete phase inventory and operator authorization, execute the planned
acceptance command and synthetic hook/refusal/failure controls. Script syntax,
Lefthook selection, error/warning propagation, dirty/submodule refusal, concurrent
invocation refusal, identity-change refusal, and all three real compiles remain
unverified. No unresolved implementation decision is delegated back to the lead.
