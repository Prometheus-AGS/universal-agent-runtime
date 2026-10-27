# Corrected P0.003 exact commands

Proposed only. Product verification CLOSED. Run each command once within the artifact producer, in order; no standalone preflight. Exact per-child environment references are in the JSON packet and frozen producer draft.

## P0.7

```bash
/Users/gqadonis/.cargo/bin/cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend --message-format=json-render-diagnostics
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: native build and build scripts; Cargo dependency/cache network if required; Cargo JSON selects actual server bin; native target isolated from later Cargo commands

## P0.8

```bash
/Users/gqadonis/.volta/bin/pnpm -C /Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend build
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`

Effects: Vite product build; empties and rewrites repository/static; no test/lint/typecheck script wrapper

## P0.9

```bash
/Users/gqadonis/.volta/bin/node /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/node_modules/typescript/lib/tsc.js --project /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/tsconfig.examples.json --noEmit false --outDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/.acceptance-artifacts --rootDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript
```

cwd: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript`

Effects: explicit compiler/emit exception; emits example and src JavaScript/declarations; Node entry .acceptance-artifacts/examples/user-outcome-acceptance.js; preserve SDK type:module package.json and node_modules

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
