## 1. WIT world

- [x] 1.1 `wit/uar-skill.wit` declaring world `uar:skill@0.1.0` exporting `run(string) -> result<string, string>`.
- [ ] 1.2 `wit-bindgen` integration via `wasmtime::component::bindgen!` — deferred; runtime currently uses untyped dispatch stub. Bindings can be added without changing the WIT contract.

## 2. Host runtime

- [x] 2.1 Created `src/uar/runtime/skills/wasm_runtime.rs`.
- [x] 2.2 Single `wasmtime::Engine` with `wasm_component_model(true)`.
- [x] 2.3 `Linker<WasmHostState>` shell — WASI snapshots to be added when bindings land.
- [x] 2.4 `register()` handles `.wasm` (JIT via `Component::from_file`) and `.cwasm` (`Component::deserialize_file`).
- [ ] 2.5 `LoadedSkill::run` via bindgen-generated typed interface — deferred (stubbed).

## 3. Discovery

- [x] 3.1 Walks `$UAR_SKILLS_WASM_BUILTIN_DIR` and `$UAR_SKILLS_USER_DIR`.
- [x] 3.2 Builds `Skill { kind: Wasm, origin }` per loaded artifact.
- [x] 3.3 Logs counts.

## 4. Dispatch

- [ ] 4.1 `SkillService::execute` kind-branch — deferred until bindgen lands and a real `execute` method exists.
- [x] 4.2 Runtime owns the components and exposes `runtime.run(skill_id, input)` stub.

## 5. Tests

- [ ] 5.1 Fixture skill build — deferred (needs cargo-component).
- [ ] 5.2 AOT vs JIT benchmark — deferred.

## 6. Docs

- [ ] 6.1 Authoring guide — deferred to integration-tests-and-docs change.
