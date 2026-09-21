# Analysis: runtime-harness-gap-closure

Date: 2026-09-16. Stage: Analyze. Mode: stack specified (existing Rust host, Liter 1.18.2, typed prompt/turn assembly). Assessment baseline: commit 226d4a0af89811975662cf3f203c699f70e8cebc. No product code or dependency changes are authorized by this artifact alone; Spec and Plan precede implementation.

## Decision

Extend the existing host with one destination-specific request-preparation path and a protected-context budget function. Keep the existing router, typed fragments, governed execution, persistence and evaluation runner. Do not add a replacement agent framework.

The operator's latest requirement strengthens F2/F3: per-model templates must be explicit, and context compression must not compress tool calls or other data. Interpret “other data” conservatively: tool arguments/results, structured payloads, code/log/file blocks, multimodal references, evidence/provenance, required instructions, current input, durable decisions and pending work are protected. Unknown content is protected until host-owned metadata explicitly makes a prose segment eligible. Role alone or a JSON-looking-text heuristic is insufficient.

The uncomfortable case is a valid tool result larger than a destination model's entire input allowance. Lossless preservation and successful dispatch cannot both be guaranteed in that case. Preserve the data and return an explicit budget/compatibility outcome, or select an already-authorized capable destination. Never make the budget appear to fit by deleting, truncating, summarizing, or silently replacing protected data with a pointer.

This is a design decision for Spec, not a claim of an implemented or proven function.

## Source findings added to the assessment

| Seam inspected | Observation | Consequence |
|---|---|---|
| `src/llm/prompt_dialect.rs` | Family detection uses substrings; request parameters and rendering preferences are family-wide. | Exact endpoint/model profiles must replace unqualified family assumptions for supported settings. |
| `src/uar/runtime/prompt/assemble.rs:39` | RenderOptions has two booleans; the renderer chooses structured envelopes or plain concatenation. | Existing fragments are a good input, but there is no inspected versioned per-model template contract. |
| `src/uar/compiler/ir.rs:771`, `src/uar/runtime/manager.rs:3427` | Descriptor declares a dialect override; inspected run rendering detects from the model string. | Add an end-to-end override propagation fixture; compiler conformance alone is not runtime propagation proof. |
| `src/uar/runtime/context/summarizer.rs:17` | The summarizer flattens messages to role/text; its instruction explicitly includes important tool results/data. | Protected messages must never enter summarizer inputs; asking an LLM to preserve them is not lossless preservation. |
| `src/uar/context/strategy.rs:409`, `src/uar/runtime/context/manager.rs:275` | Older message ranges can enter summarization. | Apply protection before every structural and token-budget strategy, not only the final normalization pass. |
| `src/uar/runtime/context/reduce.rs:40` | Severed groups are dropped to restore valid pairing. | Pair validity is weaker than the new preservation requirement. Protected groups must survive intact or cause an explicit failure. |
| `src/llm/orchestrator.rs:276,495,1731`, `src/uar/runtime/context/truncate.rs:27` | Native/MCP/graph paths can truncate tool results before history; default output limit is 32,000 bytes. | Preserve canonical raw results before model/display formatting; a later budget function cannot recover deleted bytes. |
| `src/llm/liter_driver.rs:94` | The inspected builder assigns model/messages/tools and extra body, but no explicit output ceiling there. | A calculated output reservation must match the actual destination request field and endpoint semantics. |
| `src/uar/runtime/prompt/manifest.rs:77` | PromptBudgets exposes context/output ceilings and rendered byte/character counts. | Extend existing redacted manifests with budget decisions and identities, not a parallel telemetry system. |

The new source observations are not runtime reproductions. Existing source and historical test findings remain in assessment.md.

## D1 — Per-model templates and destination preparation (F1/F2/F3)

Keep three responsibilities distinct:

1. **Prompt template:** arrangement of typed instruction/prose fragments, required sections, supported roles, escaped data envelopes and explicit placeholders.
2. **Chat serialization:** the endpoint's message/tool/multimodal representation and, only where the host actually owns raw inference formatting, the model's chat control tokens.
3. **Provider settings:** supported output/reasoning/structured-output/cache parameters and endpoint-specific wrappers.

A hosted chat API must not receive raw model control tokens merely because a local-inference model uses them. Hugging Face documents model-specific chat formats and warns against duplicating special tokens; use that as a reference pattern, not a reason to introduce Transformers into the Rust host. [Chat templates](https://huggingface.co/docs/transformers/main/en/chat_templating), [tool representation](https://huggingface.co/docs/transformers/main/en/chat_extras).

Proposed profile identity comprises provider, endpoint kind, exact model/revision (or explicitly reviewed alias), profile revision, template hash and tokenizer/counting revision. This is a proposed data contract, not an existing Rust type or configuration key. Existing policy remains authoritative over allowed models and settings.

Resolution order: validated descriptor/operator selection within host policy; exact endpoint/model profile; an explicitly declared and verified compatible family profile; conservative generic rendering only when the endpoint's required capabilities and limits are known. Unknown compatibility or count bounds returns a typed unsupported outcome. An override cannot authorize forbidden parameters or force an incompatible template. Store provenance for which override won.

Each template declares required slots and allowed roles. Missing required slots, attempted loss of host/policy/tool/evidence sections, or an incompatible descriptor override must fail preparation. Template rendering is deterministic and data-only; do not execute model-supplied templates, filesystem includes, or arbitrary filters. Reuse current typed rendering first. MiniJinja is a deferred reference, not a new dependency: current requirements do not establish a need for arbitrary Jinja programs.

For every initial call, loop iteration, retry, failover, graph step and resumed run:
- Start from the immutable canonical turn and raw protected records, not the previous model's rendered/reduced request.
- Resolve the actual destination; narrow policy; resolve its template/settings/counting profile.
- Assemble all required instructions, tool schemas, active skills, world state, memory/evidence, current input and retained history before final budgeting.
- Plan and perform eligible-prose compression under D2, then serialize and validate the complete destination request.
- Record destination/profile/template identities and budget disposition in the existing manifest. Dispatch only the validated request.
- On a destination change, redo preparation without re-executing already-completed tools. Retry must retain existing idempotency and side-effect constraints.

Opaque or signed provider continuity blocks are protected. Preserve them only through a compatible protocol. A different provider that cannot represent them is ineligible for transparent failover; do not rewrite them or invent equivalent reasoning. Cache identity includes the destination profile/template and the rendered request identity so re-rendering cannot reuse an incompatible prefix.

### Provider evidence and limits

| Family/endpoint | Current evidence | Analysis decision |
|---|---|---|
| Anthropic | Official docs distinguish manual extended thinking from adaptive thinking by model generation. Adaptive output ceilings include thinking. | Existing universal enabled/budget_tokens emission is not sufficient. Pin a supported setting profile per model; do not subtract reasoning twice. [Manual/migration](https://platform.claude.com/docs/en/build-with-claude/extended-thinking), [adaptive cost controls](https://platform.claude.com/docs/en/build-with-claude/adaptive-thinking). |
| Moonshot/Kimi | Official CLI documentation scopes thinking.keep to supported models while thinking is enabled. | Do not enable preserved thinking solely because a string contains kimi or a conversation is multi-turn. Count preserved blocks in input. [Official CLI](https://github.com/MoonshotAI/kimi-cli/blob/main/docs/en/configuration/env-vars.md). |
| GLM/Z.ai | Official overview documents thinking and model input/output limits; retrieved evidence did not establish all current high/max effort combinations. | Keep exact effort values unverified until the selected deployment contract is checked. [Parameters](https://docs.z.ai/guides/overview/concept-param). |
| Qwen/Model Studio | Official guide documents enable_thinking. This pass did not verify preserve_thinking or all deployment wrappers. | Verify exact endpoint/model fields before a supported profile can emit them. [Guide](https://docs.modelstudio.console.alibabacloud.com/en/model-studio/deep-thinking). |
| OpenAI-compatible / Responses | Local Liter adapter uses ChatCompletionRequest; the dialect comment also discusses Responses. No endpoint-specific OpenAI documentation was loaded in this bounded pass. | Do not transplant Responses fields into Chat Completions based on family name. Validate the actual endpoint and structured-output schema in Spec. |
| MiniMax / unknown | Inspected code asserts Markdown aversion; this pass did not establish that assertion for every current model. | Treat layout preference as an evaluated profile choice, not a universal fact. Unknown profiles cannot silently claim supported reasoning or precise token counts. |

These sources establish design constraints, not a complete certified provider matrix. Exact configured-model fixtures and live acceptance remain exit requirements. No provider parameter or dependency pin is changed here.

## D2 — Protected-context budget function (F3, explicit operator requirement)

Proposed pure planning interface, not existing API:
`plan_context_budget(destination_profile, canonical_turn, protection_policy, output_reservation, request_limit) -> fit | compress_prose(plan) | protected_overflow | unsupported_counting`.

The function performs no I/O or mutation. Trusted host code resolves identities, executes any approved summarization through governed model bindings, charges cost/token budgets and handles cancellation. Agent kernels gain no write capability.

### Protected representation

Protection is independent of current Retention (Session/Turn/Ephemeral/Reclaimable). A retained item can still be non-compressible. Freeze a manifest of protected IDs, original order, roles, payload bytes/hashes, call/result links and provenance before any compaction. Protect the entire assistant message when it contains tool calls, together with all corresponding results, including parallel groups and pending-call records. Preserve repeated messages by identity, not content deduplication.

Raw argument strings and textual payloads remain byte-identical in canonical storage and compaction output. Typed structured values retain semantic identity and the original source bytes where available. Provider-required serialization/escaping may change wire bytes; decoding must recover the same payload and links. This is not permission to normalize numbers, rewrite code, summarize JSON, or flatten multimodal blocks. Existing fragment hashes normalize line endings, so they are not sufficient as the byte-preservation oracle; retain a separate raw-payload digest for this invariant.

Eligible prose is explicitly marked by the host, and excludes user constraints, durable decisions, pending actions and evidence. Summarize only contiguous eligible spans, retaining their chronology around protected blocks. Summaries inherit source authority and provenance; they must never become new system authority merely because current code stores summaries with a System role. The summarizer receives eligible prose only, not protected bodies. Failed, cancelled, empty, over-target or invalid summaries do not trigger truncation fallback; retain originals and either fit or report a bounded failure.

Protection never grants access. Revalidate tenant ownership, source authorization and revocation before dispatch, including summarization, retries and checkpoint resume. If required protected evidence is no longer authorized, invalidate the prepared request and block dispatch with a redacted authorization outcome; do not leak it into context/audit or silently replace it with a summary. Retention/deletion follows the governing data policy, not the budget planner. Preservation applies only within continuing authorization; no byte-retention promise overrides a required deletion. Test revocation after retrieval, after checkpoint, and between preparation and dispatch using the host's authoritative permission boundary.

Normalization must be reconciled too: `src/uar/runtime/context/normalize.rs:151` deletes orphaned, misplaced and duplicate results, and the canonical spec mandates orphan removal and synthetic missing results. Preserve canonical records; do not normalize protected bytes away as an implicit repair. Invalid or ambiguous histories that cannot be represented without discarding protected records return an explicit invalid-history outcome. An absent result may become a typed cancellation/error record only from a verified terminal host outcome with provenance; approval/execution still pending must remain pending and await/recover or block dispatch. Never manufacture cancellation from absence alone. Provider views and any permitted repair must be explicit and audited; Spec must replace the conflicting normalization scenarios and test duplicate/orphan/misplaced results plus pending approval/execution across resume.

### Budget calculation

All quantities refer to the resolved destination and one actual request:

- C: total context limit; L: independent provider input limit if present.
- O: configured output reserve, validated against the destination's output ceiling and sent on the actual request.
- R: additional context-consuming reasoning reserve only if the endpoint's accounting excludes it from O; otherwise zero.
- M: explicit model/counting-profile uncertainty allowance, never an unexplained fixed 1,000.
- I = min(L when present, host input ceiling when present, C - O - R - M).
- F: upper-bound token cost of the fully rendered protected-only request, including required instructions, schemas, selected skills/world/evidence, current input, framing and multimodal/continuity costs.
- If O + R + M exceeds C, or F exceeds I, return protected_overflow with the limiting components. Do not hide a negative allowance with saturating arithmetic.
- Otherwise B = I - F is the maximum available allowance for eligible prose and summary framing. Return fit when the complete request fits; otherwise allocate a summary/prose plan within B.

F and B are planning bounds. Tokenization is not universally additive across concatenated fragments. Recount the complete, final endpoint-shaped request after rendering, summaries, skill reattachment, normalization and all driver-added framing. An estimator reports its method/revision and whether it is exact, bounded or approximate. Approximate fallback counts alone cannot prove a hard context bound. For unknown or unsupported multimodal accounting, require a validated conservative bound or provider preflight support; otherwise return unsupported_counting. Cache discounts affect cost, not permission to ignore context occupancy.

The count must cover tool schemas as well as tool call IDs/names/arguments/results, role/name framing, output-format schemas, retrieval citations, image/audio/video/document costs, provider continuity and generation markers. JSON byte length is not a universal model-token count.

Summarization has its own independently budgeted request and output cap through the trusted host. It cannot receive an over-budget concatenation or recursively summarize without a bound. Chunk only eligible prose spans; charge all summarization work to the same governing spend/latency/cancellation ceilings. Keep context capacity separate from money/iteration limits while reporting both.

Illustrative arithmetic (synthetic tokens, not any real model's limits): C=32,000; O=4,000; R=0; M=1,000 gives I=27,000. F=22,000 leaves B=5,000. Reducing 9,000 eligible prose tokens to at most 5,000 can permit dispatch only after final recount. F=28,000 must return overflow even if every eligible prose token is removed. A failover with C=16,000 has I=11,000 and is ineligible for this same protected set.

### Tool result acquisition and existing contract conflict

The current canonical spec explicitly permits dropping a complete call/result pair and requires bounded middle-out tool output at ingest (`openspec/specs/conversation-history-integrity/spec.md:20,45`). The user's new requirement takes precedence. Spec must modify those requirements explicitly; do not claim unchanged behavior satisfies lossless protection.

Preserve raw accepted tool results in tenant-scoped host-owned durable records before display/model formatting, for native, MCP, graph and terminal paths. Existing truncation may remain a clearly labelled display projection; it cannot silently replace the canonical protected result used by the budget function. Retrieval-by-pointer or an out-of-band artifact is not equivalent to sending required protected data to the model and is not an automatic overflow escape.

Resource-limited tool acquisition still needs explicit limits. If the tool contract cannot produce/persist the full result, report an explicit incomplete/tool-result-limit outcome and retain the available receipt; do not label truncated bytes as a complete successful protected payload. No unbounded storage promise is made. Define retention and tenant cleanup in Spec. Previously truncated historical data cannot be reconstructed; mark it incomplete on reload and never fabricate the missing middle.

## D3 — Remaining feedback: reuse versus build

| Feedback | Decision and existing seam | Required evidence |
|---|---|---|
| F1 routing | Extend src/llm/router.rs constraints with task/capability/whole-request feasibility and governed cost/latency. Keep explicit model preference and policy precedence; no second Router. Learned ranking remains experimental. | Deterministic selection/failover fixtures and task-quality/cost/latency baseline; no duplicate tool effects. |
| F4 A2A | Persist owner-scoped task/context-to-thread/run correlation in existing host persistence; adapters project shared lifecycle. Publish acceptance with recoverable/idempotent binding semantics. | Restart at creation/enqueue/publication boundaries; same ID, auth, artifacts, cancel and replay without duplicate enqueue. Use the actual persisted host, not a new in-memory adapter. |
| F5 skills | Reuse current activation machinery; freeze labeled dataset and thresholds in Plan. Preserve explicit invocation and scope precedence. | Positive/negative/ambiguous/long-context tasks; precision/recall, task success and cost. Preserve the existing 99% Recall@10 candidate-omission gate. |
| F6 graph/children | Extend existing lifecycle/step correlation only where trace joins cannot identify node, parent, iteration and tool. Preserve shared budgets and host-only writes. | Success/deny/approval/resume/cancel/exhaustion matrix, one terminal outcome and drained children across graph and linear paths. |
| F6/F8 agentic eval | Adapt the existing runner and available fixtures to drive real governed host execution with deterministic providers/tools. No additional agent framework is selected. | Seed faults in preservation, failover, approval, tenant scope and cancellation; local baseline gate must fail on each fault, with reproducible traces. |
| F7 knowledge | Extend existing retrieval/provenance verdict flow to answer support, contradiction/staleness and access-at-use decisions. Build deterministic negative fixtures before considering model-assisted verification. | Authorized evidence-to-answer support; revoked/cross-tenant content absent from both context and audit; explicit insufficient/conflicting evidence. Lexical overlap is not a factuality score. |
| F8 debt | Identify exact historical routing failures, fresh coverage denominator and profile applicability. Do not treat old pass counts as current results. | Exact test identities, coverage numerator/denominator/exclusions, retained commands and per-profile capability results. |

## Dependency and research decisions

Six candidates are machine-recorded in library-candidates.json. Adapt existing typed UAR/Liter seams and existing tiktoken-rs 0.12.0; reference Hugging Face template conventions and MiniJinja; reject LLMLingua as an as-is protected-data compressor and Rig as a replacement runtime for this phase. These are scope/fit decisions, not negative quality judgments about those projects. The custom work is a host-specific protection/accounting contract, not another tokenizer or template language.

External alternatives for F4-F8 lifecycle, observability, knowledge and evaluation were not independently surveyed in this pass. The table identifies source-grounded host integration needs, not proof that no suitable external component exists. Existing seams are the provisional integration baseline; external build-versus-adopt choices for those categories remain open for targeted Spec/Plan research if a concrete missing component is identified. The corresponding build_required entries carry this limitation rather than claiming a completed market comparison.

Planning prerequisite from final review: F4-F8 build-versus-adopt commitments are **blocked pending targeted candidate evaluation** for A2A/lifecycle persistence, trace correlation, skill-evaluation datasets/harnesses, knowledge verification/RAG evaluation and agentic-eval harnesses. Spec may define their required behavior, but Plan must not turn those provisional build_required entries into committed implementation choices until a scoped follow-up analysis records alternatives and verdicts. This is a research limitation, not a runtime defect or permission request. F2/F3's researched template/protection decisions are not blocked by this warning.

Research followed repository search, Context7, registries, then primary web fallback. Four repository searches; eight documentation requests (including three Context7 resolves, three queries and two direct official-doc opens); four registry attempts; seven fallback requests. Tier 2 reached its cap; the time cap was not exhausted. Registry TLS/403 failures left current releases/download metrics unknown; Firecrawl's installed CLI rejected developer. No certificate checks were disabled. Context7's Hugging Face query returned no match, so official docs were opened. Main-branch/generated docs are reference evidence, not proof of compatibility with the pinned Rust version. No speculative percentages or unverified version upgrades are recommended. See evidence/analysis-research.json.

## Specification boundaries and acceptance scenarios

Recommended dependency order: deterministic regression fixtures → protected representation/acquisition and budget contract → exact-model templates and per-attempt preparation → routing integration → lifecycle/skill/knowledge closure → agentic baseline and profile evidence. This refines the phase's ordering without implementing it.

Required spec deltas include conversation-history-integrity (protected groups, raw results, normalization, estimates and explicit overflow), prompt-assembly (templates/protection/manifest), model-path-resiliency (per-destination preparation), shared thread/A2A durability, graph correlation, activation evaluation, knowledge support and eval-harness local execution. Every eventual change needs at least one delta.

Minimum P0 scenarios: each context strategy preserves completed/parallel/pending groups and exact payloads; huge JSON, code, binary references, Unicode and repeated turns; summarize-only-prose capture; summarizer failure/cancel/oversize; protected-only overflow; missing token bound; actual output cap versus reserved output; under/at/over budget; reload of intact and historically incomplete data; smaller-window failover; incompatible opaque continuity; protected sequence/hash equality after each pass.

Template scenarios: same canonical turn across distinct model profiles; exact override wins only when compatible; required slots retained; escaping/round-trip data fidelity; hosted versus raw-inference formatting; no duplicate control tokens; endpoint-specific structured output/reasoning; unknown model; template change invalidates derived cache; retry/failover records the actual template and supported parameters. Golden rendering alone is insufficient: capture the final driver request and exercise it through production host paths.

## Verification tier and policy reconciliation

Replace the eval spec's general CI/nightly test requirements with local deterministic and separately labelled live-provider gates. GitHub Actions stay deployment-only.

The goals require affected-profile evidence but reserve Tier 3 for a separately scoped milestone. Do not waive either. Plan must scope a distinct **local supported-profile certification milestone** before this phase can be declared complete, with transport applicability and host-provider fixtures for embedded-mobile. No release/deployment is implied; server-full remains the implementation-session profile. The milestone is a prerequisite proposal, not an already-created or completed milestone. If it is not scheduled or cannot run, affected-profile evidence stays an explicit completion blocker. Linux/macOS Stable and Windows Experimental remain as recorded in the Rust rules.

Analysis verification is document/schema/arithmetic/source checking only. No new Rust build or runtime suite is warranted by these document-only edits. The prior compile was interrupted at 34:41 (exit143), so current compile health is still UNKNOWN. Runtime behavior, coverage and live-provider acceptance remain unverified. The prior zero-length-Ping readiness reflection remains pending; this stage does not complete it.

## Open questions carried to Spec/Plan

- Enumerate actual configured provider/endpoint/model versions and validate every supported profile's settings, tokenizer, multimodal estimator and continuity contract. Unverified profiles are not silently certified.
- Choose explicit prose-eligibility metadata and durable raw-result retention limits without weakening protected defaults. Decide migration behavior for previously truncated records.
- Freeze evaluation thresholds and datasets before implementation, and register the separate local profile-certification milestone.
- No unresolved stack contest exists: the existing Rust/Liter stack and dependency pins are retained.

## Review and verification receipt

The isolated source critic accepted D2 after authorization-precedence and normalization-preservation corrections. Cross-model round1 passed with two warnings; named unassessed test-library advice was removed and F4-F8 research claims narrowed. Round2 passed with zero critical findings and one warning, retained above as an explicit planning prerequisite. Reports and both packets are in review/analyze. Producer identity is known only at GPT-6 family level; the GPT-5.5 judge is distinct at that level, not independently verified against an exact producer deployment ID.

Tier0 checks passed for library-candidates.json against the skill's Draft2020-12 schema, six unique candidate IDs, all eight feedback IDs, numeric budget examples, evidence JSON and whitespace. Twenty analysis-source hashes were recorded and all28 assessment-source hashes remained unchanged. No Rust source changed; no new compile, runtime tests, coverage or live-provider run was performed. Current build health remains UNKNOWN from the assessment's interrupted check. No unrequested product changes, new dependencies or runtime guards were added.

ANALYSIS COMPLETE — bounded candidate research with the explicit F4-F8 planning prerequisite above. Next stage: /kbd-spec runtime-harness-gap-closure. No runtime fix is claimed.
