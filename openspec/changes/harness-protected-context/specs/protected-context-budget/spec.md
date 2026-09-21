## Purpose

Preserve authorized tool and task data while computing a truthful destination input budget and limiting compression to explicitly eligible prose.

## ADDED Requirements

### Requirement: Model-aware compaction retains task facts
Compaction triggers SHALL use resolved destination budgets. Model-specific retrieval chunking and placement SHALL require documented compatibility or measured evidence and SHALL preserve protected payloads, authority and required ordering. A frozen long-conversation evaluation SHALL compare task-fact retention, task success, cost and latency against the existing strategy using numerical acceptance criteria established before implementation.

#### Scenario: Long conversation and model switch
- **WHEN** a long conversation containing durable decisions, pending work and evidence is compacted and then switches to a compatible model
- **THEN** protected facts and valid history remain identical, measured outcomes meet the frozen criteria or block adoption, and actual summary cost/latency is reported

#### Scenario: Unproven placement preference
- **WHEN** a model-specific placement or chunking preference has neither documented support nor measured evidence
- **THEN** the existing compatible placement remains in use and no unsupported optimization is certified

### Requirement: Protection is explicit and conservative
Compression SHALL NOT summarize, truncate, delete, reorder or silently replace protected content with references. Protected content SHALL include tool calls and raw argument strings, tool results and parallel groups, mixed assistant tool messages, pending records, JSON/code/log/file data, multimodal references, required instructions, current input, evidence and citations, durable decisions and pending work. Unknown content SHALL be protected by default; only host-owned metadata SHALL mark a prose span eligible.

#### Scenario: Data in an ordinary text message
- **WHEN** history includes code, JSON, logs or file data in a user or assistant text message
- **THEN** role or text appearance alone does not make it compressible, and its protected payload remains identical

#### Scenario: Payload integrity oracle
- **WHEN** Unicode, CRLF, binary references and raw tool argument strings pass through all context strategies and persistence
- **THEN** protected record IDs, order, roles, provenance, raw-byte digests where available and typed values match the source; encoding changes are reversible and no newline-normalizing hash stands in for a raw-byte check

### Requirement: Budget calculation is pure and explicit
A budget function SHALL accept resolved destination limits, actual output reservation, protected request representation and eligible spans, without I/O or mutations, and SHALL return a fit/compression plan or typed overflow, invalid-limit or unsupported-count outcome. Let C be total context, L an independent input ceiling when present, H a host input ceiling when present, O the actual enforced output reservation, R additional reasoning occupancy only when not already included in O, and M explicit counting uncertainty. The input allowance SHALL be I = min(L, H, C - O - R - M), omitting absent L/H. Invalid negative values or reserve sums exceeding C SHALL fail explicitly. Let F be the complete rendered protected request cost, including fixed framing; only B = I - F is available for eligible prose and summary framing. F > I SHALL yield overflow.

#### Scenario: Reserves and independent input limit
- **WHEN** C=10000, L=6000, H=7000, O=2000, R=0, M=100 and F=5000
- **THEN** I=6000 and at most B=1000 remains for eligible prose including its framing

#### Scenario: Reasoning already included
- **WHEN** endpoint semantics include reasoning in its output cap
- **THEN** the enforced cap equals O, additional R is zero, and reasoning is not reserved twice

#### Scenario: Protected payload exceeds destination
- **WHEN** F exceeds I, including after switching to a smaller destination
- **THEN** preparation preserves the data and returns overflow or reselects an already-authorized compatible destination followed by fresh preparation; no truncation, summary or pointer substitution hides the overflow

#### Scenario: Invalid reservation
- **WHEN** a reservation is negative or O + R + M exceeds C
- **THEN** the function returns invalid limits rather than saturating arithmetic that disguises the deficit

### Requirement: Full final request must satisfy its bound
Final validation SHALL count the serialized destination request after all skill/world/evidence reattachment, normalization and driver adaptation. Counts SHALL include roles, tool names/IDs/arguments/results, schemas, structured-output framing, media, continuity blocks and cached input occupancy without a cache discount. Exact, validated upper-bound and approximate counts SHALL be distinguished. Dispatch SHALL require an exact count, validated upper bound or destination preflight establishing fit; an unbounded approximation SHALL NOT certify fit.

#### Scenario: Non-additive framing
- **WHEN** individually counted fragments fit but final serialization or reattachment makes the request exceed I
- **THEN** the complete request is rejected or eligible prose is replanned and recounted before dispatch

#### Scenario: Output ceiling propagation
- **WHEN** the budget reserves O output tokens
- **THEN** captured outbound parameters enforce the corresponding endpoint output ceiling and a missing or incompatible ceiling prevents dispatch

### Requirement: Summaries replace only eligible prose
A trusted host SHALL execute a pure plan's optional summarization through the existing governed model access boundary and charge its shared cost, latency and cancellation budget. Summarizer inputs SHALL contain only explicitly eligible contiguous prose spans and necessary non-sensitive framing, never protected bodies. Summaries SHALL replace only those spans at their original authority and position. Chunking SHALL be bounded and SHALL NOT recursively consume protected data.

#### Scenario: Protected blocks between prose
- **WHEN** eligible prose spans surround a tool group and evidence block
- **THEN** only prose enters the summarizer; original protected blocks remain in place and summary text is not elevated to system authority

#### Scenario: Failed summary
- **WHEN** summarization fails, is cancelled, returns empty/invalid output, or exceeds the allotted summary budget
- **THEN** originals remain intact and the host either proves the original request fits or returns an explicit failure; it does not use a truncation fallback

### Requirement: Authorization and retention precede preservation
Preservation SHALL NOT grant access or override retention/deletion policy. The trusted host SHALL reauthorize protected material before compression, each dispatch attempt and resume. Revocation SHALL invalidate prepared requests, caches and dependent summaries containing or derived from revoked material; where required data is unavailable the outcome SHALL be blocked and redacted. Revoked content SHALL NOT enter the model request, summarizer, user-visible audit or substitute summary.

#### Scenario: Revocation between attempts
- **WHEN** access to required evidence is revoked after preparation but before retry or resume
- **THEN** the old request is unusable, authorization is rechecked and a redacted blocked outcome replaces dispatch without leaking the evidence
