# B4 fail-closed negative control

Profile: `server-full` only. This result transfers to no other profile.

The pre-inversion source hash was:

```text
cd81693b96bb3c1f1dfdfa6362aedbacafaa748359dc2c276d261a1b6d65547c  src/uar/runtime/skills/registry.rs
```

The control removed only the branch that copies `enabled` and `scoped_config`
from the stored builtin row before re-registration. The exact child-process
cold-restart assertion was then run:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_scoped_governance scoped_state_and_user_deletion_survive_cold_restart -- --exact --test-threads=1
```

Actual output, exit 101:

```text
running 1 test
test scoped_state_and_user_deletion_survive_cold_restart ... FAILED

failures:

---- scoped_state_and_user_deletion_survive_cold_restart stdout ----

thread 'scoped_state_and_user_deletion_survive_cold_restart' (4446187) panicked at tests/skill_scoped_governance.rs:341:9:
B4 reopen-delete child failed
stdout:

running 1 test
test scoped_state_and_user_deletion_survive_cold_restart ... FAILED

failures:

---- scoped_state_and_user_deletion_survive_cold_restart stdout ----

thread 'scoped_state_and_user_deletion_survive_cold_restart' (4446215) panicked at tests/skill_scoped_governance.rs:227:17:
global and per-agent disables survive a cold reopen
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    scoped_state_and_user_deletion_survive_cold_restart

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.08s

stderr:

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    scoped_state_and_user_deletion_survive_cold_restart

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.32s

error: test failed, to rerun pass `-p universal-agent-runtime --test skill_scoped_governance`
```

The assignments were restored with `apply_patch`. The post-restoration hash was
identical:

```text
cd81693b96bb3c1f1dfdfa6362aedbacafaa748359dc2c276d261a1b6d65547c  src/uar/runtime/skills/registry.rs
```

The same exact command then produced exit 0:

```text
running 1 test
test scoped_state_and_user_deletion_survive_cold_restart ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.31s
```
