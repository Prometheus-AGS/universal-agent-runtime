# Remote host correction evidence — 2026-09-04

The original phase suite passed without exercising these production admission
and cancellation branches. This receipt supplements, not replaces, that history.

Command:

```sh
cargo check --locked --no-default-features --features server-full
cargo fmt --all -- --check
cargo test --locked --no-default-features --features server-full --lib uar::runtime::thread::service::tests -- --nocapture
```

Observed final run: exit 0. T0 completed in 14.99s with zero warnings;
format check passed; test build completed in 40.48s. Actual test output:

```text
running 3 tests
test uar::runtime::thread::service::tests::remote_admission_refusal_does_not_lease_root_budget ... ok
test uar::runtime::thread::service::tests::remote_persisted_launch_refusal_releases_capacity_and_shutdown_settles_record ... ok
test uar::runtime::thread::service::tests::remote_pending_cancellation_releases_capacity_before_any_dispatch ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 711 filtered out; finished in 1.48s
```

The fixture creates a real RunManager/actor root, captures its ThreadService,
and uses SurrealKV persistence. It verifies unused 1,000-token capacity can be
reserved again, expected child records remain present as Cancelled, all accepted
jobs join, repeated shutdown succeeds, and no connection reaches the peer socket.
Shutdown and join assertions have explicit deadlines.

The first test-compilation attempt used the feature-disabled in-memory backend
and exited 101; the fixture was corrected without changing features or pins.
The first runtime attempt failed all three tests because a UAR-mode root copied
its routing mode into an agent-only remote child contract. Production contract
construction now selects Agent mode for the named child while preserving resource,
approval and budget ceilings. The default-root fixture was retained unchanged.

Uncomfortable limits: occupied slots are admission reservations, not executing
local children; launch refusal is forced through private host state; cancellation
is placed directly before the child worker is polled. This is a host-path
integration test compiled in the library target, not proof of a live remote peer,
root-cancellation races, dispatched-but-uncertain cleanup, or remote billing
termination. The full phase suite remains a separate gate.
