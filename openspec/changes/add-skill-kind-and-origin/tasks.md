## 1. Domain types

- [x] 1.1 Added `SkillKind { Native, Manifest, Wasm }` + `SkillOrigin { Builtin, User }` enums in `src/uar/domain/skills.rs` with kebab-case-ish lowercase serde.
- [x] 1.2 Added `kind: SkillKind` and `origin: SkillOrigin` fields to `Skill` with `#[serde(default)]`.
- [x] 1.3 Default impls (Native / User).

## 2. Persistence

- [x] 2.1 Surreal read path tolerates missing fields (serde default handles it).
- [ ] 2.2 Postgres migration — deferred (postgres-backend not in default build).

## 3. Service guard

- [x] 3.1 `SkillService::delete_skill_permanent` returns `system_skill_immutable` error for Builtin skills.
- [x] 3.2 `src/uar/api/skills.rs` maps the error to `409 Conflict` with structured body.

## 4. Tests

- [ ] 4.1 Unit on legacy JSON — deferred to integration-tests change.
- [ ] 4.2 Integration DELETE 409 — deferred.

## 5. Construction sites updated

- [x] 5.1 `src/uar/api/skills.rs::create_skill` defaults both fields.
- [x] 5.2 `src/uar/runtime/skills/storage/filesystem.rs::load_skill` defaults `origin=User`, sets `kind=Manifest`.
