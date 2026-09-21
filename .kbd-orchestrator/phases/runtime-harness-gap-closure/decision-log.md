# Decision log — runtime-harness-gap-closure

Append-only analysis decisions. Date: 2026-09-16. Provenance: operator request, inspected source and bounded research. These guide Spec/Plan; none records delivered implementation.

## D-AN-01 — Preserve tools and other data

Decision: classify tool calls, arguments/results and other structured or mandatory data as protected; compress only host-marked eligible prose. Unknown content is protected. Protected overflow is an explicit failure/authorized routing decision.
Rationale: the operator explicitly requested a budget function that does not compress tool calls or other data. Existing normalization only preserves pairing validity, and existing ingest truncation cannot support lossless canonical data.
Tradeoff: a large protected set may make some destinations unusable. A stored pointer is not an equivalent automatic replacement.
Required follow-through: modify the current whole-group-dropping and ingest-truncation spec requirements; preserve raw accepted tool output before formatting, mark legacy incomplete data, and test final request and checkpoint invariants.

## D-AN-02 — Resolve per-model templates for every destination attempt

Decision: separate prompt templates, endpoint chat serialization and provider settings. Resolve exact provider/endpoint/model profiles with version/hash evidence; validate descriptor overrides and reprepare retries/failovers from canonical data.
Rationale: current family booleans and primary-request cloning do not establish destination correctness; provider documentation demonstrates generation-specific settings.
Tradeoff: unsupported profiles must remain explicit rather than inherit speculative settings. No universal Markdown/XML preference is certified.
Required follow-through: final driver-request fixtures, exact deployment documentation, output-limit propagation and compatibility checks.

## D-AN-03 — Reuse current host and dependencies

Decision: adapt current typed UAR/Liter seams and tiktoken-rs0.12.0. Reference model chat-template conventions and MiniJinja; reject whole-prompt LLMLingua compression and a Rig framework migration for this phase.
Rationale: missing behavior is host-specific preservation, accounting and integration; public engines do not establish those contracts.
Tradeoff: UAR owns the budget/profile contract and its regression corpus. Registry release/version verification failed; no new dependency is selected.
Candidate references: cand-001 through cand-006 in library-candidates.json.

## D-AN-04 — Summarization is governed work

Decision: pure budget planning performs no I/O; trusted host executes bounded prose-only summarization under existing cost, tenant, cancellation and inference authority.
Rationale: compression is itself a model request and cannot bypass the permissions/resource model or promote low-authority text into system instructions.
Required follow-through: source-span/provenance preservation, summarizer input capture, failure/cancellation/oversize scenarios, final recount and no lossy fallback.

## D-AN-05 — Keep verification local and preserve profile exit criteria

Decision: replace legacy eval-CI requirements with local gates. Plan a separately scoped local supported-profile certification milestone as a prerequisite to this phase's completion.
Rationale: goals demand affected-profile evidence while standing Rust rules and the phase goals reserve Tier3 for a distinct milestone. Neither may be silently waived.
Tradeoff: the phase remains incomplete if required evidence or that milestone is unavailable. No release or deployment is implied.
Status: milestone proposed for registration during planning; not created or executed here.

## D-AN-06 — Evidence limits

Decision: stop tier2 research at its eight-request cap and report partial provider/version evidence; do not disable TLS verification or introduce unverified dependencies.
Observed: registry Python TLS failures, curl HTTP403, unavailable firecrawl developer command, and a Context7 no-match handled by primary-source docs. Research receipt retains counts and successful evidence.
Carry-forward: prior compile health UNKNOWN (interrupted exit143); no current runtime/coverage/profile pass is claimed. Prior zero-length-Ping readiness reflection remains pending.

The uncomfortable thing: protecting every required tool result can prevent a run from continuing on its configured model. Hiding that by altering the data would satisfy a superficial token check while violating the operator's requirement.

## D-AN-07 — Review reconciliation (appended)

Decision: authorization takes precedence over continued inclusion of protected evidence. Revalidate before summarization/dispatch/retry/resume; block with a redacted outcome if required content is revoked, and obey retention/deletion policy. Preservation is not an access grant.
Decision: replace implicit normalization deletion/synthesis with explicit protected-history validation. Preserve canonical data, block invalid histories where repair would discard it, and synthesize a terminal tool outcome only from verified host evidence; pending approval/execution is not cancellation.
Rationale: the isolated source critic identified both conflicts between otherwise valid requirements. They require explicit spec deltas and revocation/malformed-history/pending-resume fixtures.
Cross-model round1: PASS, zero critical and two warnings. Removed named test-library adoption advice that lacked candidate entries. Clarified F4-F8 external options were not surveyed; local integration needs do not establish that no adoptable external component exists. Targeted research remains required before choosing a new external mechanism.

## D-AN-08 — Final review prerequisite (appended)

Cross-model round2: PASS, zero critical and one warning. Accept the analysis with a binding prerequisite: F4-F8 build-versus-adopt commitments remain blocked pending targeted external-candidate evaluation. The five affected machine-contract needs explicitly say PLANNING BLOCKED. Specification may define behavior; planning may not assume the provisional integration choice completes the missing research. F2/F3 template/protected-budget decisions remain ready for Spec.
Verification: round1 strict anti-theater score0.0; round2 strict score0.0803571417927742, both PASS. Isolated source critic rereview PASS after both source findings were resolved. Candidate schema, artifact whitespace and budget arithmetic passed; no runtime check was run.

## D-SP-01 — Complete sibling specification set

Decision: define six dependency-ordered OpenSpec changes with 11 capability deltas, 85 scenarios and 44 unchecked tasks. Keep early deterministic fixtures in their owning changes and reserve final aggregation for governed evaluation. Shared manager/orchestrator/prompt edits use a total order until Plan proves safe independence.
Rationale: the F1–F8 gaps intersect in final destination preparation, protected receipts and host lifecycle; isolated proposals cannot establish their joint acceptance boundary. Exact templates and the pure protected-data budget are explicit behavioral requirements.
Constraints: five external-candidate surveys, exact destination evidence, frozen metrics and a separate local Tier3 milestone remain mandatory Plan inputs. Spec validation does not authorize implementation or certify runtime behavior.
Review corrections: clarify zero-admissible-destination failure and add an exhaustion scenario; add explicit long-conversation retention/placement evaluation. Include the preserved source and validation receipt in the review inventory. The source attachment matches byte-for-byte, SHA256 5a478d7a091cd0e337ab64b15cb41b17cf869adf09f51ea07f07b75ef65f4efe. Final re-review is pending at this entry.

## D-SP-02 — Contract migration and review collection

Observed tooling constraint: installed OpenSpec1.10.0 strict validation rejects a MODIFIED requirement when an existing named scenario is omitted. Replace the intentionally retired pair-dropping/orphan-deletion requirement with an explicit REMOVED reason/migration and a new lossless requirement rather than silently deleting those scenarios. No canonical spec is edited in Spec.
Observed collector defect: the installed adversarial spec packet builder only collects native-KBD spec.md/tasks.json/verification.md. This repository uses OpenSpec. Build an explicitly inventoried packet containing all sibling proposals/designs/tasks/deltas, index/manifest, source evidence and canonical references; retain full content and hashes without modifying the external skill.
The uncomfortable thing: a schema-valid spec can still encode the wrong runtime contract or omit evidence from review. Use isolated whole-set review and preserve unsupported runtime claims as unknown.

## D-SP-03 — Spec review accepted

Final fresh-context artifact critic PASS with zero findings. Cross-model GPT-5.5 judge PASS with zero findings; producer is disclosed only as the known GPT-6 family. Both strict anti-theater rounds PASS at score0.0. Final packet includes all37 spec artifacts, nine canonical references and two source/validation inputs without truncation; current hashes match. Six strict validations PASS, 85 scenarios and44 unchecked tasks. The five research prerequisites remain binding for Plan; zero spec findings does not resolve that research or runtime evidence. Primary memory write succeeded as memory:e7maibil4725ngwvcs7d before final review; the final stage boundary is recorded separately.

## 2026-09-16 — Runtime harness gap closure Plan decisions

Keep the six Spec changes and 44 task units in a conservative total order. Change 1 now owns the minimum production preparation/output-envelope boundary, so its protected-context acceptance does not depend on change 2. Change 2 adds exact destination templates/settings and production task classification. The pure planner protects tool calls/results and all host-protected data; only eligible prose may be summarized. Unknown count bounds and protected overflow remain explicit outcomes. Acquisition policy separates 32,000-byte presentation from new 16 MiB receipt / 64 MiB run storage ceilings.

Five external candidate surveys are complete: reference A2A Python SDK, OTel GenAI conventions, Anthropic skill evaluation, Ragas claim-level faithfulness and Inspect AI methods; adapt the existing UAR host/Liter boundaries instead of installing another runtime. Exact model deployment evidence remains unverified: six configured defaults are an initial inventory, not six certified destinations. Pinned Liter's Anthropic transform can raise max_tokens to thinking budget + 1, so final request captures must follow provider transformation. No dependency pin changes.

Freeze 20 skill, 12 admission, 8 task-routing and 12 evidence cases plus context/crash matrices; actual baselines remain NOT_RUN. Preserve 99% Recall@10 and all four 60% frontend coverage gates. The two historical failure identities remain unresolved and require identification or evidence-backed supersession before the debt task passes. A separately registered local runtime-harness-profile-certification milestone must pass before final parent phase completion; missing stable-platform/live evidence blocks completion. Product checks remain local.

Review used two fresh-context native critics and two distinct-model GPT-5.5 rounds. The final cross-model BLOCK and native warnings are retained without relabeling. Requested final corrections (Surreal crash ownership, complete Tier2/milestone binding, true CRLF, semantic coding oracles) have Tier0 checks; no third review was run under the skill's two-round cap. This limitation is carried into execution. No product code or runtime tests were changed/run; interrupted prior build health remains UNKNOWN. The uncomfortable case is a correct refusal when protected content or a verified counting contract cannot fit an authorized destination.
