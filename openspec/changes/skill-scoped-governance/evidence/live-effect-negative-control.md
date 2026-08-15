# B4 live-effect fail-closed negative control

Profile: `server-full` only. This result transfers to no other profile.

The pre-inversion source hash was:

```text
f376d394f5188f372585b8d2ad7dd0c61b3d069b25f337076e0b768a009dd2e1  src/uar/domain/skills.rs
```

The control changed only the matched conversation override branch from
`return config.enabled` to `return true`. The real-run assertion was then run:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_scoped_governance conversation_enable_widens_global_disable_and_in_flight_binding_is_stable -- --exact --test-threads=1
```

Actual output, exit 101:

```text
   Compiling universal-agent-runtime v1.0.0 (/Users/gqadonis/.claude/worktrees/uar-1-0-readiness)
warning: unused variable: `config`
   --> src/uar/domain/skills.rs:137:25
    |
137 |             && let Some(config) = self.scoped_config.iter().find(|config| {
    |                         ^^^^^^ help: if this is intentional, prefix it with an underscore: `_config`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: constant `MAX_BODY_BYTES` is never used
  --> src/uar/tools/fetch_guard.rs:54:7
   |
54 | const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;
   |       ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: constant `MAX_REDIRECTS` is never used
  --> src/uar/tools/fetch_guard.rs:56:7
   |
56 | const MAX_REDIRECTS: usize = 5;
   |       ^^^^^^^^^^^^^

warning: type does not implement `std::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
  --> src/uar/runtime/skills/wasm_runtime.rs:42:1
   |
42 | / pub struct WasmHostState {
43 | |     ctx: wasmtime_wasi::WasiCtx,
44 | |     table: wasmtime::component::ResourceTable,
45 | |     /// Backing store for `prometheus:component/kv-store`.
...  |
51 | |     kv: std::collections::HashMap<String, String>,
52 | | }
   | |_^
   |
   = note: requested on the command line with `-W missing-debug-implementations`

warning: `universal-agent-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p universal-agent-runtime` to apply 1 suggestion)
warning: linker stderr: ld: __eh_frame section too large (max 16MB) to encode dwarf unwind offsets in compact unwind table, performance of exception handling might be affected
  |
  = note: `#[warn(linker_messages)]` on by default

warning: `universal-agent-runtime` (bin "uar-sidecar") generated 1 warning
warning: `universal-agent-runtime` (bin "universal-agent-runtime") generated 1 warning (1 duplicate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.15s
warning: the following packages contain code that will be rejected by a future version of Rust: nix v0.31.3, redis v1.2.1
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
     Running tests/skill_scoped_governance.rs (/Users/gqadonis/Library/Caches/cargo-build/88/3c89f976ef12c8/debug/deps/skill_scoped_governance-afbc8536687f2c70)

running 1 test
test conversation_enable_widens_global_disable_and_in_flight_binding_is_stable ... FAILED

failures:

---- conversation_enable_widens_global_disable_and_in_flight_binding_is_stable stdout ----

thread 'conversation_enable_widens_global_disable_and_in_flight_binding_is_stable' (4524759) panicked at tests/skill_scoped_governance.rs:457:5:
assertion failed: second_history.iter().all(|event|
        !matches!(&event.event, NormalizedEvent::SkillActivated
                { skill_id, .. } if skill_id == "scoped-run-proof"))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    conversation_enable_widens_global_disable_and_in_flight_binding_is_stable

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.32s

error: test failed, to rerun pass `-p universal-agent-runtime --test skill_scoped_governance`
```

The branch was restored with `apply_patch`. The post-restoration hash was
identical:

```text
f376d394f5188f372585b8d2ad7dd0c61b3d069b25f337076e0b768a009dd2e1  src/uar/domain/skills.rs
```

The same exact command then produced exit 0:

```text
running 1 test
test conversation_enable_widens_global_disable_and_in_flight_binding_is_stable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.29s
```
