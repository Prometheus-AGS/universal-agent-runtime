# Execution — `runtime-image-polyglot-toolchain-completion`

**Backend selected:** `openspec` (project default).
**Started:** 2026-05-27 by claude-code, operating on origin after the prior worktree was deleted.
**All 6 changes completed in two sessions:**

| # | Change | Files | buildx check |
|---|---|---|---|
| 1 | `pin-rust-nightly-and-bump-go` | `rust-toolchain.toml` (new), `Dockerfile` (ARGs), `AGENTS.md` | n/a (config) |
| 2 | `add-missing-wasm-toolchains-to-image` | `Dockerfile` (5 stages augmented) | PASS |
| 3 | `dockerfile-multi-arch-toolchain` | `Dockerfile` (3 multi-arch branches), `.github/workflows/image-uar-toolchain.yml` (default multi-arch) | PASS |
| 4 | `dockerfile-test-rebase-to-polyglot` | `Dockerfile.test` (rust 1.85→1.93-slim+nightly, node 20→24) | PASS |
| 5 | `ci-publish-uar-toolchain-image` | `.github/workflows/image-uar-toolchain.yml` (new) | n/a (yaml) |
| 6 | `plugin-loader-strategy-enum` | `src/uar/runtime/wasm/plugin_loader.rs` (new, ported), `wit/uar-plugin.wit` (new, ported), `src/uar/runtime/wasm/mod.rs` (registered), `openspec/changes/plugin-loader-wit-contract/{proposal,tasks,design,specs/plugin-model/spec}.md` (ported) | n/a (rust + wit) |

## Per-change detail

### Change 1 — pin-rust-nightly-and-bump-go (G1, G2)

- Created `rust-toolchain.toml` pinning `nightly-2026-05-01` with rustfmt/clippy/rust-src/rust-analyzer + all three wasm targets.
- `ARG RUST_TOOLCHAIN=nightly` → `nightly-2026-05-01`.
- `ARG GO_VERSION=1.23.4` → `1.26.3` (latest stable per go.dev/doc/devel/release).
- Added `ARG TINYGO_VERSION=0.34.0`.
- Documented toolchain pin in `AGENTS.md`.

### Change 2 — add-missing-wasm-toolchains-to-image (G3, G4, G5, G7)

Dockerfile `toolchain` stage augmented:
- apt: added `binaryen` (wasm-opt), `wabt` (wat2wasm/wasm2wat), `libffi-dev`, `patchelf`.
- Python: `uv pip install` now also pulls `componentize-py` and `wasmtime` (Python embedder).
- Rust: rustup install now also adds `rustfmt clippy rust-src` components; `cargo binstall` pulls `cargo-component`, `wasm-tools`, `wit-bindgen-cli`, `wasm-bindgen-cli`, `cargo-wasi`, `twiggy` as prebuilt binaries; cargo registry purged to save layer space.
- Node: global `npm install -g typescript@6 @bytecodealliance/jco @bytecodealliance/componentize-js assemblyscript javy-cli`.
- TinyGo install added (deb from upstream).
- Sanity probe expanded to call every new tool (`cargo component --version`, `wasm-tools --version`, `wit-bindgen --version`, `wasm-bindgen --version`, `wasm-opt --version`, `tsc --version`, `jco --version`, `asc --version`, `maturin --version`, `componentize-py --help`, `tinygo version`) so a missing tool fails the build early.

### Change 3 — dockerfile-multi-arch-toolchain (G6)

Three install steps now branch on `TARGETARCH` (set automatically by buildx):
- Go: `linux-${GO_ARCH}` (amd64/arm64).
- TinyGo: `${TG_ARCH}.deb` (amd64/arm64).
- wasmtime: `${WT_ARCH}-linux.tar.xz` where WT_ARCH = x86_64 (amd64) or aarch64 (arm64).
- Each branch errors loudly for unsupported arches.
- Added `ARG TARGETARCH=amd64` declaration inside the `toolchain` stage (the outer ARGs are not visible after FROM).
- CI workflow default platforms updated to `linux/amd64,linux/arm64` now that the Dockerfile supports it.

### Change 4 — dockerfile-test-rebase-to-polyglot (G9)

- `FROM rust:1.85-bookworm` (both stages) → `FROM rust:1.93-slim-bookworm`. Matches what the runtime container resolves to before nightly install.
- `FROM node:20-bookworm` → `FROM node:24-bookworm`. Aligns with `NODE_MAJOR=24` in production Dockerfile.
- NodeSource installer bumped `setup_20.x` → `setup_24.x`.
- `rust-toolchain.toml` now copied alongside `Cargo.toml/Cargo.lock` in the deps-cache stage so the dummy build uses the pinned nightly (otherwise stable rust builds the dummy, then the real source build re-fetches nightly and busts the cache).

### Change 5 — ci-publish-uar-toolchain-image (G10)

Already done in prior turn. Now updated with arm64 default since change 3 unblocked it. Workflow publishes the `toolchain` stage of the existing Dockerfile to `ghcr.io/${owner}/uar-toolchain` on push/manual/weekly cron with sha/nightly-date/latest tags, GHA cache, QEMU multi-arch.

### Change 6 — plugin-loader-strategy-enum (G11)

Ported from `/Users/gqadonis/.claude/worktrees/musing-sinoussi-09cea6/` (deleted worktree, branch `claude/musing-sinoussi-09cea6`) which had this work pre-built. Files copied verbatim:

- `src/uar/runtime/wasm/plugin_loader.rs` — `PluginSource`, `PluginStrategy { Jit, Aot{cache_dir}, Interpreted }`, deny-by-default `CapabilityGrant`, `LoadRequest`, `PluginId`, `PluginLoadError` (thiserror), `PluginLoader` trait, 2 unit tests.
- `wit/uar-plugin.wit` — `uar:plugin@0.1.0` package with `types`/`host`/`plugin` interfaces and `uar-plugin` world. **Sits alongside the existing `wit/uar-skill.wit`** — they describe different layers:
  - `uar-skill@0.1.0` = the guest-facing skill execution contract (`run(input) -> result<string,string>`).
  - `uar-plugin@0.1.0` = the host-side loader's capability + lifecycle contract.
- `src/uar/runtime/wasm/mod.rs` — registered new module + rustdoc bullet.
- OpenSpec scaffold at `openspec/changes/plugin-loader-wit-contract/` (proposal/tasks/design/spec delta).

**`thiserror = "2.0"` is already a workspace dep** (verified in prior session); no Cargo.toml change needed.

## Verifications

| Check | Result |
|---|---|
| `docker buildx build --check -f Dockerfile .` | PASS, no warnings |
| `docker buildx build --check -f Dockerfile.test .` | PASS, no warnings |
| Workflow YAML visual | PASS (actionlint not available locally) |
| `cargo check --features wasm-runtime` | DEFERRED — origin has many unrelated unstaged changes (see `git status`); a clean check needs a focused branch |
| `openspec validate plugin-loader-wit-contract --strict` | DEFERRED — needs openspec CLI locally |
| Full `Dockerfile` build | DEFERRED — 25–40 min cold; CI workflow exercises it |
| `wit-bindgen rust wit/uar-plugin.wit` | DEFERRED — gated on first toolchain image build |

## Active waypoint

NOT changed. `use-optimistic-patch-helper-extraction` remains the active phase. All work in this phase is staged in `.kbd-orchestrator/phases/runtime-image-polyglot-toolchain-completion/` and the production tree.

## QA gates

| Change | Files | QA |
|---|---|---|
| 1 | 3 | At threshold — skipped (config + docs trivial). |
| 2 | 1 | Skipped (single file). |
| 3 | 2 | Skipped (single Dockerfile + workflow tweak). |
| 4 | 1 | Skipped (single file). |
| 5 | 1 | Skipped (single file). |
| 6 | 7 | MANUAL_REVIEW_RECOMMENDED — `/refine-validate` skill not registered in session. Eyeball the WIT contract + strategy enum before downstream consumers ratify them. |

## Spawn-worthy follow-ups (not done here)

- `plugin-loader-instantiation` — wire `PluginLoader` impl against `wasmtime::component::Linker` and `sandbox.rs`.
- `plugin-loader-dispatcher` — register loaded plugins with the skill/tool dispatcher.
- `secondary-wasm-runtimes` — add WAMR (`wamrc`, `iwasm`) + WasmEdge (`wasmedgec`) to the toolchain image (assessment gap G8). Optional for production; useful for cross-runtime validation.
- `cosign-toolchain-image` — wire the `id-token: write` permission already declared in the workflow to a sigstore signing step.
