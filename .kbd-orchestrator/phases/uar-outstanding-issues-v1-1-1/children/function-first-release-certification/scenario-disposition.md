# Function-first scenario disposition

Date: 2026-10-17. Child Execute FC4. This is a source and contract inventory,
not an executed result. The operator verification gate is **CLOSED**.

The historical mandatory inventory has 80 IDs: runtime 23, credentials 48,
recovery 5 and compiler 4. The selected local program has 65 IDs: runtime 19,
credentials 39, recovery 4 and compiler 3. Exactly 16 historical IDs are
folded or replaced and one new user-facing ID is added. The four native source
supplements remain separate. Both published children select only `streams` and
`recovery`, in that order. An ID fold does not itself save an invocation; removed
partition loops, repeated restarts and published groups are the operational cut.

Every row below names a historical ID (or the new replacement), its final ID,
the production route and backend, a required control, and the evidence field or
observation. The runner requires exact set equality for each selected group.
No historical label may be silently skipped or used as credit for new bytes.

## Runtime: 23 historical → 19 final

Binding uses authenticated `POST /api/uar/runs`, public stream/status/history
readback and the real provider/store collaborators. Stream rows use public SSE,
AG-UI and provider wire peers. MCP and combined rows use admitted run/tool
outcome/receipt routes. The observed negative status or denied contact must be
present in each retained observation, alongside its positive control.

| Historical ID | Final ID | Consequential control and receipt |
| --- | --- | --- |
| `binding-qualified-durable-control` | `binding-16000-catalog-reservation-refusal` | Adequate envelope succeeds with provider contact, settled accounting and post-restart run; compare the same route's actual 16k catalog refusal. |
| `binding-unattested-refusal` | same | Unattested remote store refuses before provider contact; refusal history remains visible. |
| `binding-16000-catalog-reservation-refusal` | same | 16,000-token reservation is refused against the actual catalog while adequate control succeeds; record contact/accounting and history. |
| `binding-shared-alias-and-credential-free` | same | Shared alias/credential-free authority limits and allowed control; record effective source and contacts. |
| `binding-fallback-policy` | same | Allowed fallback reaches the selected provider; ungranted fallback has no unauthorized contact. |
| `binding-timeout-inheritance` | same | Effective timeout inheritance is read from the admitted run and terminal observation. |
| `binding-team-source-policy` | same | Team execution source policy admits its grant and refuses foreign source before effects. |
| `binding-summary-capture-policy` | same | Host summary remains captured through persisted run history. |
| `binding-invalid-budget-refusal` | same | Invalid budget is rejected before provider/store effect; refusal is retained. |
| `binding-v1-messages-policy` | `messages-facade-policy` | Distinct native Anthropic `POST /v1/messages` provider path, complete allowed stream, exact source denial 403 and zero denied contact; combined with OpenAI facade evidence in credentials. |
| `streams-native-partition-matrix` | NEW `streams-prompt-skill-output-timing` | One complete public `POST /api/uar/skills/` → attached `POST /api/uar/runs` journey; unique prompt and enabled overlay both determine exact SSE, AG-UI and persisted answer. Exhaustive split/BOM/newline permutations leave mandatory release execution. |
| `streams-sse-fields-fixed-partitions` | NEW `streams-prompt-skill-output-timing` | Exact assembled public answer and first parsed content timing replace repeated fixed-field partitions; missing/extra/reordered text fails. |
| `streams-eof-matrix` | same | Text-without-terminal and pending-tool EOF fail; pending tool has zero effect. |
| `streams-invalid-matrix` | same | Malformed data, overflow, invalid UTF-8, post-terminal data fail; valid unknown event is a positive forward-compatible control. |
| `streams-tool-terminal-control` | same | Real successful tool loop reaches terminal and retained result. |
| `streams-cancel-cleanup` | same | Cancellation closes the run/peer and a later run succeeds without leaked pending effect. |
| `streams-graph-inherited-control` | same | Approved inherited host graph delegation succeeds; unapproved source cannot expand its grant. |
| `mcp-application-error` | same | Application failure preserves typed error payload, cold immutable receipt and independent later success. |
| `mcp-explicit-success-and-omitted-status` | same | Explicit and omitted `isError` success remain distinct outcomes; repeated restart/later-success work is removed only for these successful controls. |
| `mcp-protocol-and-timeout` | same | Protocol error and timeout retain distinct typed outcomes and effect policy. |
| `mcp-uncertain-effect-preserved` | same | Uncertain effect stays unknown after restart; no replay can convert it to a safe success. |
| `combined-valid-control` | `combined-failed-tool-incomplete-stream` | Independent later valid run and second effect prove continuation; original failed receipt stays unchanged. |
| `combined-failed-tool-incomplete-stream` | same | Failed tool/incomplete stream remains failed with one original effect; after valid control exactly two effects exist. |
| NEW `streams-prompt-skill-output-timing` | same | Disabled eligible attached skill reports exact `disabled`, excludes unique overlay/dependencies while ordinary prompt may contact provider; second prompt changes the deterministic answer. Client admission/first parsed delta/terminal and peer receive/first/last write are recorded within separate clock domains. |

The new stream journey observes explicit attachment propagation, not autonomous
model skill selection or external-model latency. Timing is diagnostic and needs
an operator release disposition; no numeric SLO is invented.

## Credentials: 48 historical → 39 final

All nine common IDs remain direct release labels. Their installed-image routes
are authenticated `/api/uar/credentials`, `POST /v1/messages`, and admitted
`/api/uar/runs` as applicable. Controls include a distinct owner/source and
negative contact/readback; their observations remain in the credentials
adapter's direct receipt.

| Historical and final common ID | Required functional evidence |
| --- | --- |
| `identity-isolation` | Same-owner allowed credential use and cross-owner/tenant refusal before disclosure or provider contact. |
| `local-installation-stability` | Local install and restart preserve selected source and usable response. |
| `shared-source-grants` | Explicit shared source is admitted only for matching authority; ungranted source denied. |
| `absent-service-policy` | Missing credential service is reported without invented provider success. |
| `store-decrypt-failures` | Decrypt failure refuses use and does not leak secret material. |
| `single-owner-compatibility` | Single-owner mode continues to admit its owner and deny another. |
| `local-quarantine` | Quarantined local record remains unavailable without migration authorization. |
| `messages-facade-policy` | OpenAI facade uses `/v1/chat/completions`; native Anthropic uses `/v1/messages`; allowed streams complete, foreign source denies exact 403 `credential_source_denied`, with zero denied peer contact. |
| `response-correction-policy` | Response correction remains tied to its permitted source and observed output. |

The three backend journeys each use the actual local, SurrealDB and PostgreSQL
store/journal implementations through `GET /api/uar/credentials/migration-preview`
and conditional migration HTTP. Each backend still performs its own transaction,
cold boot, concurrency, lost-response and receipt decisions. Shared malformed
query and If-Match parser syntax is observed once on local; backend-specific
authority and state decisions are observed on all three.

| Local historical ID | Final ID and evidence |
| --- | --- |
| `local-metadata-upgrade` | same; metadata revision becomes ready, initializer retry leaves it stable. |
| `local-preview-authority-pagination` | `local-legacy-conditional-copy`; ordinary user 403, complete ordered unique keyset with inventory/page counts, stale cursor 409 and local malformed syntax 400. |
| `local-legacy-conditional-copy` | same; owner conditional copy, wrong owner denial, immutable journal receipt, retained source plus folded pagination/redaction. |
| `local-fixed-local-conditional-copy` | same; fixed principal source discriminator and cold replay equality. |
| `local-revision-drift` | same; stale source 412, no destination, preserved timestamps and `source_changed` status. |
| `local-mapping-drift` | same; mapping 412 plus journal conflicts 409, missing source 404 and unmapped quarantine statuses. |
| `local-destination-conflict` | same; occupied destination 409 and `destination_conflict` preview. |
| `local-concurrent-replay` | same; concurrent attempts return one immutable receipt. |
| `local-lost-response-retry` | same; discarded response retry returns same receipt and `already_applied` preview. |
| `local-immutable-rotation-deletion-restart` | same; rotation/deletion/cold replay leave original journal immutable; changed/missing destination statuses remain. |
| `local-historical-receipt` | same; unconditioned replay succeeds, conditioned replay 412, unknown historical destination revision is not invented. |
| `local-preview-status-inventory` | `local-mapping-drift`, `local-revision-drift`, `local-destination-conflict`, `local-lost-response-retry`, `local-immutable-rotation-deletion-restart`, `local-historical-receipt`; exact missing/quarantine/conflict/source-changed/already-applied HTTP and preview statuses. |
| `local-secret-redaction` | `local-legacy-conditional-copy`; preview, receipt and raw logs exclude key, ciphertext, hint and free-form reason. |

| Surreal historical ID | Final ID and evidence |
| --- | --- |
| `surreal-metadata-upgrade` | same; metadata revision ready and initializer retry stable in SurrealDB. |
| `surreal-preview-authority-pagination` | `surreal-legacy-conditional-copy`; ordinary user 403, complete ordered unique keyset/counts and stale cursor 409. |
| `surreal-legacy-conditional-copy` | same; conditional owner copy, wrong-owner denial, immutable Surreal journal/source plus folded pagination/redaction. |
| `surreal-fixed-local-conditional-copy` | same; fixed principal discriminator and Surreal cold replay equality. |
| `surreal-revision-drift` | same; stale source 412, no destination, timestamps and `source_changed`. |
| `surreal-mapping-drift` | same; mapping 412, journal conflicts 409, missing source 404, quarantine statuses. |
| `surreal-destination-conflict` | same; occupied destination 409 and preview conflict. |
| `surreal-concurrent-replay` | same; two transactions return one receipt. |
| `surreal-lost-response-retry` | same; discarded response retry returns same receipt and applied preview. |
| `surreal-immutable-rotation-deletion-restart` | same; changed/missing destination statuses, immutable receipt across cold restart. |
| `surreal-historical-receipt` | same; historical conditioned replay 412, no invented revision. |
| `surreal-preview-status-inventory` | `surreal-mapping-drift`, `surreal-revision-drift`, `surreal-destination-conflict`, `surreal-lost-response-retry`, `surreal-immutable-rotation-deletion-restart`, `surreal-historical-receipt`; exact per-store preview/HTTP statuses. |
| `surreal-secret-redaction` | `surreal-legacy-conditional-copy`; preview, receipt, logs exclude secret fields and free-form reason. |

| Postgres historical ID | Final ID and evidence |
| --- | --- |
| `postgres-metadata-upgrade` | same; metadata revision ready and initializer retry stable in PostgreSQL. |
| `postgres-preview-authority-pagination` | `postgres-legacy-conditional-copy`; ordinary user 403, complete ordered unique keyset/counts and stale cursor 409. |
| `postgres-legacy-conditional-copy` | same; conditional owner copy, wrong-owner denial, immutable PostgreSQL journal/source plus folded pagination/redaction. |
| `postgres-fixed-local-conditional-copy` | same; fixed principal discriminator and PostgreSQL cold replay equality. |
| `postgres-revision-drift` | same; stale source 412, no destination, timestamps and `source_changed`. |
| `postgres-mapping-drift` | same; mapping 412, journal conflicts 409, missing source 404, quarantine statuses. |
| `postgres-destination-conflict` | same; occupied destination 409 and preview conflict. |
| `postgres-concurrent-replay` | same; two transactions return one receipt. |
| `postgres-lost-response-retry` | same; discarded response retry returns same receipt and applied preview. |
| `postgres-immutable-rotation-deletion-restart` | same; changed/missing destination statuses, immutable receipt across cold restart. |
| `postgres-historical-receipt` | same; historical conditioned replay 412, no invented revision. |
| `postgres-preview-status-inventory` | `postgres-mapping-drift`, `postgres-revision-drift`, `postgres-destination-conflict`, `postgres-lost-response-retry`, `postgres-immutable-rotation-deletion-restart`, `postgres-historical-receipt`; exact per-store preview/HTTP statuses. |
| `postgres-secret-redaction` | `postgres-legacy-conditional-copy`; preview, receipt, logs exclude secret fields and free-form reason. |

## Recovery: 5 historical → 4 final

Recovery uses installed HTTP knowledge upload/index/retrieval and collaboration
owner/reclaim routes against both durable backend implementations. The qualified
supervisor case additionally reads the real independent controller journal and
physical exclusion receipt. Missing genuine qualification is blocked.

| Historical ID | Final ID | Consequential control and evidence |
| --- | --- | --- |
| `unattested_remote_upload_refusal` | same | Remote upload without host-attested durability refuses before committed work. |
| `attested_upload_restart_owner_isolation` | same | Attested upload, worker interruption, cold restart, owner retrieval and foreign owner/tenant/issuer denials; committed/cold inventory for Surreal and PostgreSQL. |
| `legacy_principal_namespace_cold_readback` | `attested_upload_restart_owner_isolation` | Both backend cold boots retain legacy document/inventory unchanged and reject cross-owner retrieve/retry/adopt. |
| `joined_quiesce_reclaim_controls` | same | Quiesce and joined proof transfer queued work only; invalid/replayed evidence leaves epoch/effects/accounting unchanged. |
| `qualified_supervisor_reclaim_controls` | same | Operator-qualified controller, actual journal registration, distinct fences, physical whole-boot exclusion, fresh proof/replay refusal and preserved uncertain effect/accounting. |

## Compiler: 4 historical → 3 final

The compiler adapter runs under credentials using both `/api/uar/compiler` and
`/api/compiler` aliases. Initial remote/shared denials cover every registered
operation, two admitted tenant principals and both aliases before effects;
representative spoofed headers and malformed bodies require the exact profile
denial code. Post-restart observations sample spec read, inline compile and
session cancel, then compare full stored specs/reports/sessions/registered agents.

| Historical ID | Final ID | Consequential control and evidence |
| --- | --- | --- |
| `compiler-local-single-owner-control` | same | Seed and read both aliases, compile/verify/register; after restricted profiles restore local read, write and session behavior with exact stored resource readback. |
| `compiler-remote-two-principal-denial` | same | Both admitted remote principals denied every operation/alias before effects; representative post-restart denial and full state equality. |
| `compiler-local-shared-denial` | same | Shared credential profile denies the same initial operation inventory and representative post-restart controls without changing state. |
| `compiler-restart-preserves-existing-resources` | `compiler-local-single-owner-control` | Full spec/report/session/registered-artifact storage and public readback unchanged across profile changes and restart; local operations restored. |

## Scope and risk

The published two-group program does not reobserve budget refusal, credential
migration, compiler profile restriction or MCP failure on each architecture.
They remain local release blockers. Source review cannot establish that the
folded assertions pass or that timing improved. The reduced program and every
published child still require actual operator-authorized functional receipts.
