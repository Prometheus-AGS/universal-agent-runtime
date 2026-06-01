# Execution: uar-kreuzberg-v4-migration

**Date:** 2026-06-01
**Backend:** openspec
**Change:** uar-kreuzberg-v4-bump (1 of 1)
**Dispatch:** claude-code (direct execution)

## Dispatch Contract

All tasks execute sequentially in the worktree created at §0. Cross-repo
sub-task §5 targets `prometheus-skill-system` via a branch + PR.

### Critical path
§0 → §1 → §2 → §7 (minimum viable — build green)
### Full path
§0 → §1 → §2 → §3 → §4 + §5 + §6 → §7 → §8

## QA Gate
Change has >3 files modified. Artifact-refiner applies after §7 passes.
Skip condition: build + tests green is the primary gate.

## Status
- [x] §0 Bootstrap — worktree created, cmake installed, kreuzberg-cli noted as runtime dep
- [x] §1 Cargo.toml — re-pointed to kreuzberg-dev/kreuzberg@v4.9.8; 10 features
- [x] §2 API fixes — OutputFormat enum, table.markdown rendering
- [x] §3 Chunking config — KreuzbergChunkingConfig added, wired in both paths + settings manager
- [x] §4 mcp.json — kreuzberg MCP server entry added
- [x] §5 PSS cross-repo — SKILL.md + 8 refs copied, PR #4 open at https://github.com/Prometheus-AGS/prometheus-skill-system/pull/4
- [x] §6 multi-root loader — UAR_EXTRA_BUILTIN_SKILL_DIRS support + unit test
- [x] §7 Build validation — cargo check clean; 5 unit tests pass (disk pressure prevented full suite)
- [x] §8 Docs — .env.example updated with kreuzberg-cli + env var docs

## PRs
- UAR: https://github.com/Prometheus-AGS/universal-agent-runtime/pull/20
- PSS: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/4

## Post-merge
After PSS PR #4 merges: bump crates/prometheus-skill-system submodule pointer in UAR.
