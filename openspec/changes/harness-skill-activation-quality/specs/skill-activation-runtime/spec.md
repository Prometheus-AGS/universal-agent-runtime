## MODIFIED Requirements

### Requirement: Activated skill bodies survive compaction under a budget
The most recent authorized activation of each active skill SHALL retain an immutable body/version identity. Prompt projections SHALL be reconstructible from that canonical source; compaction SHALL NOT summarize or destroy it. Required active bodies SHALL be reattached within the configurable skill budget before the complete destination request is counted. Insufficient budget SHALL yield explicit overflow or authorized reselection, never silent omission of an active required skill.

#### Scenario: Compaction with an active skill
- **WHEN** compaction runs while a skill is active
- **THEN** its exact authorized body/version is reattached after the summary at its declared fragment position and included in the skill and final request budgets

#### Scenario: Activation changed or revoked
- **WHEN** an active skill version changes or access is revoked during retry/resume
- **THEN** the old preparation is invalidated and current authorization/version policy determines explicit reactivation or a blocked outcome

## ADDED Requirements

### Requirement: Activation respects scope and explicit intent
Discovery and activation SHALL preserve conversation > agent > global scope precedence, explicit invocation and permission constraints. Implicit ranking SHALL NOT override an eligible explicit selection or bypass authorization. Existing candidate reduction SHALL remain shadow-only until its Recall@10 is at least 99 percent against the frozen exhaustive oracle.

#### Scenario: Same skill in multiple scopes
- **WHEN** matching skill identities exist in conversation, agent and global scopes
- **THEN** the conversation definition wins, then agent when conversation is absent, then global; the selected version and source are attributable

#### Scenario: Explicit but forbidden activation
- **WHEN** a user explicitly names a skill that policy forbids
- **THEN** the runtime reports governed rejection instead of silently substituting or activating the forbidden skill

### Requirement: Skill quality is measured by outcomes
Before activation behavior implementation, the local evaluation contract SHALL freeze dataset version, labels, exhaustive retrieval oracle, baseline and numerical thresholds for precision, recall, unnecessary activation rate, task success, latency and prompt-token overhead. Cases SHALL cover positive, negative, ambiguous, explicit invocation, scoped collisions and long contexts. Tool use SHALL be reported separately from task success and prompt-only skill use SHALL remain attributable.

#### Scenario: Labeled activation evaluation
- **WHEN** baseline and proposed activation behavior run on the same frozen cases
- **THEN** the report includes every declared metric and threshold, counts prompt-only use and actual tool use separately, and fails adoption on unmet required thresholds

#### Scenario: No frozen acceptance contract
- **WHEN** dataset labels or numerical thresholds are missing
- **THEN** implementation/adoption remains blocked; a post-hoc threshold or tool-use count cannot certify success

