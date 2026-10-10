## Context

The approved first-delivery contract and source decision in the initiative define exactly two no-effect tasks and a human decision. C09 catalog CAS already owns task/attempt/artifact identities, reservation accounting, exclusive dispatch and recovery. The ordinary actor host remains the executor.

## Goals / Non-Goals

Persist definition, interpretation, compiled plan, role/member and binding identities before admission; commit an original attempt link in the same admission CAS. Progress using successful authoritative outcomes and validated artifacts, holding unknown usage reservations. Decisions finalize only an internal draft. No DAG, timer, generic signal, retry, compensation, external effect or second scheduler.

## Decisions

Use additive catalog maps, rather than another store, so workflow transitions and C09 links share one durable generation. Start creates both canonical task identities with pinned role assignments. C09 admission accepts workflow-owned tasks only from the workflow controller and atomically links the attempt. The existing runtime drain advances known workflow outcomes before dispatch; completed jobs notify that same drain. Explicit recovery first uses C09 live-producer checks, then advances persisted outcomes. Polling reads never dispatch.

Use the draft.2 extension mechanism with a closed version 1.0.0 schema and required capability in workflow/package. Compile only classify/draft and exact selectors; unsupported fields produce diagnostics. Pin the complete document and plan. Narrow the ordinary context builder for workflow attempts; the bound run policy selects no skills, tools, MCP, knowledge or memory. Current binding/member authority is checked again on admission, context selection and decision.

All routes share authenticated owner/workspace extraction. Start/control command receipts bind full requests; decisions require current revision, wait identity and presented artifact digest. Cancellation intent commits before existing task cancellation, and no dependent admission is allowed after intent. Unknown execution remains reconciling. Accounting is projected from C09 rather than copied as another ledger.

## Risks / Trade-offs

Whole-catalog CAS cost grows with state → measure progression/storage overhead at the complete delivery gate; do not claim throughput ahead of measurement. A lost model outcome cannot be recreated safely → retain original attempt and show reconciliation. Current authority can invalidate a pinned run → show the denial without substituting newer definitions. Workflow qualification stays false until accepted real-path receipt; operation-stage capability is explicit.

## Migration Plan

Add serde-default catalog fields and new authenticated routes. Existing packages and team execution retain semantics. Older runtimes must not operate new workflow records. Keep pinned records on rollback; no destructive migration or external rollback is needed. Finish Boss typed IPC/UI/locales and UAR production code before coordinated build and actual-model, restart, exact-decision and negative-boundary operations. No intermediate unit/mock gate.
