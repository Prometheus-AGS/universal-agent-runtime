# Final fixture scope review

Date: 2026-10-31
Phase: `production-readiness-476-baseline`, acceptance pending.
Reviewer: isolated native Codex subagent `final_fixture_scope_critic`.
Isolation: final diff, requirements, and production source only; no producer history or handoffs supplied. This is a native same-family review, not cross-family certification.

## Scope and result

No actionable findings in the inspected fixture diff. APPROVE for the bounded source-review scope.

Reviewed `git diff --staged --stat` (empty) and `git diff -- tests/` (14 files, 123 additions and 49 removals), plus the rust-test-baseline spec delta. Traced the affected constructor contracts through `src/uar/rag/embeddings/{mod,swappable}.rs`, `src/uar/rag/ingest.rs`, both matcher implementations, MCP preflight/runtime, provider feature export, and the approval-recovery entrypoint.

- All embedding changes wrap the same concrete backend or existing trait-object instance in `SwappableEmbeddingBackend`; dimensions, model IDs, unavailable-backend reasons, matcher thresholds, chunk strategies, and behavior assertions are retained. `BackendSlot` captures those metadata values and delegates embedding to the retained backend. No swap is introduced by these fixtures.
- The two `mcp_projection.rs` calls add `None` for optional request metadata. `McpPreflight::call_tool` continues to require a projected tool and matching prepared server; `PreparedMcpServer` retains definition/catalog validation before dispatch. This argument adds no authority or connection recipe.
- The memory atomic-deadline block is unchanged inside its new existing-feature guard. It still constructs `InMemoryProvider`, calls the original `stale_deadline_intents` assertions, and only then writes its receipt. The helper still asserts late intents lose, stored state becomes expired, no decision survives, and a repeated transition loses.
- `qualify()` checks `in-memory-backend` before its existing Postgres precondition, evidence-root lookup, directory creation, or scenario launches. The parent entrypoint calls it after the existing child-mode dispatcher; child behavior was not changed. `deadline_profiles` has no other caller in the inspected tests.
- The spec explicitly retains compile-only treatment of historical qualification and preserves the memory control under `server-full,postgres-backend,in-memory-backend`. The diff deletes no test, assertion, or fixture target, adds no ignore, and changes no production feature/default/security/budget setting.

## Evidence and limits

Only source inspection commands were run: `git diff`, `rg`, `cat`, and `sed` (with bounded output selection). No Cargo, rustc, test, build, lint, format, compiler probe, acceptance script, or external action was run. The verification lock remains closed for this reviewer. No runtime receipt was produced or credited.

The uncomfortable limit: constructor reasoning does not prove that the integration targets compile, that historical qualification succeeds, or that an installed runtime passes qualification. Historical fixtures still contain credential/authentication setup outside this diff; this review does not authorize their execution or certify their suitability for current installed qualification. Issue #465 owns that qualification scope.

Only this review document was added. No code, guards, defaults, tests, or production configuration were changed by the reviewer. No automated sycophancy tool was run; manual screening removed speculative findings and unsupported execution claims.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE — bounded source review; compilation and runtime behavior remain unverified here.
