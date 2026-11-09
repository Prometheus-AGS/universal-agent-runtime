## ADDED Requirements

### Requirement: Admit logical actions durably
Trusted host action admission SHALL bind a stable interaction identity to verified owner/thread/surfacegeneration/component/action and payloaddigest; identical retries reuse one durable receipt, conflicting/stale/foreign submissions refuse, unknowneffects never auto replay. Bounded surface replay SHALL expose incompleteness and prevent stale authority.

#### Scenario: One logical click is retried through a fresh transport ID after response loss
- **WHEN** One logical click is retried through a fresh transport ID after response loss
- **THEN** The same original admission is reconciled without second effects, while a distinct intentional click has its own identity

#### Scenario: Settled original action is reconciled after restart
- **WHEN** The same verified owner reconciles or identically retries a previously admitted action after its original source run is no longer resident
- **THEN** Exact durable owner, source agent and conversation evidence permits retrieval of the original immutable receipt and continuation identity without dispatch
- **AND** Conflicting semantic input or a foreign owner/source binding refuses, while a new action still requires current executable presentation authority

#### Scenario: Durable nonterminal history has no live producer
- **WHEN** A REST or AG-UI action receipt refers to a persisted Pending, Running or Paused continuation without a resident producer
- **THEN** Its public outcome is uncertain rather than running, and the original immutable claim cannot be replayed
- **AND** A genuinely resident active continuation remains running and authoritative terminal results retain their terminal outcomes

#### Scenario: The rendered action generation is older than current authority
- **WHEN** A control rendered from generation G1 is clicked after the same surface/component/action identity has advanced to G2
- **THEN** The original rendered G1 generation remains bound to the persisted logical intent and is refused as stale rather than replaced with G2
- **AND** Missing generation is unavailable for execution, matched generation remains unchanged, and retry identity, owner checks and no-replay safeguards remain authoritative

#### Scenario: A deployment-selected AG-UI action reconciles its original continuation
- **WHEN** The wire deployment selector differs from the admitted native artifact ID and the original continuation is resident or durably settled
- **THEN** Recovery joins the exact original durable AG-UI binding and admitted native identity while preserving the immutable wire receipt key
- **AND** Original owner, source, conversation, interaction and continuation mismatches refuse or remain uncertain, without resolving the current mutable catalog
- **AND** Identical retries retain the original receipt and never dispatch another effect; nonresident nonterminal history stays uncertain

#### Scenario: Surface authority fails after an action becomes uncertain
- **WHEN** An admitted action is pending, responding, uncertain or interrupted and surface authority is missing, stale or unavailable
- **THEN** Immutable action outcome feedback remains visible independently of authority feedback, including the uncertain-effect do-not-resubmit warning
- **AND** Existing execution guards and read-only recovery remain unchanged

#### Scenario: A newly durable surface was not yet cached before reload
- **WHEN** An interrupted cached text message is followed by a native journaled A2UI surface with nested metadata/profile/source/publication generation and the server is unavailable on reload
- **THEN** Local recovery preserves the original metadata and portable representation and routes the surface through the shipped A2UI display path rather than a generic source card
- **AND** Missing generation remains read-only; recovery cannot fetch or manufacture a replacement authority token
