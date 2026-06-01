## Why

The UAR's `kreuzberg` document-extraction dependency points at the stale
`GQAdonis/kreuzberg` fork with two features (`formats`, `analysis`) that **do
not exist in the upstream v4 codebase**, making the build broken against the
canonical library. The upstream `kreuzberg-dev/kreuzberg` has jumped to
**v4.9.8 LTS**, adding 35+ new formats, a full CLI, an embedded MCP server,
native chunking, language detection, and — critically — a published
`skills/kreuzberg/SKILL.md` agent-skill that teaches any AI assistant the
correct API without hallucination.

This change fixes the broken dependency, migrates the one breaking API
(`OutputFormat` string → enum), unlocks the MCP server so agents can extract
documents natively, and wires the kreuzberg skill into the UAR's builtin
skill auto-loader.

## What Changes

### 1. `Cargo.toml` — dependency re-point + feature overhaul

```toml
# BEFORE
kreuzberg = {
    git = "https://github.com/GQAdonis/kreuzberg.git",
    branch = "main",
    default-features = false,
    features = ["tokio-runtime", "formats", "analysis"]
}

# AFTER
kreuzberg = {
    git = "https://github.com/kreuzberg-dev/kreuzberg.git",
    tag = "v4.9.8",
    default-features = false,
    features = [
        "tokio-runtime",        # extract_file_sync, batch_extract_file_sync
        "pdf",                  # replaces "formats" — PDF via pdf_oxide
        "office",               # replaces "formats" — docx, odt, pptx, xlsx, etc.
        "excel",                # replaces "formats" — xlsx/xlsm/xls
        "email",                # replaces "formats" — .eml, .msg, .pst
        "html",                 # replaces "formats" — HTML-to-markdown
        "archives",             # replaces "formats" — zip, tar, 7z
        "ocr",                  # replaces "analysis" — Tesseract backend
        "language-detection",   # replaces "analysis" — whatlang
        "chunking",             # NEW — RAG text chunking
        "mcp",                  # NEW — embedded MCP server
    ]
}
```

### 2. `src/uar/file_processing/kreuzberg.rs` — API fixes

- **`build_extraction_config()`**: map `KreuzbergConfig.output_format: String`
  to `kreuzberg::OutputFormat` enum (`"markdown"` → `OutputFormat::Markdown`,
  `"text"` → `OutputFormat::Plain`).
- **`tables` rendering**: replace `writeln!(content, "{table:?}\n")` with
  `writeln!(content, "{}\n", table.markdown)` so extracted tables render as
  clean Markdown instead of Rust debug output.

### 3. `src/config.rs` — expose `ChunkingConfig`

Add a `chunking: Option<KreuzbergChunkingConfig>` field to `KreuzbergConfig`
with `max_characters: usize` and `overlap: usize` fields. Wire it through
`build_extraction_config()` when present.

### 4. `mcp.json` — kreuzberg MCP server entry

Add the kreuzberg stdio MCP server so every agent has access to
`extract_file`, `extract_bytes`, `batch_extract_files`, `detect_mime_type`,
`chunk_text`, `embed_text`, and `extract_structured` tools:

```json
"kreuzberg": {
  "command": "kreuzberg",
  "args": ["mcp", "--transport", "stdio"]
}
```

### 5. `crates/prometheus-skill-system` — kreuzberg SKILL.md import

Open a PR on `prometheus-skill-system` that copies
`skills/kreuzberg/SKILL.md` + `references/` (8 reference docs) from
`kreuzberg-dev/kreuzberg@v4.9.8` into
`skills/document-extraction/kreuzberg/`. Bump the UAR submodule pointer
once merged.

### 6. `src/uar/runtime/skills/builtin_loader.rs` — multi-root support

Extend the loader to read an optional `UAR_EXTRA_BUILTIN_SKILL_DIRS`
env var (colon-separated paths). All directories are scanned in addition
to the primary `UAR_BUILTIN_SKILLS_DIR` root, using identical SKILL.md
discovery and frontmatter parsing logic.

## Runtime / UX Impact

- **Agents gain document extraction MCP tools** (`extract_file`,
  `batch_extract_files`, etc.) without any user configuration — kreuzberg
  MCP runs as a stdio server alongside `time` and `tavily`.
- **The kreuzberg system skill** appears in `/api/uar/skills` with
  `origin = Builtin`, instructing agents on correct API usage for all
  91+ supported formats.
- **Provider compatibility**: no LLM provider changes. The kreuzberg library
  is used for local file extraction only.
- **Realtime state**: no entity graph or stream changes.
- **KBD workflow state**: `current-waypoint.json` updated to
  `uar-kreuzberg-v4-migration` phase on merge.

## Acceptance

- `cargo build --release` succeeds with no `unknown feature` warnings.
- `cargo test` passes, including kreuzberg provider unit tests.
- Server startup logs include the kreuzberg builtin skill count.
- `GET /api/uar/skills` lists a skill with `origin = "Builtin"` and
  `name` matching `kreuzberg`.
- `mcp.json` contains `kreuzberg` server entry.
- MCP tool `extract_file` is callable from an agent session and returns
  extracted text from a sample PDF.
