## 1. Dependency boundaries
- [ ] 1.1 Measure current minimal/full dependency and binary baselines.
- [ ] 1.2 Make SurrealDB/Postgres/in-memory backends genuine dependency switches with compile-time validation.
- [ ] 1.3 Gate local embeddings/models, Cedar, sycophancy, document intelligence, telemetry, A2A, WASM sandbox and admin UI.
## 2. Feature products
- [ ] 2.1 Add `minimal`, `server-full`, `desktop-full` bundles matching the support matrix.
- [ ] 2.2 Move `model-build` to `xtask`/maintainer command.
- [ ] 2.3 Remove `memory-palace` or move it to an extension crate until integrated/certified.
## 3. Verify
- [ ] 3.1 Build/test every stable bundle on supported OSes.
- [ ] 3.2 Assert disabled capabilities disappear from dependency graph/binary/API.
- [ ] 3.3 Update docs and validate OpenSpec.
