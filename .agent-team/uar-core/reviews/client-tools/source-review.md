# Caller-tools isolated source review — 2026-10-22

Disposition: source review completed with one demonstrated HIGH defect corrected during this pass. Certification withheld: no compiler, database, runtime, protocol or cleanup receipts exist for this candidate. The operator verification gate remains CLOSED.

## Mandate and isolation

Reviewer: Codex GPT-6-family native subagent `/root/tools_source_review`. Producer identities were not supplied as a complete model roster. This is a fresh task context receiving requirements, final inventory and source artifacts, without producer generation conversation or a preferred verdict. It is a same-family fallback, weaker than a different-family adversarial judge. No additional judge was dispatched. The review is bounded to caller-tools changes, particularly adapter signatures, durable claim/replay and settled producer cleanup; it is not an exhaustive review of the historical baseline in the 99-file inventory.

Read the root AGENTS.md, `uar-project-review`, Rust workspace/best-practice skills, team tool policy/repository map, OpenSpec proposal/design/tasks/spec deltas, supplied source inventory and relevant writer handoffs. The waypoint names `uar-post-lifecycle-verification-readiness`, Execute, revision 157, plan revision 91, implementation 144/166. Its incomplete metric prevents treating the scoped source-authoring inventory as whole-phase acceptance.

Source basis: inventory identifies HEAD `ee4826f000916e2025207efdda6da33db2a93f7f` plus listed dirty/untracked files. Cumulative tracked changes were inspected with `git diff HEAD -- ...`; new shared ingress, DTO, provider storage and adapter extension files were read directly. The lead corrected the defect below during the review, so the inventory's recorded pre-correction hash for `src/llm/anthropic_driver.rs` no longer identifies the reviewed corrected bytes. Refresh the inventory before requesting any gate. No hash-verification command was run.

## Demonstrated finding

### HIGH CT-SR-01 — Invalid native Anthropic system-block initializer (corrected in source)

Original evidence: `src/llm/anthropic_driver.rs:178-183` initialized `SystemBlock` with `strict: func.get("strict").and_then(serde_json::Value::as_bool)`. In `convert_messages`, the relevant bindings are `messages`, `msg`, `role` and `text`; no `func` exists. `SystemBlock` in `src/llm/anthropic_types.rs:37-47` declares only `block_type`, `text`, and `cache_control`. Thus the added line simultaneously referenced an undefined name and an absent struct field. This is a demonstrated source inconsistency, not a compiler result.

Trigger: compile the native Anthropic driver containing this initializer. Consequence: the feature cannot compile through that module. Proposed correction: remove the `strict` field from system text blocks and retain it only on `ToolDef`.

The lead made that correction. A subsequent source read showed `anthropic_driver.rs:178-182` contains only the three declared fields; the valid `ToolDef.strict` mapping remains in `convert_tools`. Compiler confirmation remains unverified. The reviewer made no production edit.

## Focused source observations and limits

- `src/uar/api/client_tools.rs:40-59` loads the scoped durable row and acknowledges `source_settled` only after `await_settled_run_status` returns Done. `prepare` requires that acknowledgment before claiming Pending, binds protocol, visible history and schemas, and uses CAS; Claimed is not reacquired. `complete` persists adapter projections through CAS. These are observed branches, not evidence that database concurrency or replay works in a running service.
- `src/uar/persistence/client_tools.rs` retains owner/tenant/agent/run/session, graph state, budget snapshot and canonical/visible transcripts; transition validation limits the acknowledgment to its flag and preserves immutable fields. PostgreSQL and Surreal implementations use conditional updates against the prior serialized row. Actual transaction behavior, migrations and restart durability need the separately authorized real-store gate.
- `src/uar/runtime/manager.rs:8929-8956` awaits retained ordinary/actor producers and the graph root before consulting settled status; errors change the live run to Error. `src/uar/api/openai/routes.rs`, `src/server.rs` bound Anthropic projection, ACP and A2A adapters call settlement/acknowledgment helpers. AG-UI uses its durable run binding and handoff sealing. No physical cleanup claim is certified from these calls alone.
- `src/llm/orchestrator.rs` withholds raw caller metadata, freezes ownership through the resolved turn, settles host tools and writes the durable pending row before `ClientToolsRequested`; `src/uar/runtime/turn/resolved.rs` rejects advertised-name collisions and carries explicit controls. Graph resume state and cumulative budget restoration are present in the manager/accounting path. Neither complete provider behavior nor budget conservation was executed.
- AG-UI/A2A deliberately store admission receipts as Completed and reconnect to the admitted run; OpenAI/Anthropic/ACP save their terminal projection. This distinction is in the design and was not treated as a defect merely because completion timing differs.

No further demonstrated blocking defect emerged from this focused pass. That statement does not imply all inventory files were exhaustively reviewed. A different-family judge remains pending; it cannot be inferred from native subagent isolation. Release 1.2.1 publication remains separately unauthorized, and this review performed no release/tag/index mutation.

## Verification and screening record

All shell commands were source inspection (`cat`, `sed`, `nl`, `rg`, `head`, and `git diff HEAD`); all returned source text or diff text. No Cargo command, test, lint, format check, typecheck, compiler/import probe, build, packaging check, `git diff --check`, acceptance script or external gate ran. No runtime receipt was supplied. Standard review Cargo checks are deferred under the explicit source-only authorization; compilation and execution remain unverified.

Machine sycophancy detection was available and invoked on the review disposition using strict technical-review mode. Receipt timestamp: `2026-10-22T15:55:48.577332Z`; score `0.0803571417927742`; classification S-03 high; `correction_mandatory: false`; detect-only, no automatic correction applied. The generic rationale claimed no risks/gaps despite the submitted disposition explicitly withholding compilation/database/restart/wire/cleanup certification. Manual screening retained those concrete limits and this report's corrected-defect evidence instead of asserting the detector established correctness. The screening is not product verification.

## Handoff

Only reviewer-authored file: `.agent-team/uar-core/reviews/client-tools/source-review.md`. No unrequested production changes or guards were added. Existing guards inspected here trace to the declared caller-result authority, duplicate/replay, budget and producer-cleanup boundaries. The lead owns refreshing exact source identity, reconciling the waypoint/full phase ledger, and requesting exact production gate authorization. No gate has been opened by this report.
