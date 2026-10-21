# Approved-gate runtime warning repair — 2026-10-21

## Scope

This repair follows the terminal approved embedding gate. It addresses only the assigned first-party warnings in the runtime lifecycle, sandbox session manager, lifecycle reconnect, realtime lifecycle, MCP registry, and session thread sources.

## Changed paths and effects

- `src/sandbox/session_manager.rs`
- `src/server/lifecycle_reconnect.rs`
- `src/uar/realtime/lifecycle.rs`
- `src/uar/runtime/lifecycle.rs`

The redundant `std::future::Future` imports were removed because Rust 2024 provides `Future` through the prelude. Existing bounds and type aliases retain the same name and behavior.

`src/uar/runtime/lifecycle.rs` also removes the redundant `mut` receiver from `OperationLease::unresolved`. Its ownership and completion behavior remain unchanged because `complete` receives the lease by value. Manual lock-free `Debug` implementations now cover `WeakResourceScope`, `OperationLease`, `RuntimeSupervisor`, and `RuntimeHandle`. They report only safe state or a non-exhaustive marker; lifecycle internals, operation identifiers, retained state, and task handles are not exposed.

- `src/server/lifecycle_control.rs`

Manual `Debug` implementations cover the host control capability and its wire request types. They retain only the protocol version, operation kind, and reconnect generation. The backing control state, request ID, and resource ID are redacted. `Deserialize` fields and the control wire format are unchanged.

- `src/mcp/registry.rs`

The private compatibility `shutdown` helper is now compiled only for tests. All observed uses are in the file's test module; production host finalization continues to use `shutdown_report`.

- `src/session/thread.rs`

The private `Session::new` helper is now compiled only for its two unit-test callers. Production session creation continues through `new_with_activity`.

## Verification status

No test, check, build, lint, formatter, compiler probe, or review command was run. The repair cohort remains unverified until the lead renews the source inventory and reruns the same approved embedding gate.

## Limits

The terminal gate also reported warnings assigned to other owners, including memory/persistence source and vendored dependencies. Those files were not changed here.
