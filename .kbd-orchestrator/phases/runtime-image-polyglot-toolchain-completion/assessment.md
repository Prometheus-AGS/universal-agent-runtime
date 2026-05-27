# Assessment — `runtime-image-polyglot-toolchain-completion`

**Phase status:** new (staged in parallel; active waypoint remains `use-optimistic-patch-helper-extraction`)
**Generated:** 2026-05-27
**Tool:** claude-code
**Replaces stale plan:** Prior worktree-only phase `runtime-image-multi-language-toolchain` (artifacts live under `/Users/gqadonis/.claude/worktrees/musing-sinoussi-09cea6/` on branch `claude/musing-sinoussi-09cea6`, never merged). Origin shipped a polyglot Dockerfile via independent work; this fresh assessment targets origin's actual state.

## 1. Goal restated

User wants a container image with: Rust nightly + cranelift/wasmtime + full polyglot WASM toolchain (compile-to-WASM for Rust, Go, Python, Node/TS6), Python 3.13 + pyo3 + uv, Node 24 + pnpm + bun + TS6, latest Go (1.26.3 per web research May 2026), and every notable WASM tool per language.

## 2. What origin already ships (NO work needed)

Inspecting `Dockerfile` (178 lines) and `wit/uar-skill.wit`:

| Item | Status at origin |
|---|---|
| Rust nightly toolchain | ✅ `ARG RUST_TOOLCHAIN=nightly` (floating, see gap §3) |
| `wasm32-{unknown-unknown, wasip1, wasip2}` targets | ✅ all three |
| `cargo-component`, `cargo-binstall` | ✅ |
| wasmtime CLI (JIT + Cranelift AOT) | ✅ v27.0.0 |
| AOT `.cwasm` precompile loop in build | ✅ (lines 129–135, no-op until skills ship `.wasm`) |
| Node.js 24 + npm + pnpm + bun | ✅ |
| Python 3.13 (deadsnakes PPA) | ✅ |
| `uv`, `maturin`, `pyo3-build-config` | ✅ |
| Go | ✅ but **1.23.4** (see gap §3) |
| Skill WIT contract | ✅ `wit/uar-skill.wit` — `uar:skill@0.1.0` with `run(input: string) -> result<string, string>` |
| Toolchains in runtime stage | ✅ deliberate (image stays polyglot at runtime, ~3 GB) |

That existing baseline already covers ~70% of the original goal. The previous worktree-phase plan was authored before this work landed and is now obsolete.

## 3. Remaining gaps vs the goal

| # | Gap | Severity | Notes |
|---|---|---|---|
| G1 | Rust toolchain is floating `nightly` (no date pin, no `rust-toolchain.toml`) | HIGH | Reproducibility risk — nightly drift can break builds overnight. |
| G2 | Go pinned at `1.23.4`; user asked for latest (1.26.3 per [Go release history](https://go.dev/doc/devel/release)) | HIGH | Three-version drift. |
| G3 | TinyGo not installed | HIGH | TinyGo ≥ 0.33 is the only path from Go to WASI P2 / Component Model. Without it, Go skills cannot target the `uar:skill` world. |
| G4 | JS/TS Component-Model toolchain missing: `@bytecodealliance/jco`, `@bytecodealliance/componentize-js`, `assemblyscript`, `javy` | HIGH | Required to author JS/TS skills against `wit/uar-skill.wit`. TypeScript 6 also not explicitly installed. |
| G5 | Python → Component: `componentize-py` not installed | HIGH | Required to author Python skills against `wit/uar-skill.wit`. |
| G6 | Multi-arch: `Dockerfile` hardcodes `linux-amd64` for both Go (line 82) and wasmtime (line 86) | MEDIUM | Blocks arm64 hosts (Apple Silicon CI, Graviton). |
| G7 | WASM low-level tooling missing: `wasm-tools`, `wit-bindgen-cli`, `wasm-bindgen-cli`, `cargo-wasi`, `twiggy`, `binaryen` (`wasm-opt`), `wabt` | MEDIUM | Needed for component compose, size optimization, debugging. |
| G8 | No secondary runtimes (WAMR `wamrc`/`iwasm`, WasmEdge `wasmedgec`) | LOW | Optional per assessment §3.1 — the existing wasmtime AOT path covers production. Useful for cross-runtime validation. |
| G9 | `Dockerfile.test` is on `rust:1.85` + `node:20` | MEDIUM | Drift vs the polyglot prod image (which uses nightly + Node 24). |
| G10 | No CI workflow publishing the polyglot image to a registry for reuse (devs/CI must rebuild ~30 min) | HIGH | Original change-6 deliverable. Registry of choice: GHCR is conventional; existing `deploy.yml` uses GCP Artifact Registry for the prod app, but the toolchain image should be public/pullable across forks → GHCR is the right home. |
| G11 | No Rust-side plugin loader strategy enum (`Jit`/`Aot`/`Interpreted`) or capability grant type | LOW | Existing `wit/uar-skill.wit` defines the guest export only; the host has no documented strategy/capability surface. The earlier draft (`plugin_loader.rs` in the deleted worktree) covered this. Can be brought forward as a separate change. |

## 4. Active waypoint conflict

`current-waypoint.json` is on `use-optimistic-patch-helper-extraction` (frontend work, totally unrelated). Do **not** overwrite that waypoint. This phase stages itself parallel and is meant to be picked up explicitly via `/kbd-execute <change-id>` against this phase dir.

## 5. Decisions baked in (consistent with prior plan defaults)

| # | Decision | Default |
|---|---|---|
| 1 | Augment existing `Dockerfile` in place (do NOT introduce a separate `Dockerfile.rust-dev`) | ✓ origin already merges dev + prod toolchains |
| 2 | Pin nightly to `nightly-2026-05-01` (matches earlier choice) | ✓ |
| 3 | Bump Go to `1.26.3` | ✓ latest stable |
| 4 | TinyGo `0.34.0` | ✓ |
| 5 | Multi-arch: `linux/amd64` + `linux/arm64` | ✓ |
| 6 | Registry: `ghcr.io/prometheus-ags/uar-toolchain` | ✓ |
| 7 | Secondary runtimes (WAMR, WasmEdge): SKIP for now — defer to follow-up phase | ✓ keeps blast radius small |
| 8 | Plugin-loader strategy enum (G11): DEFER to a follow-up KBD phase, port from worktree artifacts | ✓ |

## 6. Proposed change list (small, ordered)

Generated by §7 below.

## 7. Sources (web research from earlier in this conversation, still current)

- [Go 1.26.3 release history](https://go.dev/doc/devel/release)
- [TinyGo WASI P2 support (wasmCloud)](https://wasmcloud.com/blog/compile-go-directly-to-webassembly-components-with-tinygo-and-wasi-p2/)
- [componentize-py](https://github.com/bytecodealliance/componentize-py)
- [jco / ComponentizeJS](https://github.com/bytecodealliance/jco)
- [javy](https://github.com/bytecodealliance/javy)
- [Wasmtime AOT pre-compilation docs](https://docs.wasmtime.dev/examples-pre-compiling-wasm.html)
