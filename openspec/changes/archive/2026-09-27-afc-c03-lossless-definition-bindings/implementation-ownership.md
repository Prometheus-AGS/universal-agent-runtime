# C03 implementation ownership

Production work is ordered because later partitions consume types and invariants from earlier ones. Each path has one active writer.

| Order | Owner | Files | OpenSpec tasks |
|---|---|---|---|
| 1 | Profile/schema owner | `docs/agents/collaboration/v0.1.0-draft.2/**` | 2.1-2.3 |
| 2 | Domain/validation owner | `src/uar/domain/collaboration.rs`, `src/uar/compiler/collaboration/validation.rs`, and `validation/{schema,schema_registry,diagnostics,authority,canonical,graph}.rs` | 3.1-3.4 |
| 3 | Legacy/projection owner | `src/uar/compiler/{ir,parser,to_artifact,storage,conformance}.rs`, collaboration `validation/projection.rs`, and collaboration `migration.rs` | 4.1-4.4 |
| 4 | Binding/runtime owner | collaboration `bindings`, `grants`, `storage`, `service`, `export`, runtime skill/binding/admission seams, and `server.rs` | 5.1-5.4, 6.1-6.2 |
| 5 | Administration owner | collaboration REST, MCP, administration manifest, capability discovery, and run binding-selector adapters | 6.3-6.4 |
| 6 | Integration-fixture owner | Dedicated C03 integration fixtures and the single completed-phase production-path gate | 7.1-7.4 |

The current authoring consumer work in the mini and full skill packs is separately owned and consumes the immutable draft.2 checkpoint. Review, verification, audit, and integration-checker roles remain dormant until all production partitions and both consumer payloads are complete.

