# Positive verification — `resolve-sdk-distribution`

Date: 2026-08-18

## Rust SDK default profile — macOS

Command:

```bash
cd sdks/rust
RUSTC_WRAPPER= cargo test --locked
RUSTC_WRAPPER= cargo check --locked --examples
RUSTC_WRAPPER= cargo doc --locked --no-deps
cargo metadata --locked --no-deps --format-version 1
cargo package --locked --allow-dirty --list
```

Observed output:

```text
test result: ok. 3 passed; 0 failed
test result: ok. 0 passed; 0 failed
test result: ok. 1 passed; 0 failed
Finished `dev` profile ... target(s) in 1m 18s
Generated .../target/doc/universal_agent_runtime_sdk/index.html
name: universal-agent-runtime-sdk
version: 1.0.0
authors: [Prometheus AGS]
license: MIT
runtime dependency requirement: =1.0.0
package list: LICENSE-MIT, README.md, 12 examples, 5 source files, and the embedded integration test
```

The standalone SDK lockfile was stale relative to the current root path
dependency. Cargo's targeted `cargo update --offline --manifest-path
sdks/rust/Cargo.toml -p universal-agent-runtime --precise 1.0.0` reconciled that
runtime graph, including dependencies already selected by the root manifest;
the locked SDK test above compiled the resolved graph. No SDK manifest
dependency was added beyond the existing runtime dependency's registry version.

The runtime compiled as the SDK's test-only development dependency with five
warnings in that SDK profile. No warning-free claim is made. `cargo fmt
--manifest-path sdks/rust/Cargo.toml --check` also exposed pre-existing source
formatting differences; this metadata-only change did not reformat adjacent SDK
source.

## Python SDK — CPython 3.13

Command from `sdks/python`:

```bash
uv run --locked --extra dev pytest tests
uv run --locked --extra dev ruff check .
uv run --locked --extra dev mypy src
uv run --locked --extra dev python -m build .
uv run --locked --extra dev sphinx-build -W -b html docs docs/_build/html
```

Observed output:

```text
4 passed
All checks passed!
Success: no issues found in 3 source files
Successfully built universal_agent_runtime_sdk-1.0.0.tar.gz and universal_agent_runtime_sdk-1.0.0-py3-none-any.whl
build succeeded.
```

Wheel inspection observed 8 files, distribution metadata, and the MIT license.

## TypeScript SDK — Node.js

Command from `sdks/typescript`:

```bash
npm ci
npm test
npm run lint
npm run typecheck
npm run build
npm run docs
npm pack --dry-run --json
```

Observed output:

```text
Test Files  1 passed (1)
Tests  4 passed (4)
lint: exit 0
typecheck: exit 0
CJS, ESM, and DTS builds: exit 0
TypeDoc: exit 0 (one examples-directory copy warning)
package: @prometheus-ags/universal-agent-runtime-sdk@1.0.0
unpacked size: 45283 bytes
```

## Package names and streaming examples

Command:

```bash
rg -n 'universal-agent-runtime-sdk|@prometheus-ags/universal-agent-runtime-sdk' website/docs/sdk-* sdks/*/README.md
rg -n 'stream|streaming' sdks/rust/examples sdks/python/examples sdks/typescript/examples website/docs/sdk-* -g '*.*'
```

Observed: the Rust install name is `universal-agent-runtime-sdk`, Python is
`universal-agent-runtime-sdk`, and TypeScript is
`@prometheus-ags/universal-agent-runtime-sdk` in both SDK README and website
quickstarts. Each language has a streaming-chat example; Rust and TypeScript
also expose a streaming run example, and Python exposes `stream_run`.

## Repository gates

Commands and observed output:

```text
$ openspec validate resolve-sdk-distribution --strict
Change 'resolve-sdk-distribution' is valid

$ git diff --check -- .github/workflows/ci.yml openspec/changes/resolve-sdk-distribution sdks/rust/Cargo.toml sdks/rust/Cargo.lock sdks/typescript/package.json website/docs/sdk-rust/intro.md website/docs/sdk-python/intro.md website/docs/sdk-typescript/intro.md
[exit 0; no output]

$ RUSTC_WRAPPER= cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
Finished `dev` profile ... target(s) in 1.35s
universal-agent-runtime (lib) generated 3 warnings

$ RUSTC_WRAPPER= cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
Finished `dev` profile ... target(s) in 0.66s
universal-agent-runtime (lib) generated 571 warnings
```

The Clippy warnings are the repository's existing pedantic baseline; the SDK
metadata/docs/workflow change introduced no Rust source warning.

## Final candidate hashes

```text
0f421b27a9b4a17d375c423923801c30e1bd3797953cc5b4a4ade0a1bdddb074  sdks/rust/Cargo.toml
fe0f02c9c4da0ab5d02e6eb0eabf1cf6c92bba0d1f46307d9120f3936c47c7e8  sdks/rust/Cargo.lock
c8571f9cf1e3c9876d100361d060b11d250bc3adb115254e4331458879d230d8  sdks/typescript/package.json
64b5fa39339ef6383c72cee8501bff686b4c278293836498a68f77d9f8f900cf  website/docs/sdk-rust/intro.md
26672ff24c1e025a1f961e14d89e3e9ec88f90fce772516ba6f6d8e20baceaa8  website/docs/sdk-python/intro.md
5934c11b8c767276491cb206d41ebba2607f30f8be2c0dda0cc2e2b7ad3081de  website/docs/sdk-typescript/intro.md
[retired]  .github/workflows/ci.yml
```
