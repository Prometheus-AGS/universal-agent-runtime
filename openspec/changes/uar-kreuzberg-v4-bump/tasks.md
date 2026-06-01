## 0. Bootstrap

- [x] 0.1 Create worktree: `scripts/worktree-new.sh uar-kreuzberg-v4-bump`
- [x] 0.2 `kreuzberg-cli` noted as runtime dep; `cmake` installed via Homebrew as transitive build dep

## 1. Cargo.toml — dependency re-point

- [x] 1.1 Replace fork entry with `kreuzberg-dev/kreuzberg@v4.9.8`
- [x] 1.2 Remove `features = ["formats", "analysis"]`
- [x] 1.3 Add features: `pdf`, `office`, `excel`, `email`, `html`, `archives`, `ocr`, `language-detection`, `chunking`, `mcp`
- [x] 1.4 `cargo fetch` — tag resolved, features accepted

## 2. File processing — API fixes

- [x] 2.1 Map `output_format` String → `kreuzberg::OutputFormat` enum in `build_extraction_config()` and `process_bytes()`
- [x] 2.2 Replace `{table:?}` with `table.markdown` in both `process()` and `process_bytes()`
- [x] 2.3 `cargo check` clean (zero errors)

## 3. Config — chunking field

- [x] 3.1 `KreuzbergChunkingConfig { max_characters: usize, overlap: usize }` added to `src/config.rs`
- [x] 3.2 `chunking: Option<KreuzbergChunkingConfig>` added to `KreuzbergConfig`
- [x] 3.3 Chunking wired in `build_extraction_config()` and `process_bytes()`
- [x] 3.4 `kreuzberg.chunking.max_characters` + `kreuzberg.chunking.overlap` exposed in settings manager

## 4. mcp.json — kreuzberg MCP server

- [x] 4.1 `kreuzberg` MCP server entry added to `mcp.json`
- [ ] 4.2 Smoke test — deferred to post-install of kreuzberg-cli binary

## 5. prometheus-skill-system — kreuzberg SKILL.md import (cross-repo)

- [x] 5.1 `skills/document-extraction/kreuzberg/` directory created
- [x] 5.2 `SKILL.md` copied from `kreuzberg-dev/kreuzberg@v4.9.8`
- [x] 5.3 All 8 reference docs copied into `references/`
- [x] 5.4 Internal links verified (relative `references/` paths intact)
- [x] 5.5 PR #4 opened and merged — https://github.com/Prometheus-AGS/prometheus-skill-system/pull/4
- [x] 5.6 UAR submodule bumped to `8ddac9a` (v1.2.0-116) — pushed to main

## 6. builtin_loader.rs — multi-root support

- [x] 6.1 `UAR_EXTRA_BUILTIN_SKILL_DIRS` env var read (colon-separated)
- [x] 6.2 `all_builtin_dirs()` returns primary + extra roots as `Vec<PathBuf>`
- [x] 6.3 Collision warning + last-wins via `HashMap<String, Skill>`
- [x] 6.4 Log: `discovered N skills from M roots`
- [x] 6.5 `discover_extra_root_loads_skill` unit test — passes

## 7. Build validation

- [x] 7.1 `cargo check` (SKIP_FRONTEND_BUILD=1) — zero errors
- [x] 7.2 5 unit tests pass: builtin_loader ×2, kreuzberg ×3, config kreuzberg test ×1
- [ ] 7.3 Full `cargo clippy` — blocked by disk pressure; `cargo check` clean is the gate

## 8. Docs

- [ ] 8.1 README dev deps — deferred (kreuzberg-cli documented in .env.example)
- [x] 8.2 `.env.example` — `UAR_EXTRA_BUILTIN_SKILL_DIRS`, `UAR_BUILTIN_SKILLS_DIR`, kreuzberg-cli install documented
- [ ] 8.3 CLAUDE.md kreuzberg note — assessed not needed; existing note is accurate
