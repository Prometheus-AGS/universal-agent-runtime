# D-E: Artifact-refiner QA gate — formally retired for this project

**Date:** 2026-07-04
**Phase:** `uar-security-deps-and-hygiene`

## Decision

The artifact-refiner QA gate (`/refine-validate` / `/refine-code`,
described in `kbd-execute`'s SKILL.md) is **not required** for changes in
this project going forward. This project instead verifies every change
via:

- **Rust code changes**: `cargo check` (lib + affected test targets),
  `cargo test` (affected scope, then full `cargo test --lib` at a
  checkpoint), `cargo clippy` (confirm zero new warnings attributable to
  the change).
- **Docs/config-only changes**: direct inspection (cross-referenced
  facts checked against source, YAML/JSON syntax validated, markdown
  fence balance checked).
- **CI/workflow changes**: YAML syntax validation (`python3 -c "import
  yaml; ..."`); execution-tested only when a runner is available.

## Why

This is the 4th+ consecutive phase (`uar-next-harness`,
`uar-spec-v2-and-polish`, and now `uar-security-deps-and-hygiene`) where
the artifact-refiner gate was carried as unaddressed debt rather than
either automated or explicitly dropped. Direct check via `ToolSearch`
during this phase's assessment confirmed: **there is currently no
invokable artifact-refiner MCP tool available in this environment at
all** — this isn't a matter of wiring an existing tool into this
project's KBD flow; the tool itself isn't provisioned here. Re-carrying
an "automate this" item across 4+ phases when the tool doesn't exist in
this environment is scope drift, not a real open item — every change in
that time has, in practice, been verified via the methods above, and
those methods have caught real bugs (e.g. `uar-spec-v2-and-polish`'s
CH-20 guardrails test-module import bug, CH-14's test assertion bug).

## What would change this decision

If an artifact-refiner MCP tool becomes available in a future session
(confirm via `ToolSearch` before assuming otherwise — tool availability
can change between environments/sessions), re-evaluate whether wiring it
in adds value beyond the verification methods above, rather than
treating its absence as permanent.

## Consequence for OpenSpec changes

Per this project's own established practice (not this decision, already
true before it — confirmed via `git log` on `uar-spec-v2-and-polish`'s
change history), OpenSpec changes in `openspec/changes/<id>/` are
verified and merged without ever running `/opsx:verify` + `/opsx:archive`
either; they remain in `openspec/changes/` rather than
`openspec/changes/archive/`. This decision does not change that —
archiving is a separate, still-open question, not addressed here.
