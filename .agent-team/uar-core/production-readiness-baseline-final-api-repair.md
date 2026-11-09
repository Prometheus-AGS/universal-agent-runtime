# Final fixture API repair handoff

Phase: `production-readiness-476-baseline`, Execute. Verification lock: **CLOSED**.
Source base supplied by lead: `9ed71e93cb2b3e130ff2c042a04b6ed3bcbd51ca`.
The following changes are local and uncommitted. Other writers share the checkout;
their files and edits were preserved.

## Delivery against assignment

- `tests/skill_activation_runtime.rs`: wrap the original backend at four matcher
  constructor sites in `Arc<SwappableEmbeddingBackend>`.
- `tests/prompt_assembly.rs`: wrap the original backend at one matcher site.
- `tests/uar_integration.rs`: wrap the existing unavailable-backend helper result
  at both matcher sites, retaining its original helper contract.
- `tests/kb_embedding_space.rs`: wrap the original hashing backend at two
  ingestion sites and one matcher site. Models, dimensions, chunking strategies,
  persisted space mismatch behavior and all assertions remain unchanged.
- `tests/mcp_projection.rs`: wrap the original backend at five matcher sites;
  supply `None` at the two prepared-projection `call_tool` sites. The actual
  `src/mcp/preflight.rs` signature accepts optional `RequestMetaObject` and forwards
  it to the existing binding. No metadata, authority or scope is fabricated.
- This handoff is the only documentation file written by this teammate.

Fifteen constructor sites and two missing-argument sites were repaired. The
recorded diagnostics are in `evidence/full-inventory-attempt-002.txt` under the
active phase. Actual `SwappableEmbeddingBackend`, `VectorMatcher`, `IngestService`
and prepared MCP call signatures were inspected. No production source, dependency,
test assertion, skip attribute or execution behavior was changed deliberately.

## Same-class findings outside ownership

Source-only inspection of constructor uses under `tests/` found these additional
obsolete arguments. Reported to lead before any edit; these files remain untouched
by this teammate:

- `tests/test_chat_completion.rs:41`: matcher helper returns `Arc<dyn EmbeddingBackend>`.
- `tests/bauar_receipt_projection.rs:97`: matcher receives `Arc<UnavailableEmbeddingBackend>`.
- `tests/mcp_preapproved_tools.rs:189`: matcher receives `Arc<dyn EmbeddingBackend>`.
- `tests/test_provider_resolution.rs:25`: matcher receives `Arc<dyn EmbeddingBackend>`.
- `tests/truthful_tool_outcomes.rs:574`: matcher receives `Arc<UnavailableEmbeddingBackend>`.
- `tests/model_path_resiliency.rs:305`: matcher receives `Arc<dyn EmbeddingBackend>`.
- `tests/durable_ingestion.rs:64`: matcher receives `Arc<UnusedEmbedding>`.
- `tests/kb_chunk_strategy_ingest.rs:61`: ingester receives `Arc<StubBackend>`.

The similar-looking calls in `tests/tool_call_protocol.rs`,
`tests/ingestion_admission.rs` and `tests/embedding_lifecycle_failover.rs` already
receive swappable handles and were left intact. This scan is source evidence, not
a claim that every target compiles or that no other failure class exists.

## Commands and remaining uncertainty

Commands were classified as source inspection (`cat`, `tail`, `sed`, `rg`, plain
`git diff -- <owned paths>`) or authorized implementation/documentation writing
(`apply_patch`, a bounded Python text-edit script). Inspected the current waypoint,
latest plan and corrected compile-only inventory scope, recorded diagnostics,
current APIs and constructor uses. Skills loaded earlier in this session remain
applicable: `prometheus-rust-workspace` and `rust-best-practices`.

No Cargo, rustc, test, check, formatter, lint, typecheck, compiler probe, build,
`git diff --check`, review gate, acceptance script or external/paid operation was
run or delegated. All compilation, warning, formatting and runtime outcomes remain
unverified. No new acceptance credit is claimed.

Lead must resolve the reported eight files, collect all cohort handoffs and renew
the exact-source implementation inventory before any verification. The awkward
remaining fact is that both historical broad attempts stopped at compilation;
neither establishes any target's runtime behavior. The corrected final inventory
is compile-only for integration targets; installed qualification belongs to #465.

No new guards or unrequested additions were introduced. No unresolved design
decision exists within the five-file assignment.
