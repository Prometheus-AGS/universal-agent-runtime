## Why

UAR's production image must double as a **polyglot build host** so that skills, plugins, and user-supplied artifacts can be compiled in any of the languages UAR itself supports. It must also vendor both new submodules (`prometheus-entity-management`, `prometheus-skill-system`) at build time and provide writable volumes for runtime-generated artifacts.

## What Changes

### Required toolchain present in the runtime image (not just the builder stage)

- **Rust nightly** — `rustup toolchain install nightly` set as default; `rustfmt`, `clippy`, `cargo`, plus `rustup target add wasm32-wasip2` and `wasm32-wasip1` for WASM component authoring.
- **Node.js LTS ≥ 24** with all three package managers: `npm` (bundled), `pnpm` (corepack-enabled), `bun` (installed from official tarball).
- **Python 3.13** with `uv` (Astral) for env/dep management and `maturin` + `pyo3` deps preinstalled so users can build Rust↔Python extensions.
- **Go (latest stable)** — `golang:latest` toolchain copied in for users compiling Go skills/plugins.
- **WASM toolchain**: `wasmtime` CLI installed, plus the `cranelift` crate available via Rust nightly for **AOT compilation** of WASM components. UAR itself uses wasmtime 41 (already a Cargo dep under the `wasm-runtime` feature) for **JIT** at runtime; the in-image AOT path lets us precompile the builtin WASM skills during image build for faster cold start.

### Multi-stage layout

1. **`toolchain` stage** (`debian:bookworm-slim` or `ubuntu:24.04` base): install all five toolchains. This stage is cache-friendly (rarely changes).
2. **`builder` stage** (FROM toolchain):
   - Copy repo + run `git submodule update --init --recursive` (or use `--mount=type=bind` for the working tree).
   - `pnpm install --frozen-lockfile && pnpm --filter ./frontend build` (after the pnpm migration change ships).
   - `cargo +nightly build --release --features "memory-palace,wasm-runtime,surreal-memory/embedded"` (Linux build drops the `metal` feature).
   - AOT-precompile builtin WASM skills: `wasmtime compile crates/prometheus-skill-system/skills/<…>/skill.wasm -o /opt/uar/skills/wasm-builtin/<name>.cwasm` for any component that ships.
3. **`runtime` stage** (FROM toolchain — keeps all build tools resident so the running container can build user-supplied skills):
   - Copy UAR binary, `static/`, skill manifests, AOT-precompiled `.cwasm` artifacts.
   - Declared volumes:
     - `/var/lib/uar/skills-derived/` — writable, derivative artifacts.
     - `/var/lib/uar/skills-user/` — user-installed skills (`.wasm`, `SKILL.md`).
     - `/var/lib/uar/cache/huggingface/` — HF model cache.
     - `/var/lib/uar/cache/cargo/` — Cargo registry/build cache for in-container builds.
     - `/var/lib/uar/cache/pnpm/` — pnpm store.
     - `/var/lib/uar/data/` — SurrealKV (only when running embedded; production typically points at the remote container).
   - Default `WORKDIR /opt/uar`, `CMD ["universal-agent-runtime"]`.

### Compose

- Update `docker-compose.prod.yaml` with named volumes for each of the above.
- Bind-mount option documented in `docker-compose.dev.yaml`.

## Acceptance

- `docker build .` succeeds with all toolchains present (`docker run … bash -c "rustc --version && node --version && pnpm --version && bun --version && python3 --version && uv --version && go version && wasmtime --version"` lists matching versions).
- Container starts UAR successfully and serves the SPA.
- AOT-precompiled `.cwasm` artifacts load faster than fresh JIT in a measurable smoke test.
- `pnpm` / `cargo` build of a sample user skill inside the running container completes without installing additional system packages.
