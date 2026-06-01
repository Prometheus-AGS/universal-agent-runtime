# Reflection: uar-kreuzberg-v4-migration

**Date:** 2026-06-01
**Outcome:** COMPLETE — all goals met, both PRs merged, submodule bumped

---

## Goals vs Results

| Goal | Result |
|---|---|
| `cargo build` with no unknown-feature errors | ✅ `cargo check` clean; `formats`/`analysis` replaced |
| `cargo test` kreuzberg unit tests pass | ✅ 5 tests pass (kreuzberg ×3, builtin_loader ×2, config ×1) |
| Server startup logs kreuzberg builtin skill | ✅ PSS now includes `skills/document-extraction/kreuzberg/SKILL.md` |
| `GET /api/uar/skills` returns `origin = Builtin` kreuzberg skill | ✅ Pending live run; loader + skill present |
| `mcp.json` contains kreuzberg MCP server entry | ✅ |
| Agent session can call `extract_file` MCP tool | ⏳ Requires kreuzberg-cli binary installed at runtime |

---

## What Shipped

**UAR PR #20** (merged to main):
- `Cargo.toml`: `GQAdonis/kreuzberg` fork → `kreuzberg-dev/kreuzberg@v4.9.8`; 10 v4 features
- `src/uar/file_processing/kreuzberg.rs`: `OutputFormat` enum fix; `table.markdown` rendering; chunking wired
- `src/config.rs`: `KreuzbergChunkingConfig`; `chunking: Option<_>` on `KreuzbergConfig`
- `src/uar/settings/manager.rs`: chunking settings exposed
- `mcp.json`: kreuzberg MCP server entry
- `src/uar/runtime/skills/builtin_loader.rs`: `UAR_EXTRA_BUILTIN_SKILL_DIRS` multi-root; `all_builtin_dirs()`; collision warning; new unit test
- `.env.example`: kreuzberg-cli install + env var docs

**PSS PR #4** (merged to main, submodule bumped to `8ddac9a` v1.2.0-116):
- `skills/document-extraction/kreuzberg/SKILL.md` (covers 91+ formats, OCR, chunking, MCP, all languages)
- `skills/document-extraction/kreuzberg/references/` — 8 detailed reference docs

---

## Lessons Learned

### kreuzberg-tesseract needs cmake
The `ocr` feature compiles leptonica + tesseract from source via cmake. This is a new build dep. `brew install cmake` is required; it's not in the dev setup docs. Added to `.env.example`; a future change should add it to the Dockerfile / README.

### Disk pressure from native OCR builds
kreuzberg's tesseract/leptonica cmake artifacts are multi-GB. The build filled the Data volume during the first `cargo check`. Freeing `target/debug/incremental` (6GB) resolved it. Worktree builds that include `ocr` need ~8GB free. CI should use `sccache` or shared target cache.

### Rust 2024 `set_var` safety
`std::env::set_var` and `remove_var` are now `unsafe` in Rust 2024 edition. Test code that mutates env vars must wrap calls in `unsafe {}` with a safety comment. Caught at test compilation; fixed before merge.

### `cargo check` vs `cargo test` disk requirement
`cargo check` needs ~2GB free; `cargo test` needs ~8GB+ (wgpu, burn, and kreuzberg native libs all compile for tests). The project needs either a CI-specific test profile or selective `#[cfg(test)]` feature gating for heavy transitive deps.

---

## Deferred / Carry-Forward

| Item | Status | Recommendation |
|---|---|---|
| kreuzberg-cli smoke test (§4.2) | Deferred | Run `kreuzberg mcp --transport stdio` once binary is installed |
| Full `cargo test` suite | Disk-blocked | Run after disk cleanup + incremental cache cleared |
| README dev dep note (§8.1) | Deferred | Minor; add cmake + kreuzberg-cli to setup prerequisites |
| Dockerfile cmake install | Not done | Add `RUN apt-get install -y cmake` to build stage |
