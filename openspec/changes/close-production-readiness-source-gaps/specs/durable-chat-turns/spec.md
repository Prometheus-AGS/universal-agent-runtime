## ADDED Requirements

### Requirement: Retain acknowledged and uncertain chat turns
Chat send/history SHALL retain a durable original turn identity, supported richcontent and truthful terminal/pending state across immediate reload/error/cancel without duplicate posts or authorization; persisted approval decisions SHALL remain visible and authoritative.

#### Scenario: Reload occurs before headers or during interrupted output
- **WHEN** Reload occurs before headers or during interrupted output
- **THEN** The accepted turn is recovered once or an explicit recoverable draft/unknown state remains, with partial metadata and prior decisions preserved

#### Scenario: A tool loop records several canonical assistant messages
- **WHEN** A run records an intermediate assistant message, a tool result and a final assistant answer, and the thread is reopened with either an existing aggregate cache or an empty cache
- **THEN** Every available canonical assistant segment retains its original identity and chronological position, including the final answer exactly once
- **AND** Presence of another message with the same run ID cannot suppress a missing canonical segment
- **AND** Available nontext evidence and settled approval history remain visible without duplicating run-level approval or usage cards

#### Scenario: Authoritative terminal follows a provisional transport interruption
- **WHEN** A run summary is locally interrupted and later receives qualified terminal evidence for that exact original native run
- **THEN** The provisional interruption is replaced by the authoritative completed, failed or cancelled outcome with its terminal time, usage and phase metadata
- **AND** Reattach alone proves no outcome, conflicting correlation refuses, already-authoritative terminal evidence remains immutable, and later weaker local events/replay cannot regress it

#### Scenario: A browser observer aborts before native settlement
- **WHEN** Local AbortError or cancel intent has no exactly correlated native terminal
- **THEN** Persisted and visible native outcome remains provisional rather than confirmed cancellation or success
- **AND** Reattachment may accept the exact original native success, error or cancellation once, while authoritative terminal rows remain immutable

#### Scenario: The native chat host reports confirmed cancellation
- **WHEN** The shipped native chat adapter receives the actual profiled RUN_ERROR with code=CANCELLED from agui_spec
- **THEN** Run persistence, message finalization and continuation feedback preserve cancellation rather than success
- **AND** Available native cancellation usage reaches the wire and consumers without inventing missing counters; original evidence and cursor remain unchanged
- **AND** The separate unprofiled standard AG-UI endpoint is not silently treated as this native chat transport

#### Scenario: Cancellation carries independently available provider usage
- **WHEN** Native cancellation retains a partial or complete provider usage report
- **THEN** The native wire and persisted event/run summary retain each available count, model and cost independently
- **AND** Missing usage stays missing and known zero stays zero; a display that requires complete counters omits its numeric summary instead of inventing the missing values

#### Scenario: The actual chat Stop callback races native settlement
- **WHEN** The user invokes useChatRuntime Stop before correlated native terminal evidence, including a failed or delayed cancellation request
- **THEN** Original assistant content persists as provisional interruption rather than confirmed cancellation
- **AND** Later exact native success, error or cancellation reconciles the same original assistant message without duplicate sends, bubbles or effects
- **AND** A late local Stop cannot overwrite a genuinely confirmed native terminal

#### Scenario: Native evidence is rejected before reduction
- **WHEN** A profiled frame conflicts with the attached native run identity
- **THEN** It cannot update current-run content, usage, phases, messages or terminal/ChatTurn settlement
- **AND** A distinct same-run persistence failure keeps original content provisional and the turn recoverable, without claiming durable settlement or dispatching again
- **AND** Later accepted original terminal evidence may reconcile the same message exactly once

#### Scenario: Newer native terminal evidence is local when the server is unavailable
- **WHEN** Cached assistant messages are interrupted but matching later native text and terminal evidence have already been durably stored before a reload
- **THEN** Local loading reconciles provisional content and status using that evidence without requiring server history or dispatching again
- **AND** Original message IDs, rich segment boundaries, chronology and settled rows remain intact; repeated loading cannot duplicate text or chunks

#### Scenario: Successful usage is partially unavailable
- **WHEN** A successful native terminal reports only some usage counters, or no usage report
- **THEN** Chat and Runtime Console projections preserve unavailable counters rather than zero-fill or derive an unknown aggregate
- **AND** Known zero and complete reports remain exact, while independently available evidence survives locally even if the numeric display requires a complete report

#### Scenario: Native artifact publication reaches the original input identity
- **WHEN** Native profiled artifact display or input-request CUSTOM events use canonical artifactId and artifactType fields
- **THEN** The shipped adapter, runtime/message persistence and renderer preserve original ID, type, content and publication metadata across their existing vocabulary boundaries
- **AND** Input submission uses the original backend artifact ID rather than a local card ID, while legacy working forms and existing execution guards remain intact

#### Scenario: A valid host context event is intentionally runless
- **WHEN** The actual native producer stores a profiled runless host context extension under the attached run between matching start/text/terminal evidence
- **THEN** Local replay preserves that original payload and can reconcile the exact owning run without treating the host extension as terminal or surface authority
- **AND** Run-bearing content/action/terminal evidence and outer run/event/type/sequence ownership remain strict; arbitrary missing or foreign identity does not gain authority

#### Scenario: Warm canonical text is merged into cached rich history
- **WHEN** A cached assistant segment contains introductory text, retained rich evidence and later text, and the same canonical message returns text-only model history
- **THEN** Warm backfill joins canonical text at the existing rich anchors instead of moving all retained cards ahead of the introduction
- **AND** Original identities, separate final-answer messages, rich metadata/decisions and ordered persisted reload survive repeated backfill without duplicate text, cards or effects

#### Scenario: Completed action recovery replays an already-settled continuation
- **WHEN** An incomplete local action receipt reads a completed original continuation and reattaches from cursor zero after its chat message is already saved
- **THEN** Observing replay cannot append duplicate text/cards, clear content, change original IDs/accounting or downgrade settled chat evidence if observation ends early
- **AND** Successful repeated recovery can finish the original receipt without new dispatch, while legitimate provisional/new streaming remains functional

#### Scenario: Same caption does not prove a different image prompt is the same user message
- **WHEN** Local and canonical user identities differ and their captions match but image content/references or order differ, including empty-caption image-only prompts
- **THEN** Backfill refuses the unsupported join and retains the original cached image identity rather than overwriting it under the old ID
- **AND** Exact stable identity and demonstrably safe legacy text-only or supported normalized media controls retain their defined behavior

#### Scenario: Hosted graph preparation preserves checkpoint denial behavior
- **WHEN** an owned graph run prepares its captured thread kernel before raw, revoked or legacy checkpoint-resume attempts
- **THEN** hosted preparation retains the same completion/cancellation ownership and all live root, owner, persisted binding and budget checks under the unchanged executor
- **AND** the original graph can reach its capture node while invalid resume attempts refuse without provider dispatch or warm-history mutation, preserving the existing capture bound and complete assertions
