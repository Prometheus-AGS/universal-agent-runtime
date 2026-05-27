## 1. Toolchain stage

- [x] 1.1 Ubuntu 24.04 base + apt deps.
- [x] 1.2 Rust nightly via rustup; wasm32-wasip2/wasip1/unknown targets; cargo-component + cargo-binstall.
- [x] 1.3 Node 24 via NodeSource; corepack pnpm; bun via installer.
- [x] 1.4 Python 3.13 via deadsnakes; uv + maturin + pyo3-build-config.
- [x] 1.5 Go (latest stable, currently 1.23.4) via tarball.
- [x] 1.6 wasmtime CLI (27.0.0).
- [x] 1.7 Smoke probe runs all toolchains in the build.

## 2. Builder stage

- [x] 2.1 FROM toolchain AS builder.
- [x] 2.2 `git submodule update --init --recursive --depth 1` (best-effort).
- [x] 2.3 `pnpm install --frozen-lockfile` + `pnpm --filter ./frontend build`.
- [x] 2.4 `cargo +nightly build --release --features "memory-palace,wasm-runtime,surreal-memory/embedded"`.
- [x] 2.5 AOT-precompile loop for any shipped `skill.wasm` (no-op until skills ship binaries).
- [x] 2.6 Stage outputs to `/out`.

## 3. Runtime stage

- [x] 3.1 FROM toolchain AS runtime (keeps all tools resident).
- [x] 3.2 COPY binary, static, skill manifests, AOT artifacts, models.
- [x] 3.3 VOLUME declarations for skills-derived/user, HF cache, cargo cache, pnpm store, data.
- [x] 3.4 ENV overrides honour the static/models/skills lookup helpers.
- [x] 3.5 WORKDIR /opt/uar + CMD universal-agent-runtime.

## 4. Compose

- [ ] 4.1 docker-compose.prod.yaml volumes — deferred (no .yaml in current repo to update beyond surreal-memory-server's).

## 5. CI

- [ ] 5.1 GH Actions `docker build` step — deferred to integration-tests-and-docs.
- [ ] 5.2 Toolchain smoke step — deferred.

## 6. Docs

- [ ] 6.1 README building-skills-inside-container section — deferred.
