# Plan — `runtime-image-polyglot-toolchain-completion`

**Backend:** OpenSpec (proposals to scaffold via `/opsx:new` when ready)
**Generated:** 2026-05-27
**Source assessment:** [`assessment.md`](assessment.md)
**Active waypoint:** NOT touched — `use-optimistic-patch-helper-extraction` stays active. Pick changes from this phase explicitly.

## Ordered change list

| # | Change id | Touches | Agent | Addresses |
|---|---|---|---|---|
| 1 | `pin-rust-nightly-and-bump-go` | `Dockerfile`, new `rust-toolchain.toml`, `AGENTS.md`/`CLAUDE.md` | claude-code | G1, G2 |
| 2 | `add-missing-wasm-toolchains-to-image` | `Dockerfile` (add TinyGo, jco, componentize-js, assemblyscript, javy, componentize-py, ts6, wasm-tools, wit-bindgen-cli, wasm-bindgen-cli, twiggy, binaryen, wabt) | claude-code | G3, G4, G5, G7 |
| 3 | `dockerfile-multi-arch-toolchain` | `Dockerfile` (`TARGETARCH` branching for Go + wasmtime + tinygo + javy) | claude-code | G6 |
| 4 | `dockerfile-test-rebase-to-polyglot` | `Dockerfile.test` (rust 1.85→nightly, node 20→24, optional `FROM uar-toolchain:<tag>`) | claude-code | G9 |
| 5 | `ci-publish-uar-toolchain-image` | new `.github/workflows/image-uar-toolchain.yml` | claude-code | G10 |
| 6 | `plugin-loader-strategy-enum` (port from deleted worktree) | new `src/uar/runtime/wasm/plugin_loader.rs`, update `mod.rs`, OpenSpec change | codex or claude-code | G11 |

## Dependencies

```
1 pin-rust-nightly-and-bump-go
  └── 2 add-missing-wasm-toolchains-to-image
        └── 3 dockerfile-multi-arch-toolchain
              ├── 4 dockerfile-test-rebase-to-polyglot
              └── 5 ci-publish-uar-toolchain-image   ← deliverable user asked for
6 plugin-loader-strategy-enum  (independent)
```

## What we ship right now in this execute run

Per user direction (option 2 — focus on what's actually missing), this `/kbd-execute` invocation produces **only change 5** (`ci-publish-uar-toolchain-image`) as the immediate deliverable, because:

- The CI workflow is purely additive (no Dockerfile edits, no conflicts).
- Changes 1–4 are Dockerfile mutations that should be reviewed as a focused batch in a fresh session.
- Change 6 needs language-design review before instantiation.

The workflow we ship will publish whatever the current `Dockerfile` produces — once changes 1–4 land, the same workflow picks them up automatically on the next push.

## Default registry / tagging

- Registry: `ghcr.io/prometheus-ags/uar-toolchain`
- Tags: `:latest`, `:sha-<short>`, `:nightly-<YYYYMMDD>`
- Multi-arch: `linux/amd64` only at first push (change 3 unlocks arm64; workflow will already be authored to opt in once arm64 builds succeed).
- Cache: GitHub Actions cache (`type=gha,mode=max`) — typical 30 min cold build, ~5 min warm.
- Auth: `GITHUB_TOKEN` is sufficient for GHCR publishes from the repo's own org.
