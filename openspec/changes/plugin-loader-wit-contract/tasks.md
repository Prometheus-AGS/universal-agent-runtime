# Tasks — plugin-loader-wit-contract

- [x] Author `wit/uar-plugin.wit` with package `uar:plugin@0.1.0`, interfaces `types`, `host`, `plugin`, world `uar-plugin`.
- [x] Add `src/uar/runtime/wasm/plugin_loader.rs` with `PluginSource`, `PluginStrategy`, `CapabilityGrant` (deny-by-default), `LoadRequest`, `PluginId`, `PluginLoadError`, `PluginLoader` trait.
- [x] Register `plugin_loader` in `src/uar/runtime/wasm/mod.rs` and document in the module rustdoc.
- [x] Add unit tests asserting `CapabilityGrant::default()` denies all capabilities and `PluginStrategy::default()` is `Jit`.
- [x] Write OpenSpec capability delta at `specs/plugin-model/spec.md` (this change).
- [x] Write design doc covering precompile vs JIT decision matrix, `.cwasm` version policy, and runtime-selection rules.
- [ ] Verify `wit-bindgen rust wit/uar-plugin.wit` generates bindings without error. (Deferred until `wit-bindgen-cli` is installed locally; gated on `Dockerfile.rust-dev` build.)
- [ ] Verify `cargo check --features wasm-runtime` and `cargo test --features wasm-runtime plugin_loader` pass. (Deferred — blocked by the same worktree workspace-manifest drift that blocked clippy in change 1.)
- [ ] Run `openspec validate plugin-loader-wit-contract --strict`. (Deferred — run from a non-worktree checkout.)
