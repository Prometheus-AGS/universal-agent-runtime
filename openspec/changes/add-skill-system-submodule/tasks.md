## 1. Submodule

- [x] 1.1 `git submodule add git@github.com:Prometheus-AGS/prometheus-skill-system.git crates/prometheus-skill-system`.
- [x] 1.2 `git submodule update --init --recursive` — 82 SKILL.md files plus 4 nested submodules cloned.
- [x] 1.3 `.gitmodules` populated.
- [x] 1.4 `[workspace] exclude = ["crates/prometheus-skill-system"]` added to root Cargo.toml.

## 2. Loader

- [x] 2.1 Created `src/uar/runtime/skills/builtin_loader.rs`.
- [x] 2.2 WalkDir scan of `SKILL.md` files.
- [x] 2.3 `serde_yaml` frontmatter parsing.
- [x] 2.4 Maps to `Skill { kind: Manifest, origin: Builtin, … }`.
- [x] 2.5 Skips `skills/imported/` unless `UAR_LOAD_IMPORTED_SKILLS=true`.

## 3. Service integration

- [x] 3.1 `SkillService::register_builtins(Vec<Skill>)` added.
- [x] 3.2 Called from `server::build_app` after existing skill providers initialize.
- [x] 3.3 Logs `registered N builtin skills` at startup.

## 4. Tests

- [x] 4.1 Unit: `split_frontmatter` works on sample SKILL.md.
- [ ] 4.2 Integration smoke (counted skills in startup log) — pending live run after install.

## 5. Docs

- [ ] 5.1 Clone instructions — deferred to integration-tests-and-docs change.
- [ ] 5.2 `UAR_LOAD_IMPORTED_SKILLS` doc — deferred.
