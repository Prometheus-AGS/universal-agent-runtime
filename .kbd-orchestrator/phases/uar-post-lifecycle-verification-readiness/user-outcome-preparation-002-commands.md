# Corrected P0.002 command packet

PROPOSED / NOT EXECUTED. Product gate CLOSED. Replaces the stopped P0 preparation instruction only after new explicit approval and material signed amendment. All original files/failed receipts remain unchanged.

P0.1 exact command, with child environment CI=true:

```sh
CI=true /Users/gqadonis/.volta/bin/pnpm install --frozen-lockfile --ignore-scripts
```

Packet mapping is environment_env.CI=UAR_PREP_NONINTERACTIVE, with the declared parent reference value exactly "true". It applies only to this child, not subsequent builds or the user shell/profile. No --force, settings mutation or pnpm upgrade.

The retained pnpm error explicitly recommends CI=true; official upstream context: https://github.com/pnpm/pnpm/issues/11562 . This prepares the documented fix, without claiming it has been exercised.

| Stage | Exact argv | Child environment delta |
|---|---|---|
| P0.1 | `/Users/gqadonis/.volta/bin/pnpm install --frozen-lockfile --ignore-scripts` | `CI=true` |
| P0.2 | `/Users/gqadonis/.volta/bin/npm --prefix /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript ci --ignore-scripts --no-audit --no-fund` | None |
| P0.3 | `/Users/gqadonis/.volta/bin/pnpm rebuild esbuild` | None |
| P0.4 | `/Users/gqadonis/.volta/bin/pnpm exec playwright install chromium` | Named references in JSON |
| P0.5-pull | `/usr/local/bin/docker pull pgvector/pgvector:pg17@sha256:ac08538c6f8b9904c33c8224c5e5706dbe760aca29db1d096972b4052c22a75d` | None |
| P0.5 | `/usr/local/bin/docker image inspect --format {{json .}} pgvector/pgvector:pg17@sha256:ac08538c6f8b9904c33c8224c5e5706dbe760aca29db1d096972b4052c22a75d` | None |
| P0.python-dependencies | `/Users/gqadonis/miniconda3/bin/python3 -m pip install --disable-pip-version-check --no-cache-dir --no-compile --no-warn-conflicts --no-deps --only-binary=:all: --require-hashes --target /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/python-site --report /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/python-dependencies-install.json -r /Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/fixtures/python-locked-requirements.txt` | None |
| P0.python-sdk | `/Users/gqadonis/miniconda3/bin/python3 -m pip install --disable-pip-version-check --no-cache-dir --no-compile --no-warn-conflicts --no-index --no-deps --no-build-isolation --target /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/python-site --report /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/python-sdk-install.json /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/python-sdk-source` | Named references in JSON |
| P0.6-pull | `/usr/local/bin/docker pull surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20` | None |
| P0.6 | `/usr/local/bin/docker image inspect --format {{json .}} surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20` | None |
| P0.7 | `/Users/gqadonis/.cargo/bin/cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend --message-format=json-render-diagnostics` | Named references in JSON |
| P0.8 | `/Users/gqadonis/.volta/bin/pnpm -C /Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend build` | None |
| P0.9 | `/Users/gqadonis/.volta/bin/node /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/node_modules/typescript/lib/tsc.js --project /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/tsconfig.examples.json --noEmit false --outDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/.acceptance-artifacts --rootDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript` | None |
| P0.10 | `/Users/gqadonis/.cargo/bin/cargo build --locked --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/rust/Cargo.toml --example user_outcome_acceptance --message-format=json-render-diagnostics` | Named references in JSON |
| P0.11 | `/Users/gqadonis/.cargo/bin/cargo test --locked --features server-full,postgres-backend --test user_outcome_grpc --no-run --message-format=json-render-diagnostics` | Named references in JSON |
| P0.12-resolve-guest | `/Users/gqadonis/.cargo/bin/cargo generate-lockfile --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/Cargo.toml` | None |
| P0.13-build-guest | `/Users/gqadonis/.cargo/bin/cargo-component component build --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/Cargo.toml --locked --release --target wasm32-unknown-unknown --target-dir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002/classifier-guest/cargo-target` | Named references in JSON |

Only scoped P0.1 CI and fresh owned output paths differ from the old command recipe. All17stages remain serial, firstfailurestop, no automatic retry/repair. Build/guest stages run once inside the named artifact producer, never independently then again. No product test, service, provider or publication command is authorized.

A7 preparation bounds remain14400seconds total/7200stage. Carry prior execution1.350480196007993seconds, conservatively deduct2seconds: corrected attempt at most14398seconds, corrected P0.1 at most7198seconds. Deduct corrected materialization time again before setting remaining artifact-producer limits. This is not a fresh4h allowance. Global A7 time/calls retain all work and only prior signed idle exclusions.

Fresh output root: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-002`. Old001 scope/evidence is retained without deletion/adoption. Generated final private bindings use002 paths; source templates stay unchanged. No new directory has been created by this correction preparation.

Source inventory006 has the same29002regular-source-file identity as005: `sha256:7e2655915ee4ab9e40f4b566e9615988659ab2a930337bb6ff0cfda93fb81610`. All111scenario objects and A7budgets are copied unchanged. Installed artifacts, private/derived fixture and final V remain pending.
