## Why

The strongest plugin model UAR can support is the **WebAssembly Component Model** — sandboxed, language-agnostic, hot-loadable, with formal interfaces declared in WIT. UAR already pulls in `wasmtime`/`wasmtime-wasi` 41 as optional deps under the `wasm-runtime` Cargo feature; we now formalize the host-side runtime and a stable WIT world (`uar:skill@0.1.0`) skill authors can target. AOT precompilation via `wasmtime compile` (Cranelift backend) gives cold-start parity with native skills.

## What Changes

### WIT world

- New file `wit/uar-skill.wit` declaring world `uar:skill@0.1.0`:
  ```wit
  package uar:skill@0.1.0;

  world skill {
    export run: func(input: string) -> result<string, string>;
    // Streaming variant kept as a planned extension; see follow-up change.
  }
  ```
- Versioned additively until 1.0; the package version is independent of UAR's crate version.

### Host runtime

- New module `src/uar/runtime/skills/wasm_runtime.rs`.
- `WasmSkillRuntime { engine: wasmtime::Engine, linker: wasmtime::component::Linker<HostState> }` constructed once at startup.
- `load_component(path: &Path) -> Result<LoadedSkill>`:
  - If path ends in `.cwasm` (AOT-precompiled), use `Module::deserialize_file`.
  - If `.wasm`, JIT compile via `Component::from_file`.
- `LoadedSkill::run(&self, input: &str) -> Result<String>` instantiates and calls the exported `run` function via the generated bindings (`wasmtime::component::bindgen!`).

### Discovery

- Directories scanned at startup:
  1. `$UAR_SKILLS_WASM_BUILTIN_DIR` (default `/opt/uar/skills/wasm-builtin` in container, `~/.uar/skills/wasm-builtin` in dev) — origin `Builtin`.
  2. `$UAR_SKILLS_USER_DIR` (default `/var/lib/uar/skills-user`) — origin `User`.
- For each `.wasm` or `.cwasm` discovered, register a `Skill { kind: Wasm, origin: …, … }` in `SkillService`.

### Dispatch

- When `SkillService::execute(skill_id, input)` is called and the `Skill.kind == Wasm`, route to `WasmSkillRuntime::run` instead of the existing native path.
- All within the `wasm-runtime` Cargo feature (already in the release build).

## Acceptance

- A fixture WASM component (`tests/fixtures/echo_skill.wasm`) registers at startup and round-trips a string through `run`.
- An AOT-precompiled `.cwasm` of the same component loads ≥3× faster than the JIT variant in a benchmark.
- A user can drop a `.wasm` into `~/.uar/skills/wasm-builtin/` and see the skill listed after a server restart (hot reload is a follow-up change).
