# Approved embedding-gate principal repair — 2026-10-21

## Trigger

The lead-owned approved embedding rerun compiled the fixture and then failed its
sole test during setup: `save owner-scoped knowledge base` returned
`knowledge requires a verified principal or the explicit anonymous boundary`.
The prior SHA-256 encoding repair compiled; this was the next observed blocker.

## Changed path and rationale

`tests/embedding_lifecycle_failover.rs` now obtains its fixture owner through
the production `authenticate_bearer_context` boundary. It initializes the
fixture verifier, signs a fixture-only JWT, verifies that JWT, and derives the
principal storage key with `UserContext::session_owner_key`. The fixture passes
that derived key through knowledge-base creation, durable document tracking,
ingestion, durable lookup, and all compatible and incompatible query paths.

The router search helper receives a clone of that verified context. The fixture
does not use the anonymous boundary, does not construct a synthetic
`UserContext`, and does not change a production authorization guard. This
matches the verified-principal convention used by `tests/durable_ingestion.rs`.

## Source completeness and verification limit

This repair owns only the test source above and this handoff document. No
production files, dependencies, warning-only cleanup, or first-party vendor
exports changed. No Cargo command, test, build, formatter, lint, compiler
probe, or review gate was run by this writer. The real local-model lifecycle,
principal-bound retrieval, and fixture cleanup remain unverified until the lead
records the renewed inventory and runs the exact already-approved embedding
command.
