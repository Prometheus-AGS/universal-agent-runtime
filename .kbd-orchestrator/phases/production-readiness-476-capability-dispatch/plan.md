# Current bounded phase plan: typed images and constrained dispatch

## Position, dependencies and useful outcome

Phase: production-readiness-476-capability-dispatch. Source base629f5702; prior scoped #462 acceptance5aa36a40. Implement the common public-path dependency for #463/#464, preserve #388 final-destination budgets, and continuously align #466 claims. The first production contract is **Linux amd64 container APIs**, Surreal3.3 default, PostgreSQL18 restricted. See production-contract.md. This phase does not close either image/routing issue or qualify a release by source inspection.

Deliver ordinary chat current-user inline PNG/JPEG parts intact through an actual compatible adapter, with final primary/retry/fallback/graph sends constrained by captured authority and required capabilities. Use existing parser/runtime history, provider registry/router, Liter adapters, final request preparation, protected checkpoints and Cucumber program.

## Complete implementation ledger

| Item | Required implementation | Owner |
| --- | --- | --- |
| 1 | Strict current-user typed admission, safe fixed errors, decoder/type/size bounds | Ingress: server.rs, new llm/inline_image.rs |
| 2 | Private typed runtime input, all three canonical append paths, full text projection | Runtime: turn/request.rs, manager.rs |
| 3 | Canonical typed image preservation in protected history/checkpoints; generic redaction retained | Runtime: turn/host/event_projection.rs, checkpoint_host.rs |
| 4 | Shared live capability/enabled/cooldown evaluation; registered metadata cannot re-enter through catalog | Provider: llm/router.rs |
| 5 | Immutable actual bound exact/fallback scope and every final-send gate; nonempty explicit artifact fallback list governs capture | Runtime bindings/manager; provider orchestrator |
| 6 | Actual compatible-adapter construction fact, honest image accounting, downward registered output restriction; preserve existing exact preparations | Provider/router/orchestrator/liter/mod; runtime graph wrapper |
| 7 | Focused real HTTP Cucumber cases, minimal existing peer observation extension, deterministic assets and source-supported nonsecret fixture templates | Fixture owner: scripts/behaviors focused files, controlled_peer.py |
| 8 | OpenSpec deltas, support/version/recovery and release/behavior documentation with truthful evidence labels | Lead |
| 9 | All writer handoffs and settings/docs reconciled; whole-phase exact source inventory before any review/check/build/test | Lead |

At most four active agents including lead, one writer per file, lead owns all build/gate targets. Every command uses /Users/gqadonis/.claude/worktrees/production-readiness-476. Preserve original checkout/process/config work. Current source-review repairs R1–R5 are tracked in OpenSpec tasks.md and progress.json; all repairs and a renewed inventory must complete before gates reopen. Pre-review inventory001 and reports remain historical records.

## Behavior contract and explicit exclusions

Both /v1/chat/completions and /api/chat/completion support the last current user string or ordered text/image parts, including image-only. Images are standard padded-base64 PNG/JPEG with detail auto/low/high; at most8 per current turn within existing10MiB HTTP envelope. Reuse pinned xberg bytes-only decoder with OCR/cache/quality/postprocessing disabled, per-operation100MiB decoded/live allocation bound, two admitted workers without queue, owned lifecycle permit/lease and cooperative10second cancellation. A synchronous codec is joined on timeout: this is neither a hard response-time SLO nor total-process RSS bound.

Preserve nonblank convenience-message precedence for all-text dual inputs; blank falls back. Refuse discarded nontext history/ambiguous images explicitly. Retain text attachments and working direct native Anthropic paths, independently unqualified. No remote retrieval, upload-ID resolution, audio/video, full client multimodal replay, multimodal caller tools, new policy DSL/kernel/blob store/global runner or durability rewrite.

Actual captured adapter facts govern the new ordinary path; catalog declarations are not certification. Final admission rechecks actual tools, retained/current images, structured format and explicit enabled reasoning. Exact targets never substitute; nonempty artifact fallback lists bind before capture; empty-list legacy global behavior remains. Admission refusal is not provider outage or new fallback authority. Final registered output ceilings can only lower typed output and existing exact contract limits; they never create a counting contract.

Keep #388 endpoint/model/template/settings/protected groups/output/reasoning/per-fallback preparation. JSON/base64 token counts are not real-image capacity. Estimated image receipts report no fit guarantee; finite accounting requiring an unproven full input bound refuses before send. Existing explicitly synthetic exact contracts stay synthetic and retain their legitimate mutable fixture limits.

CD-08 cold checkpoint qualification is deferred to #469/#465: ordinary default graph has no CheckpointNode, and a real governed producer plus exact owned lifecycle is missing. Public strict-real-image fit is unsupported because no such public selector/counting contract exists. These source-informed frontier decisions preceded every test; no failing case was deleted or counted passed. Genuine image understanding needs a concrete qualified destination and applicable model-call budget authorization. No paid call, signing, publishing, deployment or remote builder is authorized.

## Acceptance and exit contract

- CD01/02: both aliases and streaming modes preserve mixed/multiple image order and independent byte hashes; image-only, literal/parts/JSON-looking text, attachments and legacy convenience precedence remain useful. Same comparator rejects swapped hashes.
- CD03: malformed/excluded source/type/base64/MIME/detail/decoder/count/envelope/attachment/ignored-history forms yield safe bounded errors with **zero actual chat arrivals** at every owned peer.
- CD04/05: actual public runs honor vision/tools/structured/context demand, exact target, disabled/cooldown and policy restrictions. Preference cannot override hard facts; no eligible route refuses without send. Real catalog overlay regressions are supplementary, not a namespace fixture claim.
- CD06: real controlled pre-output primary failure permits only captured capable fallback; protected image group/model/endpoint and fallback's own32token scalar output survive. Exact/incapable/outside fallbacks receive zero sends. Existing #388 exact preparation regressions remain affected checks.
- CD07: later image-free current turn retains canonical prior image hashes/order on the same owned warm session. No cold/restart claim.
- CD09: actual final-provider request-budget receipt reports approximate representation and fit_guarantee:false, tied to the observed destination. Same finite-cost policy first succeeds for text, then refuses unproven image liability before send. Attempt manifest alone cannot satisfy accounting.
- CD10: observed public diagnostics and sanitized receipts exclude raw image bytes/data URLs/private sentinel; sensitive comparator controls fail. Separate external log/Debug observations remain required for broader privacy claims.

Exit requires these mounted useful behaviors plus affected existing budget/text/tool checks, warning-free required compilation/build, truthful support labels and exact source/config/image/process/store receipts. Deterministic wire evidence is not visual understanding, whole-issue closure, recovery or release/platform certification. Missing external fixtures/artifact remain BLOCKED, not PASS.

## Serial gates and evidence

No verification during investigation/implementation or an unfinished repair cohort. Once all ledger/repair items, handoffs and docs are complete, record/present a renewed inventory; the current user authorization permits the planned bounded phase-exit gates without routine reapproval. Independent source review follows whole implementation. Lead runs serially; exact argv/environment and affected selectors are in execution.md:

1. Ordinary local commit hooks, then clean committed bash scripts/verify-rust-test-baseline.sh --acceptance (minimal, server-full, server-full,postgres-backend library harnesses), required affected history/embeddings/OpenAPI assertions.
2. Existing local Docker buildx help/builder inspection, then root Dockerfile --load Linux/amd64 image using an owned source-specific local tag and existing locked cache. Confirm local Unix backend before building. No new Dockerfile/workflow edit or native substitute for Linux evidence.
3. Focused existing Cucumber runner with capability-dispatch.cucumber.mjs and actual UAR_BASE_URL/UAR_WORKSPACE_ID/UAR_CAPABILITY_FIXTURE; external infrastructure owns identity, tests carry no credentials or authentication setup. Only owned records/peers/container/volume; no other chat runtime reuse.
4. Affected context_history_integrity integration and supplemental registered catalog regressions; no unrelated suites or cancelled historical runner.
5. pnpm support-matrix:validate and openspec validate carry-inline-images-and-constrain-dispatch; exact receipts/reflection and any normal spec synchronization only when exit is actually satisfied.

Use the established local Cargo build-directory workaround without changing global config or clearing caches. Retain initial failures and source identities. Fix one coherent affected cohort, renew its inventory, and rerun only affected gates. Status vocabulary: PASS, FAIL, BLOCKED, NOT_EXERCISED, HISTORICAL, UNVERIFIED. Successful build alone proves no runtime behavior.

Operational templates are preparation, not a booted fixture. The lead handles a separate owned local candidate/container/Surreal volume, controlled peers, source/dependency mounts and loopback-only host mappings where existing facilities support them. Peer default binding remains127; the explicit private-container bind option lets internal UAR wire URLs stay127 while host control ports publish only127. Existing host gateway facilities own identity; tests use only the supplied URL/workspace and no credentials or authentication setup. Actual source/image/config/process/namespace receipts remain required. Never retarget another chat service or weaken product boundaries.

## Program continuation

After this useful slice exits, select applicable P1 #468/#469, then #470/#471 based on the selected contract and existing transferred receipts. #467 applies only if AOT is promoted; current contract excludes it. #472–475 apply to actual capacity/telemetry/memory/team promises and require their own bounded plans. Keep #466 aligned continuously. #465 consumes actual release artifact/supported-lane evidence and #421 transferred obligations for final qualification. Triage unrelated findings separately.

Uncomfortable limit: all new mounted outcomes are currently NOT_EXERCISED. Historical native passes and99/102release counts cannot qualify this Linux artifact or its recovery/vision behavior.
