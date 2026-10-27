# Corrected P0.006 exact commands

PROPOSED only. Product gate CLOSED. Five commands once within unchanged producer; serial first-failure stop. Explicit current-source native build after production edits; locked SDK+3queued gates; frontend/TS/install passes retained.

## P0.7

```bash
/Users/gqadonis/.cargo/bin/cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: native build and build scripts; Cargo dependency/cache network if required; Cargo JSON selects actual server bin; native target isolated from later Cargo commands; Explicit cached native rebuild exception after a previous PASS: bind current whole-source identity to native receipt required by unchanged artifact producer. No broader gate or frontend rebuild.

## P0.10

```bash
/Users/gqadonis/.cargo/bin/cargo build --locked --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/rust/Cargo.toml --example user_outcome_acceptance --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: compile SDK example and build dependencies; do not run it; Cargo JSON selects example executable; independent SDK lock

## P0.11

```bash
/Users/gqadonis/.cargo/bin/cargo test --locked --features server-full,postgres-backend --test user_outcome_grpc --no-run --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: explicit no-run test-harness compiler exception and build scripts; Cargo JSON selects user_outcome_grpc executable; no test execution

## P0.12-resolve-guest

```bash
/Users/gqadonis/.cargo/bin/cargo generate-lockfile --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/Cargo.toml
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: Resolve exact-direct-pinned guest dependencies into only the new private generated Cargo.lock; no repository source or product lock mutation; Private Cargo.toml/src/WIT copied from frozen guest templates before this stage

## P0.13-build-guest

```bash
/Users/gqadonis/.cargo/bin/cargo-component component build --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/Cargo.toml --locked --release --target wasm32-unknown-unknown --target-dir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/cargo-target
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: Compile/componentize current eligible-skill keyword classifier; emit private generated bindings and component; Explicit internal rustc --print sysroot and installed wasm32-wasip1 prerequisite inspection; missing target fails, no automatic rustup install; bindings.format=false prevents implicit rustfmt; no component instantiation or UAR strategy test
