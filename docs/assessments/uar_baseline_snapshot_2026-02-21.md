# Universal Agent Runtime — Baseline Snapshot

**Evolution Name**: `uar-evolution-2026-02`  
**Snapshot Date**: 2026-02-21  
**Purpose**: Quantitative baseline for future reassessment delta computation  
**Next Reassessment Target**: After Phase 1 + Phase 2 plan actions complete (~3–4 weeks)

---

## Usage Instructions

When running the next evolution cycle, load this file as `prior_assessment` input. The reassessment should compute deltas against every metric below and report:
- `delta` = new_value − baseline_value  
- `status` = improved / regressed / unchanged
- `narrative` = 1–2 sentence explanation of the change

Target: all critical/high-impact items show measurable improvement. Any regression in a "Healthy" indicator is a blocker.

---

## 1. Code Quality Metrics

| Metric | Baseline Value | Source | Target (Post-Plan) |
|--------|--------------|--------|-------------------|
| Rust source files | 191 | `find src -name "*.rs" \| wc -l` | — (tracking only) |
| Total Rust lines | 56,571 | `find src -name "*.rs" \| xargs wc -l` | — (tracking only) |
| TypeScript files | 42 | `find web -name "*.ts" \| wc -l` | — (tracking only) |
| Test files | 18 | `find tests -name "*.rs" \| wc -l` | ≥ 18 |
| Estimated test count | ~130–150 | Code review of new test files | Measured + reported |
| Cargo build errors | 0 (prior baseline, Feb 18) | `cargo check 2>&1 \| grep -c "^error"` | 0 |
| Clippy warnings | 0 (prior baseline, Feb 18) | `cargo clippy --all-targets 2>&1 \| grep -c "^warning"` | 0 |
| Test pass rate | 100% (109/109, Feb 18) | `cargo test 2>&1` | 100% |
| Test coverage | Not measured | `cargo tarpaulin` | Establish baseline |
| `#[allow()]` suppressions in production code | 1 (KreuzbergConfig) | `rg "#\[allow\(clippy" src/` | 0 (after A21) |

---

## 2. Architecture Metrics

| Metric | Baseline Value | Source | Target |
|--------|--------------|--------|--------|
| UAR modules under `src/uar/` | 16 | `ls src/uar/` | ≤ 16 (refactoring only) |
| `server.rs` line count | ~3000+ | `wc -l src/server.rs` | < 500 (after A10) |
| Storage traits with dual impl | 6/6 | Code review | 6/6 |
| Feature-gated code (`wasm-runtime`) | Correct | Code review | Correct |
| Git deps without rev/tag | 2 (`rmcp`, `surreal-memory`) | `grep "git = " Cargo.toml` | 0 (after A03) |
| SSE event variants | 13 | `NormalizedEvent` enum | ≥ 14 (after A17 adds artifact) |
| `NormalizedEvent::Artifact` maps to SSE | No (returns None) | `sse.rs:to_agui_event()` | Yes (after A17) |

---

## 3. Spec Coverage

| Metric | Baseline Value | Target |
|--------|--------------|--------|
| UAR spec sections implemented | 18/19 (95%) | 19/19 (100%) — after A05 |
| §06 A2UI runtime resolution | Not implemented | Implemented (A05) |
| §10 Memory system | Implemented | Implemented |
| §11 Context management | Implemented | Implemented |
| §15 Settings | Implemented | Implemented |

---

## 4. Infrastructure Metrics

| Metric | Baseline Value | Target (Post-Plan) |
|--------|--------------|-------------------|
| CI/CD pipeline present | No | Yes (GitHub Actions, after A01) |
| CI/CD runs on PRs | No | Yes |
| Test coverage reported in CI | No | Yes (after A22) |
| Rustdoc published | No | Yes (after A11) |
| Docker quickstart available | No | Yes (after A08) |
| Developer time-to-first-agent | ~30–60 min | < 5 min (after A08) |
| Commercial license documented | No | Yes (after A02) |
| `COMMERCIAL_LICENSE.md` exists | No | Yes |
| MCP server exposure | No | Yes (after A04) |
| AGNTCY Directory registration | No | Yes (after A06) |
| Execution cost tracking | No | Yes (after A07) |
| Trace ID propagation across stack | No | Yes (after A15) |
| Human-in-the-loop workflow | No | In progress (A12) |
| Governance admin API | No | In progress (A16) |

---

## 5. Competitive Position Metrics

| Metric | Baseline | Target |
|--------|---------|--------|
| Unique differentiators vs. market | 5 (compiler, Cedar runtime, embedded, UAR-AGENT-MD, dual-mode compiler) | 7 (add MCP server, AGNTCY integration) |
| Competitor protocols implemented | A2A ✅, MCP client ✅, AG-UI partial | A2A ✅, MCP client ✅, MCP server ✅, AG-UI ✅ (after A04, A05) |
| Protocol ecosystem connectability | Via A2A endpoint only | Via A2A + MCP server (after A04) |
| Internet of Agents discoverability | Internal registry only | AGNTCY Directory (after A06) |
| Enterprise AGPL barrier | Undocumented | Documented dual-license (after A02) |

---

## 6. Goal Alignment Scores

| Goal | Baseline Score | Target Score | Key Actions |
|------|--------------|-------------|-------------|
| G1: Code quality + spec | 80% | 95% | A01, A02, A03, A05, A10, A17 |
| G2: Competitive benchmark | 65% | 85% | A04, A06, A13 |
| G3: User demand addressed | 90% | 90% | A07, A08, A09, A12 |
| G4: Product-market fit | 70% | 85% | A02, A04, A06, A08, A09 |
| G5: Classified plan | 100% | 100% | Completed |
| G6: Baseline snapshot | 100% | 100% | Completed |
| **Overall** | **51%** (assessment-phase) | **90%** | All Phase 1+2 actions |

*Note: 51% reflects that G5 and G6 were 0% at assessment time; assessable goals (G1–G4) averaged 76%. After Plan phase complete, G5/G6 are 100%, raising effective score to 84%. Post-execution target: 90%.*

---

## 7. Open Items from Previous Assessment (Carried Forward)

| Item | Status at Feb 18 | Status at Feb 21 | Plan Action |
|------|-----------------|-----------------|-------------|
| §06 A2UI runtime resolution | Open | Open | A05 |
| CI/CD pipeline | Absent | Absent | A01 (Critical) |
| Commercial license docs | Absent | Absent | A02 (Critical) |
| Git dep pinning (`rmcp`, `surreal-memory`) | N/A | Absent | A03 (Critical) |

---

## 8. Key Dependency Versions (for future comparison)

| Dependency | Version at Baseline |
|-----------|-------------------|
| axum | 0.8 |
| surrealdb | 3.0.0 |
| ractor | 0.15 |
| cedar-policy | 4.9 |
| wasmtime | 41.0.3 |
| rmcp | git HEAD (unpinned) |
| surreal-memory | git HEAD (unpinned) |
| kreuzberg | v4.0.0-rc.17 (RC) |
| burn | 0.20.1 |
| tokio | 1.x |
| Rust edition | 2024 |

---

## 9. Raw Commands for Metric Collection

Run these commands to collect updated metrics in future reassessments:

```bash
cd /path/to/universal-agent-runtime

# Source file counts
find src -name "*.rs" | wc -l
find src -name "*.rs" | xargs wc -l 2>/dev/null | tail -1
find web -name "*.ts" | wc -l
find tests -name "*.rs" | wc -l

# Build health
cargo check 2>&1 | grep -c "^error"
cargo clippy --all-targets 2>&1 | grep "^warning\[clippy" | wc -l
cargo test --lib 2>&1 | tail -5

# Lint suppressions
rg "#\[allow\(clippy" src/ | wc -l

# Git deps
grep "git = " Cargo.toml | grep -v "rev ="

# server.rs size
wc -l src/server.rs

# SSE variant count
grep -c "=>" src/uar/api/sse.rs

# CI/CD present
ls .github/workflows/ 2>/dev/null && echo "Present" || echo "Absent"

# Commercial license
ls COMMERCIAL_LICENSE.md 2>/dev/null && echo "Present" || echo "Absent"

# Docker quickstart
ls docker-compose.quickstart.yaml 2>/dev/null && echo "Present" || echo "Absent"
```

---

*Baseline produced by Iterative Evolver — Plan Phase. This file is the source of truth for the next reassessment delta computation.*
