# Upgrade embedded UAR persistence to SurrealDB 4.0

## Why

The embedded persistence dependency is moving from 3.3 to 4.0. Storage compatibility, resource shutdown and owner-scoped query behavior need an explicit contract because a converted store cannot be used for same-volume rollback.

## Scope

Qualify the Linux amd64 OCI API service behind the trusted gateway with its existing shipping feature profile, nonroot runtime and dedicated SurrealKV volume. Retain established API and trust semantics. Align all shipping SDK, types and embedded engine consumers to 4.0.0, including the actual lockfile, toolchain and patch closure. Review documented 4.0 improvements and take the ones that preserve these contracts, with native shutdown the first candidate. The separate restricted PostgreSQL lane retains its current contract.

Remote memory adoption, SurrealDB Agent Memory cloud, shared agent and team memory, remote SurrealDB support promotion, actual RocksDB conversion and other platform certification are follow-up. Reuse existing artifact qualification infrastructure where its assumptions match this child.

## Decision and evidence

This is a draft. An explicit, offline migration mechanism remains to be selected. The uncomfortable case is a baseline whose data cannot be transferred with demonstrable parity. In that case existing-store migration remains blocked and the original stays on 3.3. Record the recovery point and the disposition of later writes before any cutover. Native upgrade behavior is version-specific and requires exact-artifact evidence.

- [SurrealDB 4.0 release notes](https://surrealdb.com/releases/4.0)
- [SurrealDB Rust SDK requirements](https://surrealdb.com/docs/reference/rust)
- [SurrealDB 3.x to 4.x migration guidance](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x)

## Recovery continuation — 2026-11-06

The proposal above is recovered historical intent. The operator adopted the embedded-only contract and VLBV. The implemented offline method is the typed 4.0 SDK upgrade of a private copy, documented in `docs/operations/embedded-surrealdb-4-upgrade.md`; ordinary startup never opts in. T1–T5 implementation is recovered; qualification is pending in the new isolated KBD record. Historical results are not fresh acceptance.
