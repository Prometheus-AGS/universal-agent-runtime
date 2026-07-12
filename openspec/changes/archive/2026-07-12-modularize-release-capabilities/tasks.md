## 1. Dependency boundaries
- [x] 1.1 Measure current minimal/full dependency and binary baselines.
- [x] 1.2 Make SurrealDB/Postgres/in-memory backends genuine dependency switches with compile-time validation.
- [x] 1.3 Gate local embeddings/models, Cedar, sycophancy, document intelligence, telemetry, A2A, WASM sandbox and admin UI.
## 2. Feature products
- [x] 2.1 Add `minimal`, `server-full`, `desktop-full` bundles matching the support matrix.
- [x] 2.2 Move `model-build` to `tools/uar-model-builder` maintainer command.
- [x] 2.3 Remove `memory-palace` or move it to an extension crate until integrated/certified.
## 3. Verify
- [x] 3.1 Build/test every stable bundle on supported OSes.
- [x] 3.2 Assert disabled capabilities disappear from dependency graph/binary/API.
- [x] 3.3 Update docs and validate OpenSpec.
