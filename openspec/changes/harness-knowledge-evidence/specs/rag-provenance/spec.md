## ADDED Requirements

### Requirement: Evidence is authorized at use
Retrieval, model context assembly, retry, resume and evidence presentation SHALL use current owner/tenant authorization. Access revocation SHALL invalidate prepared requests, dependent summaries and cached evidence projections. Retrieved content SHALL remain untrusted data and SHALL NOT grant tools, privileges or instruction authority. Audit projections SHALL preserve authorized provenance without raw secret or unauthorized content.

#### Scenario: Revoked evidence between retrieval and use
- **WHEN** a source is retrieved and its access is revoked before dispatch or resume
- **THEN** the source and derivatives cannot reach the model or user-visible audit, and required missing evidence produces a redacted blocked outcome

#### Scenario: Retrieved prompt injection
- **WHEN** a document instructs the model to ignore policy or execute a tool
- **THEN** it remains a source quotation/data item, no instruction authority changes, and tool execution still passes host policy

### Requirement: Evidence verdicts distinguish support and uncertainty
A knowledge answer SHALL link factual claims to authorized source/chunk/version evidence and record a supported, insufficient, conflicting or stale verdict with an inspectable rationale. Lexical overlap or a citation alone SHALL NOT prove factual support. A source update or conflict SHALL be represented without silently presenting an obsolete claim as verified. Deterministic access and integrity checks SHALL remain hard gates independent of any advisory model judge.

#### Scenario: Conflicting sources
- **WHEN** authorized sources disagree about a requested fact
- **THEN** the answer identifies the conflict and supporting source versions rather than declaring one unqualified verified fact

#### Scenario: Unsupported or stale answer
- **WHEN** the answer contains a fact unsupported by current evidence or relies on a superseded source
- **THEN** its verdict is insufficient or stale and the response exposes that limitation with authorized provenance

