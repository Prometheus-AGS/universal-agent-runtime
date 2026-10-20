# FC3 handoff — credential, compiler and recovery receipt consolidation

Date: 2026-10-17. Task: child `2.2` / FC3, Execute.
Writer: assigned `uar-trust-tools`. Operator verification gate: **CLOSED**.

## Delivery and exact ownership

Authored only these assigned files in the shared checkout:

- `scripts/release/uar_1_1_1_credentials.py`
- `scripts/release/uar_1_1_1_compiler.py`
- `scripts/release/uar_1_1_1_recovery.py`
- `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/handoffs/FC3.md`

No runtime/product source, manifest, dependency pin, vendored code, canonical
KBD projection or other writer's file was edited. No publication occurred.
The lead owns task transitions and later integration. Preserve concurrent edits.

## Source-grounded reduction recipe

Before edits, source inspection established these repetitions; none was executed:

1. `migration_cases` ran authority/pagination, copy/journal/status and raw-log
   redaction in one sequential backend flow but emitted three separate ancillary
   labels. It repeated identical malformed-query and If-Match syntax on all
   three stores. Those parser forms can be sampled once while each backend still
   executes its own migration and journal decisions.
2. Compiler `run_group` invoked `_denials` with the entire operation × alias ×
   principal × spoof matrix both before and after each restricted-profile restart.
   The source route inventory in `src/uar/api/compiler.rs::build_router` has 14
   method/path operations; its profile middleware rejects before body extraction.
3. Recovery `_attested` returned upload and legacy observations from the same
   backend runs and cold boots; `run_group` emitted them under separate IDs.
4. The binding adapter's native Anthropic messages case and the credentials
   adapter's OpenAI-backed messages case exercised distinct provider paths. Moving
   only the binding label would lose that distinction.

This is source evidence for the requested reduction, not a reproduced runtime
failure or an estimate of measured execution time. The existing Compass artifact
does not establish Python adapter behavior; exact repository source was used.

## File effects and receipt interface

### Credentials

`REQUIRED_SCENARIOS["credentials"]` retains the nine common IDs and ten migration
IDs per local/Surreal/PostgreSQL backend: 39 labels. Every existing backend flow,
transaction call, cold restart, concurrent replay and lost-response retry remains.

| Removed suffix per backend | Retained destination and evidence |
| --- | --- |
| `preview-authority-pagination` | `legacy-conditional-copy.preview_authority_pagination`: ordinary-user 403, page/inventory counts, ordered complete unique keyset, stale cursor 409. Common malformed query and If-Match syntax executes on local only. |
| `secret-redaction` | `legacy-conditional-copy.secret_redaction`: preview/receipt/raw-log key exclusion and preview ciphertext/hint/free-form reason exclusion. |
| `preview-status-inventory` | `mapping-drift`: race/fixed journal conflicts with 409, missing source with 404, unmapped quarantine; `revision-drift`: fixed source `source_changed`; `destination-conflict`, `lost-response-retry`, `historical-receipt`, `immutable-rotation-deletion-restart`: corresponding final preview statuses. |

Rotation still observes `already_applied_destination_changed`, deletion/cold
replay observes `already_applied_destination_missing`, historical conditioned
replay remains 412, and journal replay never overwrites destination state.
Backend receipts are emitted only after their folded assertions all finish;
a later folded failure cannot leave that backend's retained receipt marked passed.
This conservatively reports the backend's incomplete cases through the existing
failure path rather than claiming partial folded success.

Consumed the written “Native `/v1/messages` fold contract for FC3” in
`handoffs/FC2.md`. Credentials imports FC2's public
`observe_native_messages_policy(context)` and records its return value as
`messages-facade-policy.observations.native_anthropic`. The existing account peer
records path strings alongside account labels; facade success requires the
OpenAI `/v1/chat/completions` path. Its denied foreign tenant requires HTTP 403,
`invalid_request_error` / `credential_source_denied` and zero provider contacts.
FC2 supplies native Anthropic `/v1/messages`, allowed complete stream, exact denial
and zero denied contacts. Both observations finish before the combined label passes.

The exact facade refusal is grounded in
`src/server.rs::authorized_http_model` and `api_messages`; this strengthens the
existing source-policy oracle without changing policy or runtime behavior.

### Compiler

Three IDs remain. `compiler-restart-preserves-existing-resources` is folded into
`compiler-local-single-owner-control`. The local control now passes only after
all restricted-profile attempts, restoration, exact spec/session/report API
readback and restored inline compile/descriptor verification on both aliases.
Full database snapshots still compare specs, reports, sessions and registered
agents across profile changes, denied operations and restarts.

Initial restrictions still cover every one of the 14 registered operations,
both aliases and both admitted tenant-qualified principals under remote and
trusted-local/shared profiles. Representative spoofed read/write/session requests
and malformed compile bodies retain the exact profile-denial code. Admission is
observed separately through the protected principal route. After restart each
profile uses representative spec read, inline compile and session cancellation
denials followed by the full storage comparison.

`after_restart_operations` now contains the three operation observation records,
instead of only their count. Initial and restart observations record the exact
denial code; malformed controls are explicit records. The local control also
records restored operation evidence and the four unchanged table names.
Source `CompilerService::compile_content` performs inline compilation without
persisting reports, so restored inline operations preserve the compared store.
No claim of owner-scoped compiler authoring or #387 closure is made.

### Recovery

Four IDs remain. `legacy_principal_namespace_cold_readback` is folded into
`attested_upload_restart_owner_isolation`: each backend observation now embeds
`legacy_cold_readback` with before/after inventories, document identity and exact
unchanged assertion. Both Surreal and PostgreSQL upload, worker interruption,
cold boots, owner retrieval, foreign owner/tenant/issuer denials, legacy
non-adoption and full final-state comparisons remain.

Joined reclaim and qualified supervisor implementation are unchanged. Their
exact proof/fence binding, physical exclusion, replay refusal and uncertain
effect/accounting preservation remain required. Missing qualification still
produces a blocked result under the retained supervisor ID.

## Current source, unresolved integration and verification limits

At entry, the authoritative waypoint reported revision 3084, Execute
IN_PROGRESS, active child `function-first-release-certification`, current task
`2.1`, implementation 137/152. This is coordination metadata, not a frozen byte
identity. Work is authored in the shared working tree; no fresh Git/source hash,
import or standalone source-identity command was run.

FC2 owns implementation of the consumed public helper and its receipt shape.
FC4 must reconcile the runner/schema/docs/scenario disposition to these 39
credential + 3 compiler + 4 recovery labels. The lead must collect both handoffs
before the new whole-phase inventory. No additional design decision is requested.

Commands executed were source inspection (`pwd`, `cat`, `sed -n`, `rg`,
`rg --files`) and assigned implementation/documentation writes (`apply_patch`).
Observed outputs were repository source/contracts and successful patch receipts.
No test, lint, format, typecheck, Cargo, compiler/import probe, build, acceptance,
external gate, `git diff --check`, or review-driven verification command ran.
Therefore syntax/import compatibility, actual denial/status/contact results,
database transactions, cold persistence, qualified recovery and release timing
remain unverified. No verification tier passed in this task.

No unrequested feature or production guard was added. Added fixture assertions
trace to explicit requirements: complete pagination, exact source-policy denial,
distinct provider paths, unchanged full compiler resources and restored local
operations. MCP metadata, annotations and kernel requests receive no new
authority; this task adds no NativeTool/McpRegistry execution route.

The uncomfortable limit is that receipt consolidation itself provides no runtime
evidence, and the published two-group program no longer directly re-observes
credential migration or compiler restrictions on each architecture. Only an
operator-authorized program after complete implementation can establish current
behavior; all earlier receipts remain historical.
