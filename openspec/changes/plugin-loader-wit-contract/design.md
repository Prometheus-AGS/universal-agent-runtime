# Design — plugin-loader-wit-contract

## Scope

Contract only. This change does NOT implement instantiation, dispatch, or
host-function wiring beyond what is needed to declare the surface. Two
follow-up changes will land separately:

1. `plugin-loader-instantiation` — wire `PluginLoader` to
   `wasmtime::component::Linker` against `src/uar/runtime/wasm/sandbox.rs`.
2. `plugin-loader-dispatcher` — register loaded plugins with the existing
   skill/tool dispatcher.

## WIT package layout

`wit/uar-plugin.wit` defines a single package `uar:plugin@0.1.0`. Three
interfaces, one world:

| Interface | Direction | Purpose |
|---|---|---|
| `types`  | shared    | `plugin-id`, `capability-grant`, `invoke-result`, `plugin-error`. |
| `host`   | import    | `log`, `now-ns`. Both capability-checked. |
| `plugin` | export    | `init`, `invoke`, `shutdown`. |

Versioning: any change to interface shape requires a new package version
(`uar:plugin@0.2.0`). The host loader matches `0.x` packages with semver
compatibility; major bumps refuse to load older plugins.

## Strategy decision matrix

```
                ┌─────────────┐
                │ LoadRequest │
                └──────┬──────┘
                       │
        ┌──────────────┴───────────────┐
        │                              │
   PluginSource::Wasm(path)     PluginSource::Cwasm(path)
        │                              │
        ▼                              ▼
   strategy == Jit?            strategy must be Aot
        │                              │
   ┌────┴──────┐                       │
   │           │                       │
   yes         no                      │
   │           │                       │
   ▼           ▼                       ▼
  JIT       Aot{cache_dir}        deserialize_file
  load        │                   (no Cranelift)
  via         ▼
Component  cache hit?
::new      │      │
           yes    no
           │      │
           ▼      ▼
        load   wasmtime
        cached compile,
        cwasm  write to
               cache,
               then load
```

Rules:

- `PluginStrategy::Jit` always works for `PluginSource::Wasm`.
- `PluginStrategy::Aot { cache_dir }` works for both sources; for
  `Wasm`, the loader precompiles on first miss.
- `PluginSource::Cwasm` requires an Aot strategy. Combining it with `Jit`
  is a programmer error and returns `PluginLoadError::CacheMissNoFallback`.
- `PluginStrategy::Interpreted` returns `InterpretedNotImplemented` in
  v1. Reserved for WAMR/wasm3 integration when memory pressure rules out
  Cranelift's working set.

## `.cwasm` version policy

`.cwasm` artifacts are wasmtime-version-locked. The cache layout is:

```
${PROMETHEUS_PLUGIN_CWASM_CACHE}/${WASMTIME_VERSION}/<plugin-hash>.cwasm
```

- `PROMETHEUS_PLUGIN_CWASM_CACHE` is set in the production `Dockerfile`
  (change `dockerfile-runtime-wasmtime-base`) to `/var/cache/uar/cwasm`.
- `WASMTIME_VERSION` is the same pinned ARG used in both `Dockerfile`
  and `Dockerfile.rust-dev` — bumping wasmtime in one MUST be done in the
  other as part of the same KBD change.
- Loader compares its compiled-in wasmtime version against the cache
  subdir; on mismatch it falls through to JIT compilation and writes a
  fresh artifact under the new version dir. Old version dirs are GC'd by
  a future maintenance job, not at load time.

## Capability model

- Deny-by-default. `CapabilityGrant::default()` grants nothing and caps
  resources at 32 MB memory / 5 s CPU.
- Capability checks are enforced **at the host import boundary** (see
  `interface host` in the WIT). Guests cannot bypass by importing
  unauthorized symbols — the linker simply does not provide them.
- `cpu-ms-max` is enforced via wasmtime fuel + epoch interruption (to be
  wired in the instantiation follow-up).

## Runtime selection (future)

The `Interpreted` variant is forward-looking. When a plugin's manifest
declares `prefer_interpreter: true` AND the host's resident set is below
a configured threshold, the loader will route to WAMR `iwasm`. WAMR is
already installed in `Dockerfile.rust-dev` for the build-side `wamrc`
AOT compiler. Whether to also ship WAMR in production is deferred to a
separate KBD change.

## Open questions

1. Should `init`'s return value carry a structured descriptor (name,
   version, declared capabilities) instead of a raw `string`? Current
   shape is permissive; tightening is a non-breaking refinement if the
   string parses as JSON.
2. Per-plugin `tracing` span propagation across the host/guest boundary
   — out of scope for the contract change, addressed in dispatcher work.
3. Async invoke. The WIT `invoke` is synchronous today. Async needs
   `wasmtime`'s async store + epoch yields, decided in the
   instantiation follow-up based on real plugin workloads.
