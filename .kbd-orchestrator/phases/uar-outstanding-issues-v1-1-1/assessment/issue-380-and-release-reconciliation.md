# Issue 380 and release issue reconciliation

Assessment date: 2026-10-15. Phase: `uar-outstanding-issues-v1-1-1`.

This artifact records source inspection and existing evidence. It does not execute verification, change production code, close GitHub issues, or resume the canceled Dot qualification. The uncomfortable thing: substantial repairs and historical passing fixtures exist, but they do not establish every requirement in #380 or certify the current source revision.

## Scope and evidence

- [#380](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/380): OPEN; updated 2026-10-13T11:12:14Z. Four runtime/credential findings and their exact acceptance requirements.
- [#379](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379): OPEN; updated 2026-10-15T20:06:34Z. Release acceptance and supplementary amd64 evidence already posted.
- [#383](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/383): OPEN; updated 2026-10-15T08:55:55Z. Unchecked release checklist and no issue comments despite completed release records.
- [#382](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/382): OPEN; updated 2026-10-14T13:40:09Z. Cross-repository dependency tracker with bounded release-lane criteria and explicit exclusions.
- #385 diagnosis belongs to the other assessment owner. The completed v1.1.0 acceptance is historical evidence, not proof that its subsequently reported deployment failure is resolved.

GitHub issue bodies/comments were read using `gh issue view ... --json ...`. Repository inspection used `cat`, `sed`, `rg`, and file listings only. No tests, compiler, lint, format, builds, review gates, or deployment actions ran. No private infrastructure or credential values were printed.

## #380 item 1: explicit provider and manager completion

### Implemented

`src/llm/orchestrator.rs:2438–2463` tracks normalized terminal evidence per attempt and rejects EOF without it with `provider_stream_incomplete`. The check no longer depends on graph mode. The existing history selector at approximately line 1839 still uses graph messages in graph mode and captured canonical history in ordinary mode. Tool dispatch follows the terminal check. Existing retry/fallback behavior is not replaced by this repair.

`docs/operations/provider-stream-completion.md` documents protocol terminals, framing, clean EOF, interruption, tool dispatch, and the distinction between a stream-closing `RunDone` and successful execution.

### Confirmed remaining implementation

The ordinary manager path in `src/uar/runtime/manager.rs` still breaks on exhausted orchestrator stream around line 7041. Its event match handles errors but has no dedicated successful `Done` observation; unmatched events fall through around line 7319. It sets `RunStatus::Done` around line 7523 when neither cancellation nor another error was recorded. This does not implement #380's separately requested manager completion guard.

Add successful-orchestrator-completion tracking in that existing path, and classify stream exhaustion without it as failure before status/history settlement. Preserve cancellation and cleanup-error precedence. Do not infer success from `RuntimeStep::Finished` or an API stream-close envelope. The orchestrator already prevents the common truncated-provider path; the missing manager guard is a separate explicit requirement, not evidence of a reproduced live failure.

The history-selection boolean remains named `require_terminal` even though terminal validation is unconditional. A narrowly scoped rename would clarify the requested separation; rewriting history selection is unnecessary.

### Acceptance coverage to reconcile or add

The existing `tests/stream_terminal_contract.rs` runs 83 protocol/scenario rows through the legacy orchestrator and local/shared durable graph manager. Its cases include valid, bytewise split, empty, missing terminal, trailing partial event, malformed, invalid UTF-8, valid tool loop, unterminated tool request, and cancellation.

It does not enter the ordinary full-harness task API. Add the issue's exact ordinary manager/full-harness failed projection, with empty EOF, metadata-only EOF, text EOF, partial tool JSON EOF, and completed tool call EOF, plus explicit error/cancel and valid text/tool-loop controls. Repeat the relevant terminal assertions through fallback and preserve ordinary versus graph history selection.

Reuse existing neighboring coverage rather than declaring all regression support absent: `tests/model_path_resiliency.rs` already covers fallback, metadata-only stalls, partial-output timeout without retry, interrupted history, and partial tool calls; `tests/context_history_integrity.rs` covers protected history and fallback preparation. These are not identical to the requested EOF/terminal cases. `tests/bauar_full_harness_cursor.rs` already provides a real router/executor fixture but targets idempotent admission and cursor/reconnect behavior. No exact combined truncated-Anthropic/full-harness plus failed-MCP receipt fixture was located in the inspected targets.

## #380 item 2: truthful MCP outcomes

### Implemented

`src/mcp/registry.rs:2606` derives metrics success from the typed MCP result's `is_error`, and around line 2724 classifies application failure before returning serialized data. Typed `McpToolFailure` retains SDK-represented content and certainty. Ordinary and graph orchestration use the explicit failure, preserve model-visible diagnostics, and record failed admission/canonical outcomes. Transport recovery does not grant replay authority.

`docs/operations/truthful-tool-outcomes.md` describes `succeeded`, `application_failed`, `transport_failed`, and `outcome_unknown`, preserved content, secret projection, model/display limits, canonical immutability, dependency settlement, and no replay.

The compatibility entry points retain `Result<Value>` and successful JSON shape. Application errors now return a typed error rather than `Ok` containing `isError:true`. This compatibility distinction is documented; assess external callers before making a broader API promise. No new core MCP outcome rewrite is justified by this inspection.

### Acceptance coverage to reconcile or add

The existing whole target `tests/truthful_tool_outcomes.rs` has 65 cases spanning direct/scoped calls, actual runtime processes, persisted receipts, failed application results, unknown effects, restart/readback, and dependency settlement. Its peer at `tests/fixtures/truthful_tool_outcome_mcp.py` always supplies `isError` as true or false, with content, structured content, and metadata.

The exact omitted-`isError` case is absent from this peer. Add it. Map existing cases before adding tests for explicit JSON-RPC failure, timeout, no reconnect on application failure, later distinct recovery without rewriting the original receipt, and coincidental `isError` in a native tool result. Those exact assertions were not established by this bounded inspection; do not claim repository-wide absence merely from the main fixture inventory. Maintain the existing no-replay and payload assertions.

## #380 item 3: native Anthropic framing

### Implemented

`src/llm/anthropic_driver.rs:375–464` uses the actual response `bytes_stream()` and an incremental `liter_llm::SseDecoder`. It preserves split UTF-8, parses complete frames, rejects malformed JSON and mismatched event names, waits for a valid `message_stop`, validates trailing framing/EOF, and emits one successful normalized `Done`. Normalized provider errors terminate processing. Usage handling is preserved.

The decoder in `vendor/git/liter-llm/crates/liter-llm/src/http/streaming.rs` retains line/event state and pending bytes, joins data fields with newlines, ignores comments and unknown SSE fields, removes one optional field-value space, and rejects incomplete EOF. `MAX_BUFFER_BYTES` is 1 MiB and overflow is an explicit error.

### Confirmed remaining implementation and policy

`SseDecoder::next_event` around line 375 searches only for LF and uses `trim_end_matches('\r')`. It does not recognize a lone CR as a line separator, although #380 explicitly requires LF, CRLF, and CR boundaries. Correct this framing behavior through the smallest appropriate native adapter/decoder change while preserving the approved dependency pin and recording effective vendor changes if the shared decoder is edited.

Unknown event policy needs explicit resolution. `src/llm/anthropic_types.rs` has a closed `StreamEvent` enum including `ping`; it has no unknown/other variant. The native driver's failed deserialization path rejects every unknown event type. Thus ping is already handled and unknown SSE fields are already ignored; neither establishes forward-compatible handling of unknown Anthropic JSON event types. Document which types may be ignored and implement that decision without swallowing malformed known events. This assessment does not assert that every unknown event must be accepted or invent an upstream event taxonomy.

Document the exact retained-byte limit rather than only saying an existing limit is reused.

### Acceptance coverage to reconcile or add

The current matrix's `split` case sends bytes one at a time through real HTTP. It is valuable production-path coverage, but not the issue's exhaustive every-two-chunk-split normalized-sequence equivalence check. Add or locate explicit whole-buffer/every-split/bytewise/fixed-partition comparisons, CR-only separators, split CRLF, multiline data, comments, optional field whitespace, overflow, and the chosen unknown-event policy. Preserve malformed known JSON, invalid UTF-8, missing terminal, unterminated event, and one-terminal-outcome controls. The combined ordinary full-harness case belongs with item 1.

## #380 item 4: credential ownership and migration

### Implemented

- `src/uar/security/claims.rs:187–258` defines a typed verified principal with issuer, tenant, and subject and unambiguous length-framed storage identity. Synthetic/inconsistent contexts fail.
- `src/uar/api/credentials.rs` captures the verified owner for compatible masked CRUD and exposes operator-assisted legacy mapping.
- `src/uar/security/credentials/principal_store.rs` provides disjoint principal storage, structured composite keys, exact configured mappings, atomic ciphertext copy and journal, preserved timestamps, destination-conflict rejection, and idempotent receipts that do not recreate deleted destinations.
- `src/uar/security/credentials/resolver.rs` namespaces session, agent, and user lookups. Store/decryption errors propagate; they do not silently become ordinary misses.
- `migrations/20261011000000_principal_credentials.sql` is additive; `migrations/surrealdb/principal_credentials.surql` provides the corresponding schema. Legacy rows stay quarantined for ordinary principal lookup.
- `docs/operations/principal-provider-credentials.md` documents mapping configuration, owner verification, request/response shapes, errors, repeated mapping, and quarantine.

### Confirmed remaining implementation

`CredentialPrincipal::from_verified_context` uses constant `urn:uar:local:jwt` for issuerless verified tokens and constant `urn:uar:local:host` for local-host identity. Missing tenant maps to `local`. There is no installation-specific trusted authority input or explicit shared-mode missing-tenant rejection in this construction.

`CredentialResolver::resolve_with_context` unconditionally falls through to `resolve_system` after principal misses. `manager.rs:597–630` resolves system for absent principal and preserves the existing configured key on ordinary miss. No explicit shared-mode host-owned allow/deny fallback policy is present in these paths. The existing shared credential fixture intentionally demonstrates house fallback, so the new policy must distinguish compatibility from explicit shared authorization.

Add minimal credential-service policy, not a new global tenancy subsystem:

1. Explicit single-owner compatibility mode preserves intended local behavior and binds local authority to stable trusted installation/auth configuration.
2. Shared mode rejects missing tenant/authority for principal operations. Missing identity is not permission to use system credentials.
3. Shared stored-system and env/config fallback are disabled unless the trusted host explicitly permits them. Gate both through the same policy. Storage, identity, decryption, and migration errors always stop resolution.
4. Keep system credentials installation-owned and public credential body/path/masked shapes compatible.

These are proposed semantics. No configuration key names are asserted to exist; select names and defaults in the phase plan after inventorying host configuration construction. Do not derive installation identity from current login, first requester, arbitrary request strings, or an empty tenant.

### Migration operations still required

Provide a non-secret dry-run inventory with mapping/conflict counts and explicit source-to-destination reporting. The existing procedure instructs the operator to identify legacy rows but provides no dry-run operation. Complete backup, legacy-writer quiescence, verification, rollback, and shared-mode enablement instructions. Keep old rows/journal and explain that overwritten historical collisions cannot be reconstructed. An old binary with subject-only credential lookup is not a safe shared rollback.

Extend acceptance for missing tenant, installation authority, explicitly denied/allowed shared fallback, inventory and cutover verification. Reuse the existing tests for same-subject tenant/issuer isolation, rotation/deletion, scope precedence, forged context denial, mapping replay/conflicts, timestamp/ciphertext preservation, and real stores. In-memory integration coverage currently rejects synthetic ownership; the full memory storage/migration contract needs mapping to tests or additions rather than assuming durable-backend tests cover it.

## Historical evidence and exact existing commands

The archived Dot phase's `closeout.json` and `reflection.md` state 24 production items implemented, 15 remaining verification tasks canceled, final certification waived by the operator, temporary qualification infrastructure removed, and no successor work scheduled. Preserve that decision. New bounded #380 work is a separate scope; do not reactivate Dot's canceled acceptance program.

Historical accepted records include:

- `evidence/credentials/20261011T223312Z-production-boundary.json`: command exit 0; four passed, zero failed/ignored; real local SurrealKV, shared Surreal and PostgreSQL credential coverage. Broader phase/release and independent review were deferred.
- Archived `execution.md` around line 467: whole MCP outcome target exit 0, 65 cases complete. No PostgreSQL receipt roundtrip, genuine inference, or final release claim.
- Archived `execution.md` around line 524: stream target exit 0, 83 rows/249 boundaries. Around line 614 it records a later affected-stream rerun after resume changes. Controlled protocol evidence, not genuine provider inference or every alias.

These records bind their own source snapshots. This assessment did not rerun them or prove current-source equivalence.

Existing commands, copied from source/evidence, **not executed**:

```bash
cargo test --locked --no-default-features --features server-full,postgres-backend --test credentials_api_integration_test -- --include-ignored --test-threads=1
```

Requires isolated `UAR_CREDENTIAL_TEST_POSTGRES_URL`, `UAR_CREDENTIAL_TEST_SURREAL_URL`, `UAR_CREDENTIAL_TEST_SURREAL_USER`, and `UAR_CREDENTIAL_TEST_SURREAL_PASS`.

```bash
UAR_DOT04_EVIDENCE_ROOT="$PRIVATE_EVIDENCE_DIRECTORY" cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test truthful_tool_outcomes -- --test-threads=1 --nocapture
```

Requires a private `UAR_DOT04_INFRA_FILE` for isolated durable shared Surreal infrastructure.

```bash
env -u BEDROCK_BASE_URL -u BEDROCK_CROSS_REGION \
VERTEXAI_PROJECT=dot05-fixture \
VERTEXAI_LOCATION=us-central1 \
UAR_DOT05_INFRA_FILE=/absolute/private/infrastructure.json \
UAR_DOT05_NAMESPACE=dot05_fresh_namespace \
cargo test --locked --no-default-features --features server-full \
  --test stream_terminal_contract -- --nocapture --test-threads=1
```

The phase plan must reconcile these with the new bounded cases and actual available infrastructure. Old temporary paths are not current infrastructure. #380's older checklist requests early checks and CI/verification evidence; current AGENTS rules supersede that schedule. All non-deployment verification stays local and starts only after the complete new phase implementation inventory is recorded. No early gate or full canceled Dot program is implied by listing these commands.

## #379 and #383: delivered release, stale issue bookkeeping

The completed `uar-1-1-0-kubo-conformance` phase records all nine original #379 items and four follow-ups met. Immutable source/tag is `d815ef72ac7d2c57a9bd18b23fcc58ffb224f326`, `v1.1.0`; published multiarch digest is `sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b`.

[Release](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.1.0), [published acceptance comment](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379#issuecomment-6199871487), and [amd64 supplement](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379#issuecomment-6201219322) provide exact receipts. Arm64 and emulated amd64 full acceptance passed against published images. Native amd64 hardware, launch-to-first-capture gap, actual infrastructure rollouts, and review-independence limits remain explicit. They were not deferred Kubo implementation requirements.

#383's unchecked list and lack of a release receipt are stale bookkeeping relative to that record. Reconcile it to the existing evidence, preserving #385 as a later reported blocker with its own causal assessment. Do not move the immutable tag or claim that old acceptance proves the new report resolved. External comments/closure remain lead-owned actions; this artifact makes none.

## #382: bounded external tracker

#382 explicitly tracks cross-repository dependencies, not a claim that every subsystem must be built before a pilot. Its completion requires a chosen release lane, verified owning issue links or explicit exclusions, pinned consumer/provider contracts, and a bounded integration record identifying passed, blocked, and untested cases.

Existing references include skill-system #184/#185, skills-mini #59, surreal-memory-server #51/#52, and the separate LibreFang product epic. Some client/delegation/decision/inference drafts remain blocked on repository write access or disabled Issues. Source inclusion does not resolve those external owners or prove deployment.

Its non-goals explicitly exclude production deployment, credential changes, cross-tenant raw trace export, model/harness rewrites, reopening canceled acceptance, blanket certification, automatic purchases, and label-only isolation claims. Assessment should keep this issue as tracking/external scope unless the operator selects a concrete lane. It must not automatically expand #380/#385 into GPU, fleet, optional inference engine, or whole-estate qualification work.

## Handoff and evidence limits

Owned change: this assessment document only. No product files or issue state changed. No unrequested guards or dependencies were added. Every proposed guard traces to #380's explicit requirements and existing credential/provider trust boundaries. Source gaps have high confidence where the exact branch was inspected; fixture absence claims are scoped to inspected targets and require requirement-to-existing-test mapping before adding duplicate cases. Runtime outcomes, current-source acceptance, fresh backend availability, new migration cutover, and #385 resolution remain unverified.
