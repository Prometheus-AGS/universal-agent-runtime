# Proposal

## Why

Issue #385 reports a deployed chat outage whose model-binding cause is erased, plus unexplained durable-upload and ownership upgrade behavior. Issue #380 retains concrete completion, Anthropic framing, credential-policy and acceptance gaps despite substantial existing implementation; v1.1.1 must repair these without weakening authority or claiming historical receipts cover the new source.

## What Changes

- Add typed, secret-safe binding and durable-store diagnostics while preserving error codes, AG-UI event fields, explicit budgets, liability and #384 timeout inheritance.
- Require observed orchestrator completion in the ordinary manager and adapt the pinned decoder locally for native Anthropic CR/LF/CRLF, initial BOM and well-formed future event types.
- Complete exact real-path MCP outcome fixtures and retain typed failures, receipt immutability and no automatic replay.
- Add operator-owned single-owner/shared credential policy, exact shared fallback grants and credential-only installation identity. Omitted mode retains single-owner compatibility; remote transport does not determine tenancy.
- **BREAKING for local credential operations:** issuerless/local-host credential owners require a stable operator-provisioned installation UUID; fixed-local v2 rows require explicit approved migration. External issuer owners and general session/KB/schedule keys remain unchanged. Shared mode is opt-in and rejects incomplete verified ownership or ungranted house-account fallback.
- Extend atomic credential migration with fixed-local sources and a trusted-host-only non-secret preview; document backup, quiescence, cutover and safe rollback.
- Document authenticated KB reupload, legacy retention, qualified team recovery and missing evidence. No privileged KB import, forced takeover, blind durability attestation or bulk skill-pack edit.
- Restrict the ownerless compiler REST subtree on both aliases when ingress is remote or credential policy is shared. Keep local single-owner authoring, disclose the profile limit and defer full owner-scoped authoring under open #387.
- Publish immutable v1.1.1 only after the full implementation boundary and local production acceptance; run the full program against both published architecture digests. Reconcile #379/#383 with existing v1.1.0 evidence and retain #382 external ownership.

## Capabilities

### New Capabilities

None. Reuse the existing capability boundaries.

### Modified Capabilities

- `provider-diagnostic-status`: safe origin categories for run binding, separate from provider catalog status.
- `model-path-resiliency`: manager completion evidence and precise native Anthropic framing/forward-compatibility behavior.
- `tool-call-protocol`: exact MCP application/transport/uncertain outcome preservation and no-replay controls.
- `tenant-isolation`: credential-local installation identity, shared fallback policy and authorized migration/preview.
- `compiler-authoring-isolation`: block ownerless REST authoring in remote and shared profiles until owner-scoped storage and sessions are implemented.
- `dot-durable-budgets`: actionable durable eligibility refusal without discarding explicit budgets or liabilities.
- `dot-durable-ingestion`: safe eligibility diagnostics while preserving durable acceptance.
- `knowledge-rag-product-certification`: supported owner-qualified upgrade reupload and explicit preservation limits.
- `kubo-runtime-conformance`: immutable patch-release evidence, both architecture acceptance and honest issue dispositions.

## Impact

Scope: UAR runtime/provider streaming, credential security/storage/API, persistence eligibility, operator configuration, fixtures, recovery documentation and release records. No frontend redesign, vendor/pin change, new model engine, global identity rewrite or external Dot implementation. Existing supervisor recovery requirements remain in force without a new recovery capability.

Runtime UX gains actionable safe error messages and truthful completion; public credential CRUD remains masked and compatible except the documented local-owner migration. Provider compatibility retains Liter 2.1.1, #384 settings and existing native Anthropic routing; realtime state retains RUN_ERROR code/message and truthful persisted outcomes. `versions.toml` pins SurrealDB 3.3.0.

KBD remains authoritative: this proposal belongs only to phase `uar-outstanding-issues-v1-1-1`; Spec is followed by Plan. Tasks, fixtures and required documentation all belong to one full-phase implementation boundary. Product checks and isolated review are deferred until that inventory is complete; product checks never run in GitHub Actions. No cluster mutation or kill-switch access is authorized by this specification.

The uncomfortable limit: better diagnostics do not prove the reported backing store is durable or recover original KB sources. Actual deployment recovery remains evidence-dependent even when equivalent product acceptance passes.
