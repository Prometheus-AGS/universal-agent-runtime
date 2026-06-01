# Skill authoring guide

UAR supports three skill execution models, signalled by the `kind` field on the `Skill` domain type:

| `kind`     | Format                                 | Where to put it                                                                                   |
|------------|----------------------------------------|---------------------------------------------------------------------------------------------------|
| `Manifest` | `SKILL.md` (YAML frontmatter + body)   | `crates/prometheus-skill-system/skills/<domain>/<name>/SKILL.md` (built-in) or user filesystem    |
| `Wasm`     | WebAssembly Component (`.wasm` / `.cwasm`) | `~/.uar/skills/wasm-builtin/` or `~/.uar/skills/user/` (in-container: `/opt/uar/skills/...`)  |
| `Native`   | In-process Rust                        | Compiled into the UAR binary (the default for legacy / built-in tool helpers)                     |

Provenance is tracked separately via `origin`:

- `Builtin` — discovered at startup from the canonical built-in directories (`prometheus-skill-system` submodule, in-container `/opt/uar/skills/builtin/`). **Cannot be deleted via the API** — DELETE returns `409 Conflict { "error": "system_skill_immutable" }`.
- `User` — created via the Admin UI or REST API; fully editable/deletable.

## Manifest skills (`SKILL.md`)

YAML frontmatter is parsed by the built-in loader at startup. Required fields:

```yaml
---
name: my-skill
version: "1.0.0"
description: One-paragraph summary
triggers:
  keywords:
    - keyword-a
    - keyword-b
  semantic: optional natural-language description
allowed-tools: file_system code_interpreter
---
# Skill body (Markdown)

The body becomes the prompt overlay injected into the model context when this skill is matched.
```

Built-in manifest skills are scanned from `$UAR_BUILTIN_SKILLS_DIR` (default `crates/prometheus-skill-system/skills`). To include skills from `skills/imported/` sub-submodules, set `UAR_LOAD_IMPORTED_SKILLS=true`.

## WASM Component Model skills

Target the `uar:skill@0.1.0` WIT world:

```wit
package uar:skill@0.1.0;

world skill {
  export run: func(input: string) -> result<string, string>;
}
```

Authoring options:

- **Rust** — `cargo component new --lib my-skill`; implement the `run` export.
- **JavaScript / TypeScript** — [`jco`](https://github.com/bytecodealliance/jco) componentize.
- **Python** — [`componentize-py`](https://github.com/bytecodealliance/componentize-py).
- **Go** — TinyGo + the Component Model preview.

Build to a `.wasm` then optionally pre-compile to AOT for a faster cold start:

```bash
wasmtime compile -o my-skill.cwasm my-skill.wasm
```

Drop the resulting file into:

- `$UAR_SKILLS_WASM_BUILTIN_DIR` for built-in (system) skills.
- `$UAR_SKILLS_USER_DIR` for user skills.

Both `.wasm` and `.cwasm` are accepted — the runtime auto-selects the AOT path when available. The host runtime currently dispatches via an untyped stub; wit-bindgen integration to invoke `run` end-to-end lands with a follow-up change (the WIT contract itself is stable).

## Picking a kind

- **Default to Manifest** — fastest to author, hottest reload path, lowest overhead (just prompt context).
- **Use Wasm** when you need code execution that should not run as native Rust (sandboxed, language-flexible, hot-reloadable, cross-platform binaries).
- **Use Native** only when the skill is a fundamental host capability that must live in-process (filesystem helpers, sandbox runners, etc.) — these are typically not user-extensible.

## Provenance and immutability

Built-in skills survive UAR restarts because they're rediscovered from disk every startup. They are **not** persisted into SurrealDB and cannot be edited via the API; mutate them by changing the source in `prometheus-skill-system` and re-deploying. User skills are persisted normally.
