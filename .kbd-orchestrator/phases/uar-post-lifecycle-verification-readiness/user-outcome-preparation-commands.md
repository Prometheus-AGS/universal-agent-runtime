# Exact P0 preparation commands

PROPOSED; CLOSED. These commands have NOT run. Full argv/cwd/environment references and metadata effects are in user-outcome-preparation-commands.json.

| Stage | Exact argv |
|---|---|
| P0.1 | `/Users/gqadonis/.volta/bin/pnpm install --frozen-lockfile --ignore-scripts` |
| P0.2 | `/Users/gqadonis/.volta/bin/npm --prefix /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript ci --ignore-scripts --no-audit --no-fund` |
| P0.3 | `/Users/gqadonis/.volta/bin/pnpm rebuild esbuild` |
| P0.4 | `/Users/gqadonis/.volta/bin/pnpm exec playwright install chromium` |
| P0.5-pull | `/usr/local/bin/docker pull pgvector/pgvector:pg17@sha256:ac08538c6f8b9904c33c8224c5e5706dbe760aca29db1d096972b4052c22a75d` |
| P0.5 | `/usr/local/bin/docker image inspect --format {{json .}} pgvector/pgvector:pg17@sha256:ac08538c6f8b9904c33c8224c5e5706dbe760aca29db1d096972b4052c22a75d` |
| P0.python-dependencies | `/Users/gqadonis/miniconda3/bin/python3 -m pip install --disable-pip-version-check --no-cache-dir --no-compile --no-warn-conflicts --no-deps --only-binary=:all: --require-hashes --target /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/python-site --report /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/python-dependencies-install.json -r /Users/gqadonis/Projects/prometheus/universal-agent-runtime/scripts/acceptance/fixtures/python-locked-requirements.txt` |
| P0.python-sdk | `/Users/gqadonis/miniconda3/bin/python3 -m pip install --disable-pip-version-check --no-cache-dir --no-compile --no-warn-conflicts --no-index --no-deps --no-build-isolation --target /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/python-site --report /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/python-sdk-install.json /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/python-sdk-source` |
| P0.6-pull | `/usr/local/bin/docker pull surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20` |
| P0.6 | `/usr/local/bin/docker image inspect --format {{json .}} surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20` |
| P0.7 | `/Users/gqadonis/.cargo/bin/cargo build --locked --bin universal-agent-runtime --features server-full,postgres-backend --message-format=json-render-diagnostics` |
| P0.8 | `/Users/gqadonis/.volta/bin/pnpm -C /Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend build` |
| P0.9 | `/Users/gqadonis/.volta/bin/node /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/node_modules/typescript/lib/tsc.js --project /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/tsconfig.examples.json --noEmit false --outDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript/.acceptance-artifacts --rootDir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/typescript` |
| P0.10 | `/Users/gqadonis/.cargo/bin/cargo build --locked --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/sdks/rust/Cargo.toml --example user_outcome_acceptance --message-format=json-render-diagnostics` |
| P0.11 | `/Users/gqadonis/.cargo/bin/cargo test --locked --features server-full,postgres-backend --test user_outcome_grpc --no-run --message-format=json-render-diagnostics` |
| P0.12-resolve-guest | `/Users/gqadonis/.cargo/bin/cargo generate-lockfile --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest/Cargo.toml` |
| P0.13-build-guest | `/Users/gqadonis/.cargo/bin/cargo-component component build --manifest-path /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest/Cargo.toml --locked --release --target wasm32-unknown-unknown --target-dir /Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest/cargo-target` |

Materialization stages (pnpm/npm/Python installs, browser downloads and two immutable image pulls/inspections) precede private configuration assembly. Build/guest resolution stages P0.7-P0.13 run ONCE inside prepare_artifacts.py, not first outside then again inside.

The private SDK copy prevents repo wheel/egg-info mutation. Python wheels are exactly17 lock-pinned packages with actual uv.lock hashes; backend/wheel metadata validation is explicitly included, no standalone pipcheck/import probe. SDK runtime uses immutable copied python-site.

Guest templates copy into the new private classifier-guest package. cargo generate-lockfile writes only its derived lock. cargo-component0.21.1 has explicit internal rustc --print sysroot / installed wasm32-wasip1 inspection, and bindings.format=false prevents rustfmt. RUSTUP_TOOLCHAIN is absent, so a missing target stops instead of implicit installation. No guest or model executes.

Existing packaged FastEmbed inputs need no model download. Final server models.models_dir points to their actual immutable snapshot, separate from hosted KB/Cohere embeddings.

Create exact new packet parent/private directories0700; preserve prior static output before frontend rewrite; use one build owner and isolated native/grpc/RustSDK targets. Source/locks unchanged, no retained fixtures adopted or erased.

Bounds:4h total/2hstage/64MiBstageoutput/1Mtreeentries/16GiBfile/128GiBselectedtrees. First failure stops; no retry, source fix, service/container/provider call, publication or unlisted probe.

Actual hashes/resource paths/private artifact config are populated from materialization under this source-bound recipe; they are not fabricated now. P0 succeeds only on real stage/snapshot receipts. Final V remains CLOSED until actual derived inputs/inventory and renewed explicit instruction. This is a requested AGENTS.md exception because build-produced inputs do not yet exist.
