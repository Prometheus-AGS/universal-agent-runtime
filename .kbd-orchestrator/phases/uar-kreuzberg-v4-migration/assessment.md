# Assessment: kreuzberg v4 Migration + Skills Auto-Loading

**Phase:** `uar-kreuzberg-v4-migration`
**Date:** 2026-06-01
**Assessed by:** kbd-assess

---

## Executive Summary

The `kreuzberg` document intelligence library has undergone a major version jump (unknown fork version → **v4.9.8**) and organisational move (`GQAdonis/kreuzberg` fork → `kreuzberg-dev/kreuzberg` upstream). The fork has no published releases and was last touched 2026-04-26; the upstream is actively maintained with an LTS patch released 2026-05-17.

Beyond the dependency URL fix, v4 introduces **three strategic opportunities** this runtime should exploit:

1. **MCP server mode** — kreuzberg can now run as `kreuzberg mcp --transport stdio`, exposing 11+ document extraction tools directly to any agent in the runtime.
2. **Skills auto-loading** — upstream ships `skills/kreuzberg/SKILL.md` (compatible with our `builtin_loader.rs` format) covering the full Rust/Python/Node.js/CLI API — a zero-effort system skill if we extend the loader to accept multiple skill roots.
3. **Feature flag restructuring** — the fork's bespoke `formats` and `analysis` flags are gone; v4 uses granular opt-in features that unlock chunking, embeddings, code intelligence, and language detection with no extra dependencies.

---

## 1. Current State (Gaps)

### 1.1 Cargo.toml dependency — BROKEN link

```toml
# current (Cargo.toml:229)
kreuzberg = {
    git   = "https://github.com/GQAdonis/kreuzberg.git",
    branch = "main",
    default-features = false,
    features = ["tokio-runtime", "formats", "analysis"]
}
```

**Problems:**
| Issue | Impact |
|---|---|
| Fork URL (`GQAdonis/kreuzberg`) diverges from upstream | Misses all v4 bug fixes, format additions, MCP module |
| `features = ["formats"]` — feature does not exist in v4 | Build fails against upstream |
| `features = ["analysis"]` — feature does not exist in v4 | Build fails against upstream |
| No version pin — tracks `branch = "main"` | Non-deterministic builds |

**Required change:**
```toml
kreuzberg = {
    git = "https://github.com/kreuzberg-dev/kreuzberg.git",
    tag  = "v4.9.8",                          # pin to LTS release
    default-features = false,
    features = [
        "tokio-runtime",   # extract_file_sync, batch_extract_file_sync
        "pdf",             # replaces "formats" (PDF via pdf_oxide)
        "office",          # replaces "formats" (docx, odt, pptx, xlsx, etc.)
        "excel",           # xlsx/xlsm/xls — requires tokio-runtime
        "email",           # .eml, .msg, .pst
        "html",            # HTML-to-markdown, pulled in by pdf+ocr anyway
        "archives",        # zip, tar, 7z
        "ocr",             # replaces "analysis" (Tesseract backend)
        "language-detection",  # replaces "analysis" (whatlang)
        "chunking",        # NEW — text chunking for RAG
        "mcp",             # NEW — MCP server embedded in the crate
    ]
}
```

> **Note:** kreuzberg v4 is not yet published to crates.io under a stable `version = "4"` semver tag as of the assessment date. Use `git + tag` until the registry publication lands.

---

### 1.2 Rust API breaking changes in `src/uar/file_processing/kreuzberg.rs`

#### 1.2.1 `OutputFormat` — now an enum, not a string

```rust
// OLD — string field on ExtractionConfig (via KreuzbergConfig.output_format)
config.output_format = "markdown".to_string();   // ← does not compile against v4

// NEW — enum
use kreuzberg::OutputFormat;
config.output_format = OutputFormat::Markdown;   // ← v4 API
```

**Affected sites:**
- `src/uar/file_processing/kreuzberg.rs` — `build_extraction_config()` does not set `output_format` (defaults to `OutputFormat::Plain`). Config wiring must be updated to map `KreuzbergConfig.output_format: String` → `OutputFormat` enum.
- `src/config.rs` — `KreuzbergConfig.output_format: String` can stay as the config surface; conversion happens at `build_extraction_config()`.

#### 1.2.2 `OcrConfig.backend` — now a typed enum/string-newtype

The v4 `OcrConfig` accepts `backend: "tesseract"` as a string ref still (confirmed via SKILL.md pitfalls section — `"tesseract"` is the canonical string). The `.into()` conversion is available. **No change required at call sites.**

#### 1.2.3 `ExtractionConfig` third-argument signature

```rust
// OLD (fork): extract_file_sync(path, mime_type, &config) → could accept owned config
// NEW (v4):   extract_file_sync(path, mime_type, &config) → &ExtractionConfig reference required

// Current UAR code is ALREADY CORRECT — passes &extraction_config
kreuzberg::extract_file_sync(path_buf, None, &extraction_config)
```

No change needed.

#### 1.2.4 `extract_bytes_sync` signature — unchanged

Current usage in `process_bytes` is correct.

#### 1.2.5 `result.tables` — now has richer shape

```rust
// OLD (fork): result.tables was Vec<kreuzberg::Table> with opaque Debug repr
// NEW (v4):   each table has .markdown: String and .cells: Vec<Vec<String>>
```

The current code does `writeln!(content, "{table:?}\n")` — this still compiles but produces ugly debug output. **Functional, but should be upgraded** to use `table.markdown` for clean rendering.

#### 1.2.6 `result.images` — `img.data` type unchanged

`img.data.to_vec()` still works (data is `Bytes` which derefs to `[u8]`). No change required.

#### 1.2.7 `result.metadata` — fields still present

`title`, `subject`, `authors`, `keywords`, `language`, `pages` all confirmed present in v4 types. No change required.

---

### 1.3 `mcp.json` — kreuzberg MCP server not registered

kreuzberg v4 ships `kreuzberg mcp --transport stdio` via `kreuzberg-cli`. Every agent in the runtime could call `extract_file`, `extract_bytes`, `batch_extract_files`, `detect_mime_type`, `chunk_text`, `embed_text`, and `extract_structured` as first-class MCP tools — currently unavailable.

**Required addition to `mcp.json`:**
```json
"kreuzberg": {
  "command": "kreuzberg",
  "args": ["mcp", "--transport", "stdio"]
}
```

This requires `kreuzberg-cli` to be installed (`cargo install kreuzberg-cli` or via Homebrew `brew tap kreuzberg-dev/tap && brew install kreuzberg-cli`).

---

### 1.4 Skills — kreuzberg SKILL.md not loaded as a system skill

`kreuzberg-dev/kreuzberg` ships `skills/kreuzberg/SKILL.md` with:
- Full Rust/Python/Node.js/CLI API coverage (91+ formats, all config options)
- Pitfall section preventing the most common mistakes
- References to 8 detailed sub-documents (Rust API, CLI reference, configuration, supported formats, advanced features)

The UAR's `builtin_loader.rs` currently only scans `crates/prometheus-skill-system/skills` (or `UAR_BUILTIN_SKILLS_DIR`). It does **not** scan any kreuzberg-internal path.

**Gap:** Agents have no system skill instructing them how to use kreuzberg correctly, leading to hallucinated API calls.

**Proposed fix — multi-root builtin loader:**
Extend `builtin_loader.rs` to accept `UAR_EXTRA_BUILTIN_SKILL_DIRS` (colon-separated) in addition to the primary root. At startup, scan all roots. Example:

```
UAR_EXTRA_BUILTIN_SKILL_DIRS=crates/kreuzberg/skills
```

Since `kreuzberg` is a git dependency, its source tree is available at `~/.cargo/git/checkouts/kreuzberg-*/skills/` — the path is resolvable via `cargo metadata`.

**Alternative (simpler, no env var needed):** Copy `skills/kreuzberg/SKILL.md` + its `references/` into `crates/prometheus-skill-system/skills/document-extraction/kreuzberg/` as part of the dependency bump. The prometheus-skill-system already vends third-party skill bundles.

**Recommended approach:** Both — ship the skill in prometheus-skill-system for reliability, AND add `UAR_EXTRA_BUILTIN_SKILL_DIRS` for extensibility (so future library upgrades auto-surface new skill revisions).

---

## 2. New Capabilities — What to Unlock

| Feature | Status | Action needed |
|---|---|---|
| MCP server (`kreuzberg mcp`) | Not used | Add to `mcp.json`; install `kreuzberg-cli` |
| CLI (`kreuzberg extract`) | Not used | No code change; available after CLI install |
| Chunking (`ChunkingConfig`) | Not exposed | Add `chunking` feature; wire `KreuzbergConfig.chunking` field |
| Code intelligence (tree-sitter) | Not used | Add `tree-sitter` feature; new mime types |
| Language detection | Feature missing | Add `language-detection` feature to Cargo.toml (replaces `analysis`) |
| Embeddings | Not used | Add `embedding-presets` or `embeddings` feature |
| Structured extraction via LLM | Not used | Requires `liter-llm` feature + `extract_structured` MCP tool |
| 91+ formats (was 56) | Partial | Fix feature flags → unlocks new formats automatically |
| Table markdown rendering | Degraded | Use `table.markdown` instead of `{:?}` debug format |
| kreuzberg SKILL.md as system skill | Missing | Copy to skill-system OR extend builtin loader |

---

## 3. Work Items (Priority Order)

### P0 — Build fix (blocking)

| ID | Task | File(s) |
|---|---|---|
| K-1 | Update `kreuzberg` git URL → `kreuzberg-dev/kreuzberg`, pin to `v4.9.8` | `Cargo.toml` |
| K-2 | Replace `features = ["formats", "analysis"]` with v4 feature set | `Cargo.toml` |
| K-3 | Fix `OutputFormat` string → enum in `build_extraction_config()` | `src/uar/file_processing/kreuzberg.rs` |

### P1 — Quality / correctness

| ID | Task | File(s) |
|---|---|---|
| K-4 | Render tables via `table.markdown` instead of `{:?}` | `src/uar/file_processing/kreuzberg.rs` |
| K-5 | Expose `ChunkingConfig` in `KreuzbergConfig` + wire to `ExtractionConfig` | `src/config.rs`, `kreuzberg.rs`, `settings/manager.rs` |
| K-6 | Confirm `cargo build` + `cargo test` pass with new dependency | CI |

### P2 — MCP integration (strategic)

| ID | Task | File(s) |
|---|---|---|
| K-7 | Add `kreuzberg` MCP server entry to `mcp.json` | `mcp.json` |
| K-8 | Add `kreuzberg-cli` install step to dev setup docs / Dockerfile | `README.md`, `docker/` |

### P3 — Skills auto-loading (strategic)

| ID | Task | File(s) |
|---|---|---|
| K-9 | Copy `skills/kreuzberg/SKILL.md` + `references/` into `crates/prometheus-skill-system/skills/document-extraction/kreuzberg/` | submodule PR |
| K-10 | Extend `builtin_loader.rs` to accept `UAR_EXTRA_BUILTIN_SKILL_DIRS` colon-list | `src/uar/runtime/skills/builtin_loader.rs` |
| K-11 | Update `UAR_BUILTIN_SKILLS_DIR` documentation in `CLAUDE.md` / `.env.example` | docs |

---

## 4. Risk & Notes

- **git dependency latency**: Until kreuzberg publishes `version = "4"` to crates.io, the `git + tag` approach is correct. Pin to `v4.9.8` (LTS) not `branch = "main"` to avoid silent breakage on future upstream commits.
- **kreuzberg-cli separate binary**: The MCP server and CLI require `kreuzberg-cli` as a binary in PATH. This is a new runtime dependency not present in the current Dockerfile/dev setup.
- **`formats` and `analysis` are dead features**: They existed only in the GQAdonis fork. Attempts to compile against the upstream without removing them will produce `Package kreuzberg does not have feature 'formats'` build errors.
- **Chunking field name**: The Rust `ChunkingConfig` uses `max_characters` and `overlap` (NOT `max_chars`/`max_overlap` — that's the Python API). The SKILL.md pitfalls section documents this explicitly.
- **`tree-sitter` feature**: Adds 300+ language code intelligence. Heavy compile-time dependency. Recommended only if code file extraction is a priority use case.

---

## 5. Suggested Phase Scope

Given the above, this phase can be scoped as a single OpenSpec change or split:

**Option A — single bundled change `uar-kreuzberg-v4-bump`** (recommended for velocity):
K-1 through K-9 as one PR. Keeps migration atomic and self-reviewing.

**Option B — two changes**:
- `uar-kreuzberg-v4-core` → K-1..K-6 (build + correctness)
- `uar-kreuzberg-v4-skills-mcp` → K-7..K-11 (strategic integrations)

Option A is recommended because the skills and MCP entries are trivial additions (JSON + file copy) that don't risk destabilising the build.
