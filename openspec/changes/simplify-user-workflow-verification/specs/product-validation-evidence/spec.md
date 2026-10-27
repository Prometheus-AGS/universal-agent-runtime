## ADDED Requirements

### Requirement: User workflows have independent admission
The verification runner SHALL admit a selected workflow/platform/backend cell using only its declared prerequisites. An unavailable fixture or platform SHALL block only dependent cells. The full coverage registry SHALL retain every advertised capability and map every prior acceptance/supporting obligation, with explicit reasons for deduplication or non-applicability.

#### Scenario: Knowledge or mobile preparation is unavailable
- **WHEN** the configured-prompt cell has its real provider, supported store and actual platform artifact but KB or mobile preparation is unavailable
- **THEN** the configured-prompt cell can run under explicit execution authority and the unavailable dependent cells remain BLOCKED
- **AND** their absence is not counted as PASS or removed from the denominator

### Requirement: Useful outcomes and failure collection remain truthful
Positive functional claims SHALL use genuine configured inference and task-specific observable predicates. Results SHALL preserve initial failures, cleanup evidence and exact source/artifact cohorts. At most one bounded authorized correction and same-item rerun SHALL occur; independent work SHALL continue when its prerequisites and resource safety permit.

#### Scenario: A cell fails and its correction does not resolve it
- **WHEN** the cell fails and its single authorized correction/rerun still fails
- **THEN** both attempts remain recorded, an unresolved failure is logged, and independent cells continue
- **AND** a changed-source PASS never certifies a previous or unexercised source cohort

### Requirement: Simplification does not create authority or erase history
The child SHALL preserve parent evidence, cumulative consumption, UNKNOWNs and separate provider ceilings. The replacement policy and exact commands SHALL require explicit operator adoption/approval after implementation; planning or child activation SHALL not open execution.

#### Scenario: A child phase starts while parent execution is stopped
- **WHEN** the verification redesign child is activated
- **THEN** design work proceeds with execution CLOSED and previous consumption/failures retained
- **AND** no signature, test, build, service or provider request is implied

### Requirement: Complete platform sweeps in operator order
Verification SHALL finish the complete applicable macOS sweep before Linux, then finish Linux before iOS. Within-platform independent cases SHALL continue after logged failures when their prerequisites and resource safety permit. Every runnable cell SHALL be attempted and every blocked dependent explicitly recorded before advancement. Sweep completion SHALL not be reported as platform certification when required failures remain.

#### Scenario: macOS reveals a shared product defect
- **WHEN** the macOS sweep exposes a shared defect and an authorized bounded correction is made
- **THEN** its original failure and rerun remain recorded and later Linux/iOS artifacts use the corrected source
- **AND** the remaining applicable macOS checks finish before Linux begins

#### Scenario: A later platform fix changes previously verified behavior
- **WHEN** a Linux or iOS correction changes shared behavior previously exercised on an earlier platform
- **THEN** affected earlier-platform claims require targeted regression evidence or remain explicitly unverified for the changed source
- **AND** unchanged platform-independent evidence is not needlessly rerun and platform runtime results never transfer

### Requirement: Functional scenarios do not perform authentication
Manual functional verification SHALL use an externally provisioned identity endpoint and SHALL NOT require, issue, renew or send authentication credentials from the test scripts. Nonsecret model, endpoint and workspace selection remain supported. The external runtime host preserves owner/tenant context for persistent records; production identity boundaries are not disabled.

#### Scenario: Team behavior without test credentials
- **WHEN** the existing math/writing BDD runs with only endpoint, model, workspace and report settings
- **THEN** external infrastructure supplies owner context without test-managed authentication
- **AND** the original concurrency, arithmetic, editorial result, accounting and merged-article assertions remain unchanged

### Requirement: Manual conversation continuity uses server-held history
The selected conversation-continuity BDD SHALL send two separate requests to the saved mathematics agent in one fresh session. The follow-up SHALL require facts and a result from the first exchange without the client replaying prior messages. Both answers SHALL match independently computed arithmetic and the saved agent identity.

#### Scenario: Follow-up remembers the original calculation
- **WHEN** a user asks the saved mathematics agent to add two fresh numbers and then sends only a follow-up in the same session asking it to transform its previous answer and subtract the original second number
- **THEN** both final responses contain the correct numeric answers and saved specialist identity
- **AND** distinct completions retain the same session, request/response evidence is saved, and the existing UAR stays running
- **AND** the result claims warm conversation continuity only, without claiming restart durability or cross-session memory

### Requirement: Manual verification proves built-in deletion protection
The selected protected-agent BDD SHALL attempt deletion of the existing default and orchestrator agents, require the explicit built-in protection refusal, and confirm their complete definitions remain unchanged.

#### Scenario: Default and orchestrator deletions are rejected
- **WHEN** DELETE requests target default-agent and orchestrator-agent through the existing agent API
- **THEN** each returns HTTP403 with the specific built-in deletion protection message
- **AND** subsequent GET requests return both unchanged definitions
- **AND** no model call, service restart, test-managed authentication or previously verified scenario is required

### Requirement: Custom instruction updates affect execution before deletion
A manual custom-agent lifecycle scenario SHALL use a new disposable agent, verify its original instructions through inference, persist different instructions, and prove a fresh conversation uses the new instructions before deleting that agent.

#### Scenario: Change a disposable agent's rule and remove it
- **WHEN** a disposable agent's arithmetic rule and random response marker are changed through the agent API
- **THEN** a readback contains the updated instructions and the same user prompt in a fresh session returns the new marker and independently computed result
- **AND** neither the new rule nor marker is supplied in the user prompt
- **AND** deletion returns204 and a subsequent definition read returns404
- **AND** existing agents and services remain intact, no previous scenario reruns, and no authentication credentials are supplied by the test

### Requirement: Anthropic-compatible prompting uses the saved agent
The selected manual Anthropic scenario SHALL prompt the existing mathematics agent through `/v1/messages` using the configured genuine model, and require a correct final answer with the identity supplied only by the agent's saved instructions.

#### Scenario: Nonstreaming Messages prompt to the saved math agent
- **WHEN** a fresh arithmetic question is sent to `/v1/messages` with the saved mathematics agent explicitly selected
- **THEN** the API returns an Anthropic assistant message with text content, the requested model and an end_turn completion
- **AND** its final answer contains the saved specialist marker and independently calculated numeric result
- **AND** raw evidence is retained without test credentials, agent mutations, service restarts or reruns of prior scenarios

### Requirement: A2A prompting executes the saved mathematics agent
The selected manual A2A scenario SHALL send one fresh arithmetic question to the saved mathematics agent using A2A v1 SendMessage and verify a completed task containing the correct answer and saved identity.

#### Scenario: Saved-agent A2A task returns the correct answer
- **WHEN** SendMessage targets the saved mathematics agent with a fresh arithmetic question and returnImmediately false
- **THEN** the correlated JSON-RPC response contains a completed task for that agent and a matching agent-message task/context identity
- **AND** its final answer contains the specialist marker from saved instructions and the independently calculated result
- **AND** raw evidence is retained without test credentials, model or agent changes, service restarts, or prior scenario reruns

### Requirement: ACP prompting executes the saved mathematics agent
The selected manual ACP scenario SHALL create a session for the saved mathematics agent and send one fresh arithmetic question through the ACP API using its unchanged genuine configured model.

#### Scenario: Saved-agent ACP run answers correctly
- **WHEN** an ACP session targets the saved mathematics agent and runs/create submits a fresh arithmetic question through the ACP stream
- **THEN** answer content and a successful terminal event share the run identity, and ACP runs/get confirms completion for the selected agent
- **AND** the final answer contains the specialist marker from saved instructions and the independently computed numeric result
- **AND** raw evidence is retained without test credentials, agent/model changes, restarts or reruns of previously verified scenarios

### Requirement: AG-UI streams the saved mathematics agent answer
The selected manual AG-UI scenario SHALL prompt the existing mathematics agent and assemble its genuinely streamed response using the unchanged configured model.

#### Scenario: Saved math agent returns a correct streamed answer
- **WHEN** a fresh arithmetic question targets the saved math agent through AG-UI
- **THEN** multiple text fragments occur between correlated assistant-message start/end events and the run/thread completes successfully
- **AND** the assembled final answer matches independent arithmetic and contains the identity supplied only by saved instructions
- **AND** raw evidence is retained without test credentials, agent changes, restart or reruns of previously verified scenarios

### Requirement: AG-UI delivers a genuinely published A2UI response
The selected manual A2UI scenario SHALL request a fresh greeting surface from a real configured agent through the agui_spec stream mode and require host-published A2UI messages, not prose approximations.

#### Scenario: Native renderer publishes the requested greeting surface
- **WHEN** a disposable agent calls a2ui_render for a fresh greeting through /v1/chat/completions with agui_spec streaming
- **THEN** correlated successful renderer output, ordered surface/component STATE_DELTA events and a published A2UI artifact contain the requested greeting and surface identity
- **AND** the run completes successfully and only the owned disposable agent is removed
- **AND** raw evidence is retained without test credentials, prior scenario reruns or a browser-display claim

### Requirement: A rendered A2UI user action reaches the genuine agent
The selected manual browser scenario SHALL display a genuinely generated A2UI calculator in the shipped browser UI and verify a user action causes a real agent continuation.

#### Scenario: Fresh browser input produces the correct continued answer
- **WHEN** a disposable configured agent publishes an A2UI form in the browser and the user enters a fresh number and clicks its rendered action
- **THEN** the actual browser action request carries that number and receives a distinct continuation run
- **AND** the continuation returns the independently computed result and saved agent marker, visible in the browser
- **AND** raw evidence, screenshots and video are retained, only the disposable agent is deleted, and no credentials or earlier scenarios are used

#### Scenario: Renderer tool advertises the browser-supported catalog
- **WHEN** the model requests a2ui_render with UAR's urn:uar:a2ui:catalog:1 catalog
- **THEN** the tool schema accepts it consistently with the backend parser and browser renderer
- **AND** the existing basic catalog URL remains accepted by the backend, while arbitrary catalogs remain rejected

### Requirement: Skill detection is evidenced by selection and actual use
The manual scenario SHALL require runtime matching and activation evidence as well as an answer applying instructions stored only in a skill. It SHALL preserve the configured strategy and earlier scenario results.

#### Scenario: Parcel request selects the matching skill
- **GIVEN** a disposable agent, a parcel skill with a private pricing recipe and marker, an unrelated gardening skill, and service keyword matching
- **WHEN** the user requests a parcel quote without naming or attaching a skill
- **THEN** run inspection and activation events identify the parcel skill and exclude gardening
- **AND** genuine inference returns the independently calculated price and skill-only marker
- **AND** disposable resources are removed, raw evidence is retained, and no earlier scenario reruns

### Requirement: Semantic skill evidence excludes exact keyword selection
The selected semantic BDD SHALL require a paraphrased request with zero exact trigger matches, explicit semantic matching evidence, and genuine use of private skill instructions. Temporary matching settings SHALL be restored.

#### Scenario: Delivery paraphrase selects the intended skill
- **GIVEN** equally eligible disposable delivery and gardening skills and local embedding matching
- **WHEN** the user describes a delivery-pricing task without a skill ID, attachment or exact trigger keywords
- **THEN** run inspection identifies local embedding matching and the delivery skill is selected and activated
- **AND** the answer applies the private pricing recipe and marker absent from the user and agent prompts
- **AND** the original matching configuration is restored and disposable resources are deleted
- **AND** no previously verified scenario is rerun

### Requirement: Context budgeting preserves required follow-up information
The manual scenario SHALL exceed a bounded input allowance, demonstrate actual context reduction and a bounded final request, and answer a follow-up using earlier facts absent from the new request.

#### Scenario: Keep first and latest within a small context window
- **GIVEN** a disposable agent using truncate-middle retention, an8192-token context window and2048-token output reserve
- **WHEN** the user supplies fresh reference/quantities, then adds oversized irrelevant context and asks a dependent follow-up without replaying the facts
- **THEN** source history exceeds the input allowance and reduced history fits it
- **AND** final serialized request accounting and reported provider usage fit the declared budgets
- **AND** the answer contains the original reference and independently correct sum
- **AND** the original model settings are restored, the disposable agent is deleted, and prior verified scenarios are not rerun

### Requirement: Summarization preserves facts through actual replacement
The manual summary scenario SHALL prove genuine summarization of the fact-bearing turn, rather than infer it from a strategy label or correct recall alone.

#### Scenario: Recall a booking after summarization
- **GIVEN** a disposable agent using summarization and a long user turn containing fresh booking facts
- **WHEN** a follow-up requests those facts without repeating them or replaying the conversation
- **THEN** completed summary receipts bind the fact-bearing source, and changed prepared-context hashes/counts demonstrate compression
- **AND** the final answer preserves the exact reference, derived headcount and delivery constraint
- **AND** disposable cleanup succeeds and previously verified scenarios are not rerun

### Requirement: Durable conversation survives a process restart
The manual scenario SHALL prove recall after the previous owned UAR process exits and a new process starts with the same binary, configuration and store. Client transcript replay SHALL NOT substitute for persisted conversation recovery.

#### Scenario: Continue a conversation after a graceful restart
- **GIVEN** a completed exchange containing fresh user facts in a new session
- **WHEN** the owned Mac runtime restarts without resetting its existing store
- **AND** the client sends a new follow-up containing only references to earlier facts and the same session identity
- **THEN** the agent definition remains persisted and the response includes the original reference and correctly derived total
- **AND** process identity/configuration evidence and raw exchanges are retained
- **AND** disposable cleanup succeeds, UAR remains running, and prior verified scenarios are not rerun

The normal chat host MUST save a successful conversation before reporting terminal completion and restore owner-scoped history on a cold session lookup. A configured store read or write failure MUST surface as a run error rather than silently claiming durable continuity.

#### Scenario: Concurrent conversation turns survive graceful restart
- **WHEN** two genuine turns in the same fresh conversation overlap and complete successfully, followed by a graceful UAR restart with the existing store
- **THEN** a follow-up using only the conversation identifier and question MUST recover distinguishing facts from both turns and their derived sum
- **AND** evidence MUST bind both run identities, observed start-before-terminal overlap, the same session, old/new process identities, unchanged store/config/binary, raw responses and exact assertions
- **AND** the receipt MUST limit the claim to the exercised interleaving rather than exhaustive concurrent durability

#### Scenario: REST user preference survives graceful restart
- **WHEN** a user changes `prompt_caching_enabled` through the mounted REST API and UAR restarts with the existing durable store
- **THEN** a fresh GET MUST return the changed saved record, including its unchanged update timestamp
- **AND** evidence MUST establish a genuinely different value, actual owned process replacement, unchanged binary/config and no request replay after restart
- **AND** the manual scenario MUST restore the original nullable preference and retain raw REST exchanges without supplying test credentials

#### Scenario: Retained agent knowledge remains usable after restart
- **WHEN** UAR restarts with an existing agent, bound indexed knowledge base and document intact
- **THEN** a fresh-conversation prompt to that agent MUST recover the correct specialized mathematics and the document-only retrieval seal
- **AND** an actual retrieval citation MUST match the retained knowledge-base ID, document ID, seal-bearing snippet and answer marker
- **AND** evidence MUST show unchanged binding/index metadata and embedding space across owned process replacement without creating, uploading, rebinding or reindexing knowledge

### Requirement: Removing one KB link restricts knowledge without deleting either source
The manual scenario SHALL unlink exactly one of two persisted KB references while keeping knowledge enabled and verify genuine inference uses only the remaining source.

#### Scenario: One of two KBs remains selected
- **GIVEN** two indexed KBs with distinct private facts and a disposable agent with both references saved
- **WHEN** only one reference is removed and the agent is asked about both sources in a fresh conversation
- **THEN** its answer contains the remaining source fact and matching runtime citation, while the removed source fact is unavailable and absent from citations
- **AND** both original KB documents remain directly searchable and unchanged
- **AND** only disposable resources are cleaned up, raw evidence is retained, and previously verified scenarios are not rerun
