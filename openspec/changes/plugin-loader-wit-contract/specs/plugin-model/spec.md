# plugin-model Specification (delta)

## ADDED Requirements

### Requirement: Polyglot Plugin Contract

UAR SHALL expose a single WIT package `uar:plugin@0.1.0` that every
language-agnostic plugin targets.

#### Scenario: A plugin author compiles a component
- **WHEN** an author runs `cargo component build`, `jco componentize`,
  `componentize-py componentize`, or `tinygo build -target wasip2`
- **THEN** they MUST target the WIT world `uar-plugin` in
  `wit/uar-plugin.wit`
- **AND** the produced `.wasm` Component MUST export an implementation
  of `interface plugin`.

#### Scenario: A plugin imports a host capability it was not granted
- **WHEN** a plugin attempts to call `host.now-ns` without `clock-read`
  in its `CapabilityGrant`
- **THEN** the host MUST return `result<u64, string>::err` with a
  capability-denied message
- **AND** MUST NOT trap the guest for the denial alone.

### Requirement: Strategy Selection

UAR SHALL select between JIT, AOT, and (future) Interpreted execution
strategies on a per-plugin basis at load time.

#### Scenario: AOT cache hit at load
- **WHEN** `LoadRequest.strategy = Aot { cache_dir }` and a `.cwasm` for
  the plugin hash already exists under `cache_dir/${WASMTIME_VERSION}/`
- **THEN** the loader MUST use `Component::deserialize_file` and SHALL
  NOT invoke Cranelift.

#### Scenario: AOT cache miss with JIT fallback
- **WHEN** `LoadRequest.strategy = Aot { cache_dir }` and no `.cwasm`
  exists for the plugin under the current wasmtime version
- **THEN** the loader SHALL precompile the source, write
  `<plugin-hash>.cwasm` under `cache_dir/${WASMTIME_VERSION}/`, and
  proceed with the freshly compiled component.

#### Scenario: Cwasm source with JIT strategy
- **WHEN** `LoadRequest.source = Cwasm(_)` and `strategy = Jit`
- **THEN** the loader MUST return
  `PluginLoadError::CacheMissNoFallback(Jit)`.

#### Scenario: Interpreted strategy requested in v1
- **WHEN** `LoadRequest.strategy = Interpreted`
- **THEN** the loader MUST return
  `PluginLoadError::InterpretedNotImplemented`.

### Requirement: Capability Defaults

UAR plugins SHALL receive deny-by-default capability grants.

#### Scenario: Caller omits explicit grant
- **WHEN** a caller constructs `LoadRequest` without overriding
  `CapabilityGrant::default()`
- **THEN** the plugin SHALL receive `net_outbound = false`,
  `fs_read = false`, `fs_write = false`, `clock_read = false`,
  `memory_mb_max = 32`, `cpu_ms_max = 5_000`.

### Requirement: Cwasm Version Lock

UAR SHALL scope precompiled `.cwasm` artifacts by wasmtime version.

#### Scenario: Wasmtime is upgraded
- **WHEN** the runtime wasmtime version differs from the cache
  subdirectory used for an existing `.cwasm` artifact
- **THEN** the loader MUST treat the artifact as absent and recompile
  under the new version subdirectory
- **AND** MUST NOT attempt to load the stale `.cwasm`.

### Requirement: WIT Package Versioning

UAR SHALL refuse to load components whose declared `uar:plugin` package
major version differs from the host's.

#### Scenario: Major version mismatch
- **WHEN** a component declares `uar:plugin@2.0.0` and the host
  implements `uar:plugin@0.1.0`
- **THEN** the loader MUST return `PluginLoadError::HostRejected` with
  a version-mismatch message
- **AND** MUST NOT instantiate the component.
