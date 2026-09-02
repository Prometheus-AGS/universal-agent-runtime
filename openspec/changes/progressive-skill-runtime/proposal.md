# progressive-skill-runtime

Rank 4 of the codex-harness-comparative-analysis change set. Source: gap G4 and the skill-attribution item of G11 in the phase `analysis.md`.

## Why

Every matched skill's full SKILL.md body is appended to the system prompt with no token budget (`src/uar/runtime/manager.rs:1448-1454`). The keyword threshold parameter is never read (`src/uar/runtime/skills/service.rs:698`), LLM matching falls back to keyword with a warning (`service.rs:638-643`), and the legacy classifier path injects every scored skill even below threshold (`manager.rs:1383-1394`). `max_active` and `prefer` are defined, defaulted, and never enforced (`src/uar/domain/artifact.rs:96-105`). There is no explicit activation for clients or for the model. Matching runs once against the first input (`manager.rs:1334`, `:1360`). Outcome telemetry excludes overlay-only skills (`manager.rs:1441-1445`), which are the common case. The 2026-08-09 gotcha measured 2,266 skills machine-wide, so unbudgeted injection is an observed scale problem.

Codex renders a one-line catalog under 2% of the context window with a 10,000-token cap and an 8,000-character fallback, truncates descriptions round-robin before omitting any entry, and loads a body only for an explicitly selected skill (`ext/skills/src/render.rs:17-22`, `:127-153`, `:325-366`, `:408-447`; `skills/src/selection.rs:42-109`). Its model-driven selector runs in shadow mode only (`ext/skills/src/shadow_selection_experiment/mod.rs:1`). Gemini CLI and OpenCode expose a model-callable activation tool. UAR takes both paths because its clients are programmatic. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- A budgeted skill catalog fragment: one line per eligible skill (id, title, source, description) under `min(2% of context window, 10,000 tokens)`, 8,000 characters when the window is unknown, round-robin description truncation, omission only under extreme pressure with a count note.
- Explicit activation: optional `skill_attachments` on `POST /api/uar/runs`, intersected with the effective eligible set and loaded before the first model call; a model-only `activate_skill(skill_id)` tool that loads the body, validates MCP dependencies, records exact usage, and updates the next step's tool set. Ineligible, disabled, missing, or dependency-invalid skills return a typed activation failure.
- Scored matching: `SkillCandidate` and `SkillMatchResult` with threshold and margin enforced; the "include top matches anyway" branch is removed; `max_active` enforced; `prefer` applied as a tie-break. `HarnessConfig.skill_activation_mode: legacy_overlay | catalog` decides what an above-threshold implicit match does: in `legacy_overlay` (migration default) it activates the skill as an implicit activation; in `catalog` (target default, flipped by a later change on recall evidence) it only marks the skill as suggested in the catalog. Below-threshold matches never activate in either mode.
- Shadow reducers: TF-IDF and local-embedding candidate reduction emit Recall@10 telemetry against explicit activations and do not change the catalog until recall is at least 99%.
- Activated bodies are conversation items with `Skill` authority and `Retention::Reclaimable`; compaction reclaims them first, and the most recent activation is re-attached after a compaction under a configurable budget (default 5,000 tokens per skill, 25,000 total).
- Telemetry: activation outcome recorded for every activated skill including overlay-only ones; new per-skill attribution counters `uar_skill_request_tokens_total{skill}` and `uar_skill_request_cost_usd{skill}` increase for every skill active during a request, while the existing unlabeled totals are unchanged, so multiple active skills attribute without double-counting the totals.

## Scope

- `src/uar/runtime/skills/{service.rs,registry.rs,matching.rs}`
- `src/uar/runtime/manager.rs` (skill block `:1306-1470`, correlation `:136-151`)
- `src/uar/domain/{artifact.rs,skills.rs}`
- `src/uar/api/routes.rs` (run request decoding), `src/uar/api/openapi.rs`
- `src/uar/runtime/native_skills/` (new `activate_skill` tool)
- `src/uar/telemetry/metrics.rs`
- new: `src/uar/runtime/skills/catalog.rs`, `src/uar/runtime/skills/activation.rs`
- tests: `tests/skill_activation_runtime.rs`; extend `tests/skill_scoped_governance.rs`

Out of scope: skill-contributed native tools or WASM components (wasm-component-skill-runtime), MCP server lifecycle (projected-mcp-runtime).

## Dependencies

deterministic-prompt-assembly (the catalog and bodies are fragments). fail-closed-tool-arguments (the `activate_skill` tool is a descriptor with `ModelOnly` exposure).

## Verification

Tier 0 per edit; Tier 1 the new tests, including a 2,000-skill synthetic catalog fitting the cap while retaining every id; Tier 2 at the boundary.

## The uncomfortable thing

Operators who today rely on implicit injection of full bodies will see fewer skills "fire" until they attach or the model activates them. The shadow recall telemetry exists so that switching implicit matching back on is a measured decision, not a feeling.
