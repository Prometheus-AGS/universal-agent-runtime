# Candidate build warning correction — runner-owned packet

Phase: Dot Execute. Source corrections only; no post-correction verification executed.

Observed evidence: `/tmp/uar-dot17-validation-20261013/candidate-attempt-04/candidate_build/candidate-build-4.log` reports 33 library warnings and one linker warning. This packet owns only the following source spans; other teammates own the remaining warnings.

- `src/uar/compiler/collaboration/validation/projection.rs`: removed unused duplicate authority filter. Production manifest and definition validation still call `validation/authority.rs`.
- `src/uar/compiler/collaboration/migration.rs`: retained the existing draft.1 converter, result, and field helper with three narrow `#[expect(dead_code)]` annotations naming their lack of a runtime caller. Root explicitly excluded adjacent migration implementation from this Dot correction.
- `src/uar/compiler/collaboration/team_execution/context.rs`: manual Debug omits selected context and attribution.
- `src/uar/rag/chunking.rs`: production Markdown selection calls the existing identical marker helper instead of duplicating its expression.
- `src/uar/runtime/manager.rs`: removed unused checkpoint-resume convenience wrapper; current protected approval and A2A continuation entry points remain.
- `src/uar/runtime/thread/approvals.rs`: removed unused legacy convenience request wrappers; the real recovery path still calls request_with_continuation.
- `src/uar/runtime/thread/kernel.rs`: removed unused admit_host_tool wrapper; host-owned budget admission remains.
- `src/uar/guardrails.rs`: safe unit-type Debug derive; manual chain Debug exposes screen count only.
- `src/uar/runtime/observer/controller.rs`, `observer/channel.rs`, `schedules/controller.rs`, `followups/controller.rs`: manual Debug omits stores, catalogs, gate configuration, and runtime graphs.
- `src/uar/runtime/team_execution/host.rs`: manual Debug exposes binding revision only; excludes MCP, directory, and admission material.

The uncomfortable finding: `docs/agents/collaboration/v0.1.0-draft.2/compatibility.md:53` promises readable draft.1 migration, but source reference inspection found no caller of migrate_draft1_document. This existing compatibility promise is unverified; annotations do not implement or certify it. It is outside the current Dot plan and is deliberately not expanded in a warning correction.

No new runtime guards, configuration changes, dependency changes, or test additions. No checks, tests, builds, formatters, or review gate ran during this packet. The affected Cargo gate must run only after the entire correction bundle freezes and root rebinds its inventory.
