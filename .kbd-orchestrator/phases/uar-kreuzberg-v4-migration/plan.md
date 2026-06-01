# Plan: uar-kreuzberg-v4-migration

**Phase start:** 2026-06-01
**Backend:** OpenSpec
**Strategy:** Single bundled change (Option A — velocity)
**Evolver:** none

---

## Change Roster

| # | Change ID | Priority | Scope | Cross-repo? |
|---|---|---|---|---|
| 1 | `uar-kreuzberg-v4-bump` | P0→P3 | All K-1..K-11 items | Yes (PSS PR in §5) |

One change captures the full migration. The cross-repo sub-task (§5 —
copying the kreuzberg skill into `prometheus-skill-system`) is a tracked
sub-step within the change's `tasks.md`, not a separate OpenSpec change.
This keeps the submodule bump atomic with the loader and MCP wiring.

---

## Execution Order

Tasks within `uar-kreuzberg-v4-bump` must run in the following order due to
build-time and cross-repo dependencies:

```
§0 Bootstrap (worktree, kreuzberg-cli install)
  └─ §1 Cargo.toml re-point + feature overhaul   ← P0, unblocks everything
       └─ §2 kreuzberg.rs API fixes               ← P0, depends on §1 compiling
            └─ §3 Config chunking field           ← P1
                 ├─ §4 mcp.json entry             ← P2, independent of §3
                 ├─ §5 PSS cross-repo PR + bump   ← P3, independent of §3
                 └─ §6 builtin_loader multi-root  ← P3, depends on §5 merged
                      └─ §7 Build validation      ← gate before §8
                           └─ §8 Docs
```

**Critical path:** §0 → §1 → §2 → §7 (minimum viable build fix)
**Full path:** §0 → §1 → §2 → §3 → §4 + §5 + §6 → §7 → §8

---

## Recommended Agent Assignment

| Task group | Agent | Notes |
|---|---|---|
| §0 Bootstrap | claude-code | Script call + CLI check |
| §1 Cargo.toml | claude-code | Mechanical text edit + `cargo fetch` |
| §2 API fixes | claude-code | Rust edit in single file |
| §3 Config | claude-code | Struct + wiring in config.rs + settings/manager.rs |
| §4 mcp.json | claude-code | JSON edit + smoke test |
| §5 PSS cross-repo | claude-code | Requires PSS worktree; file copy + PR |
| §6 builtin_loader | claude-code | Rust extension + unit test |
| §7 Build validation | claude-code | `cargo build/test/clippy` |
| §8 Docs | claude-code | README + .env.example edits |

---

## Risk Notes

1. **`tag = "v4.9.8"` must exist** — confirmed via `gh api repos/kreuzberg-dev/kreuzberg/releases/latest`.
2. **kreuzberg-cli binary required** for MCP smoke test and runtime use — not in current Dockerfile; document as new dev dep.
3. **Chunking field name** — Rust uses `max_characters` / `overlap`, NOT `max_chars` / `max_overlap` (Python names). SKILL.md pitfalls section confirms this.
4. **PSS PR gate** — §6 (multi-root loader) can be coded immediately but the kreuzberg skill won't appear at startup until the PSS submodule pointer is bumped after the PSS PR merges.
5. **OutputFormat** — `kreuzberg::OutputFormat` is an enum; `KreuzbergConfig.output_format` stays as `String` at the config layer and is converted in `build_extraction_config()`. Do not change the TOML/env surface.

---

## Phase Success Criteria

- [ ] `cargo build --release` clean (no unknown-feature errors)
- [ ] `cargo test` green (all kreuzberg unit tests pass)
- [ ] Server startup logs kreuzberg builtin skill count > 0
- [ ] `GET /api/uar/skills` returns at least one `origin = "Builtin"` kreuzberg skill
- [ ] `mcp.json` contains `kreuzberg` MCP server entry
- [ ] Agent session can call `extract_file` MCP tool and receive extracted text
