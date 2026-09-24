## Context
UAR combines a trusted host Rust runtime, Liter provider adaptation, MCP tools and policy, durable memory, and an entity-driven frontend. Compass indexes Rust only and has known parse failures; its edges are navigation evidence, not runtime proof. KBD position and versions.toml remain authoritative.

## Goals / Non-Goals
Create project-owned team definitions for Codex, Claude Code, Kimi Code, OpenCode and MiniMax CLI. Do not alter application code, activate live runtime agents, create other repository teams, install global CLIs, or claim a shared autonomous scheduler.

## Decisions
Use seven reusable roles: coordinator, runtime, providers, trust/tools, state/memory, console, independent verifier. Default to coordinator + one specialist + verifier; activate a second specialist only for independent ownership. One implementer suffices for an isolated edit. Specialist boundaries follow source responsibilities, not vendored graph node counts. Compare the three independent research candidates in the team decision artifact and submit it to a fresh-context critic.

A single portable manifest generates five native exports. Install project agent copies only; avoid duplicate plugin registration. Skills are project-local, explicitly referenced by path; preserve upstream files and record pinned provenance plus overriding UAR constraints. Model choices use discovered IDs and native aliases; unknown prices remain unknown. Kimi models are selected at invocation, not per-role. MiniMax uses a project data directory and requires an available mcode CLI and separate account setup.

Ownership is a coordination agreement, not filesystem enforcement. Native permissions remain authoritative. Shared composition roots and manifests require one named writer per task. No role edits versions.toml or vendored/submodule code by default. Final reviewer receives only artifacts, requirements and evidence, never producer history.

Cross-project work uses existing team handoff contracts and explicit file receipts; repository relationships distinguish direct dependencies, tooling, and future peers. No transport or peer team is presumed active.

## Risks / Trade-offs
Seven loaded specialists can waste context: keep activation narrow and read skill bodies on demand. Native feature parity is incomplete; validate discovery independently and report source-only MiniMax support. A graph can miss entrypoints: inspect source for parse failures and all non-Rust surfaces. Skill examples can contradict entity boundaries or pins: project instructions override them.

## Migration Plan
Add project files without overwriting existing agents or MCP settings. Keep generated exports and installation inventory for review and rollback. Remove only listed newly installed files to undo; preserve existing configuration.

## Open Questions
No blocked design questions. Live inference, provider billing, and MiniMax execution remain separate from static installation acceptance.
