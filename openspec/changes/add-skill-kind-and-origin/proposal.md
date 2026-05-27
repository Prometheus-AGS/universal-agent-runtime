## Why

UAR's current `Skill` domain type assumes a single execution model (WASM-flavored). We're about to load both Markdown/YAML-manifest skills (from `prometheus-skill-system`) and true WASM Component Model skills, and we need a guard so system-shipped skills cannot be deleted via the API. A single `kind` discriminator plus an `origin` flag covers both needs cleanly.

## What Changes

- Add `pub enum SkillKind { Manifest, Wasm, Native }` and `pub enum SkillOrigin { Builtin, User }` to `src/uar/domain/skills.rs`.
- Add fields `kind: SkillKind` and `origin: SkillOrigin` to `Skill`. Default deserialization: `kind = Native`, `origin = User` (handles legacy rows).
- Migrate persisted rows in surreal + postgres providers: backfill missing fields to defaults on read.
- `SkillService::delete` returns `409 Conflict` with body `{"error": "system_skill_immutable"}` when `origin == Builtin`. Surface the 409 in `src/uar/api/skills.rs`.
- No UI change (handled by `builtin-skills-ui-affordance`).

## Acceptance

- `DELETE /api/skills/{id}` with a Builtin skill → 409.
- Legacy rows continue to load and round-trip without manual migration.
- New skills written via the API default to `origin = User`.
