## Why

The runtime images now ship a polyglot WASM toolchain (see KBD phase
`runtime-image-multi-language-toolchain`) so plugins can be authored in
Rust, JS/TS, Python, and Go and compiled to WebAssembly Components.
Before any of those plugins can be loaded, UAR needs a stable contract:
the WIT world guests target and the host-side strategy enum that decides
between JIT, AOT, and (future) interpreted execution.

This change introduces the **contract only**. No loader wiring, no
dispatcher integration, no host-function glue beyond a `log`/`now-ns`
import. Follow-up changes implement instantiation against the existing
`src/uar/runtime/wasm/sandbox.rs`.

## What Changes

- Add `wit/uar-plugin.wit` defining the `uar:plugin@0.1.0` package with
  three interfaces (`types`, `host`, `plugin`) and the `uar-plugin` world.
- Add `src/uar/runtime/wasm/plugin_loader.rs` with `PluginSource`,
  `PluginStrategy { Jit, Aot { cache_dir }, Interpreted }`,
  `CapabilityGrant` (deny-by-default), `LoadRequest`, `PluginId`,
  `PluginLoadError`, and the `PluginLoader` trait.
- Register the new module in `src/uar/runtime/wasm/mod.rs`.
- Cover the contract with two unit tests (defaults).
- No runtime behavior change yet — instantiation paths are typed surface only.

## Capabilities

### New Capabilities

- `plugin-model`: Defines the polyglot WebAssembly plugin contract,
  capability model, and JIT/AOT strategy that UAR plugins must satisfy.

### Modified Capabilities

- None.

## Impact

- Adds one feature-gated Rust module and one WIT file. Zero production
  surface change while the loader implementation is absent.
- Locks in the `uar:plugin@0.1.0` package name and capability shape.
  Breaking changes to either require a major version bump and a new
  OpenSpec change.
- The `Aot { cache_dir }` variant ties into the
  `PROMETHEUS_PLUGIN_CWASM_CACHE` env exposed by the production
  `Dockerfile` in change `dockerfile-runtime-wasmtime-base`. Wasmtime
  version is the implicit cache key.
- Future provider/skill registration code will consume `PluginId`. No
  current consumers, so no migration needed today.
