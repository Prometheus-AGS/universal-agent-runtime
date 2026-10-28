# A2UI action continuation adversarial design review

2026-10-25. Execute; verification CLOSED. Reviewed only `design.md` in this directory and the root AGENTS verification lock. No producer conversation, other critic reports, implementation diff or runtime evidence was read. This is a native fresh-context GPT-family review, not a cross-model claim; exact runtime producer/judge model identifiers are not exposed. The artifact's final resolved contract governs earlier wording.

## WARNING — define restored pending receipts as uncertain when no local task owns them

The final contract persists a pending receipt before POST, retains it across remounts, and treats transport uncertainty as consumed. It does not define how a pending graph receipt restored after a document/process restart is classified when its ownership token and async task no longer exist. The disclosed crash window covers server acceptance before local receipt persistence, so this is an admitted failure scenario rather than speculative hardening.

Falsifying design scenario: persist pending; server accepts POST; client exits before persisting the 202 receipt; reopen the same source-run/surface. The graph says pending, but there is no live owned request or known continuation. Displaying “Sending action…”/busy implies current work the client cannot establish. Clearing pending and permitting retry risks another admitted continuation. Leaving a durable pending receipt without a recovery rule makes the proposed state machine incomplete.

Required correction: explicitly classify restored pending-without-current-owner as outcome unconfirmed and consumed, display the existing unconfirmed/no-resubmit text, and permit unrelated chat by retaining no orphaned local stream reservation. A component remount while its original document-owned async task remains active must continue to reflect that task instead. Keep the crash limitation: this classification cannot recover an unknown continuation or prove the server stopped. Add this transition to the authored, unrun fixture inventory. Do not add a new discovery endpoint or claim host idempotency.

## Other challenged decisions

No additional unresolved design decision was identified in the final contract for these boundaries:

- The operation key is explicitly source native run plus surface; same-ID content changes/remounts cannot reset consumption.
- Actual edited model capture occurs at click and is detached; callback guards supplement fieldset disabling for programmatic/custom callbacks.
- Normal chat and A2UI share synchronous ownership acquisition before I/O. Matching-token release and quiesced failure handling prevent one operation from releasing/canceling another.
- Only explicitly proven pre-admission refusals restore retry eligibility. Unknown transport outcomes, malformed success receipts and failed persistence do not trigger blind retries.
- A parsed admission receipt precedes continuation attachment; attachment reuses the existing stream reduction path and success requires a successful terminal, not HTTP 202, cancellation or EOF.
- The visible contract distinguishes unbound, pending, admitted, interrupted, refused and uncertain states without moving focus or exposing private HTTP content.

These are decisions stated in the artifact, not independently established implementation facts. In particular, upstream constructor/schema claims, mounted endpoint refusal statuses, graph bootstrap/persistence behavior and the completeness of shared stream reuse have not been source-verified in this artifact-only assignment. Implementation must trace them to actual source before relying on them.

## Disposition and limitations

Direction acceptance is withheld for the restored-pending transition above. The correction is bounded to the existing receipt state machine and uncertainty language. No production changes were made; this report is the only authored file. No tests, builds, imports, checks, browser, audit CLI or detector ran. Machine sycophancy screening is deferred under the CLOSED lock; manual screening separates design decisions from proof and imposes no finding/praise quota. There is no runtime PASS or acceptance credit. Actual edited-value submission, endpoint admission, duplicate prevention, stream ownership under concurrency, persistence/restart recovery and accessibility remain unverified.

## Bounded corrective rereview

Reviewed only the final appended paragraph of `design.md` in this artifact-only rereview. The original report is preserved.

The orphaned-pending WARNING is resolved at design level. The correction explicitly classifies restored pending without a current document-owned async task as unconfirmed and consumed, displays the no-resubmit message, and retains no orphaned local stream reservation. It distinguishes a component remount with a still-active task using actual document-local ownership rather than elapsed time. It names the acknowledged crash window and adds authored, unrun transition cases without inventing server recovery or idempotency.

No unresolved design finding remains from this bounded review. The direction may proceed to implementation. This is not implementation certification, a runtime PASS or permission to run verification; gate remains CLOSED. No implementation, source/API behavior, runtime or tooling check was inspected or executed during this rereview. Same-family native isolation and deferred machine screening limitations remain as recorded above. Only this disposition was appended.
