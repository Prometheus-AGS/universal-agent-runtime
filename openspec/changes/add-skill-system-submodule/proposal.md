## Why

`prometheus-skill-system` is the canonical home for Markdown/YAML skill manifests we want UAR to ship with by default. These skills (`SKILL.md` files) are not Rust crates and not WASM — they are structured prompt-context contributions that UAR's matcher selects at request time. Embedding the repo as a submodule lets us version and update them together, and the `BuiltinSkillLoader` makes them automatically available with `origin = Builtin` so they cannot be deleted.

## What Changes

- Add submodule: `git submodule add git@github.com:Prometheus-AGS/prometheus-skill-system.git crates/prometheus-skill-system` (note: NOT a Cargo workspace member — it's a heterogeneous repo of manifests + sub-submodules).
- Document that consumers must clone with `--recurse-submodules` or run `git submodule update --init --recursive`.
- New module `src/uar/runtime/skills/builtin_loader.rs`:
  - Walks `crates/prometheus-skill-system/skills/<domain>/<name>/SKILL.md` (excluding `skills/imported/` unless `UAR_LOAD_IMPORTED_SKILLS=true`).
  - Parses YAML frontmatter with `serde_yaml` into `BuiltinManifest { name, description, license, version, tags, triggers, … }`.
  - Constructs a `Skill { kind: Manifest, origin: Builtin, … }` per manifest.
- Loader runs at server bootstrap *after* the existing `Local Skills` filesystem provider, registering all builtins via `SkillService::register_builtin(skill)` (new method that bypasses normal storage and marks the record in-memory only — discussion item: persist or keep in-memory? Default to in-memory to avoid migration churn).
- Logs `Loaded N builtin manifest skills` at startup with N from the walk.

## Acceptance

- Cloning the UAR repo + `git submodule update --init --recursive` populates `crates/prometheus-skill-system/skills/`.
- UAR startup logs the number of builtin manifest skills loaded.
- `GET /api/uar/skills` (or equivalent) lists Builtin manifest skills alongside user skills.
- `DELETE` on a Builtin manifest skill returns 409 (enforced by change `add-skill-kind-and-origin`).
