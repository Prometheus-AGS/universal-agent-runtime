# Offline build design

## Inventory and sourcing

The pre-change build fetched `https://models.dev/api.json`, Swagger UI fetched
GitHub assets, and six direct dependencies used Git sources. Runtime network
access (LLM providers, MCP, document services, tool provisioning) is explicitly
configured application behavior and is not a build input.

Published Git dependencies moved to crates.io: liter-llm 1.9.3, rmcp 1.8.0,
and Kreuzberg 4.10.2. Unpublished surreal-memory, sycophancy-core, and
prometheus_parking_lot sources are reduced, vendored snapshots with immutable
commit and license records. Cargo.lock contains zero Git sources. Swagger UI
uses its official vendored assets feature.

## Versioned data

`catalog/provider_catalog.json` is the reviewed 269-provider merged snapshot.
`catalog/SNAPSHOT.md` records its source, date, SHA-256, and the SHA-256 values
of every packaged BGE model/tokenizer input. `build.rs` copies this file and
does not contain HTTP catalog logic.

Refresh is separated into `scripts/refresh-provider-catalog.mjs` and
`scripts/refresh-local-model.sh`; both print digests and require an explicit
diff review and snapshot metadata update.

## Release archive and verification

`scripts/package-offline-source.sh` creates a 294 MiB compressed archive that
contains recursive source inputs, unpublished sources, and every Cargo registry
crate selected by the lockfile. A local extraction passed
`cargo check --locked --offline --lib --features minimal`.

CI builds two independent archive extractions with a fixed source epoch and
source-path remapping, then compares the release binary and catalog bytes. It
also asserts the unpublished-source license records are present.
