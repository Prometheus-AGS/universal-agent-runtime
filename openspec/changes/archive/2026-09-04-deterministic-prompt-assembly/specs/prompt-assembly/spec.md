## ADDED Requirements

### Requirement: Prompt sections are typed fragments in a fixed order
The runtime SHALL assemble the model prompt from typed fragments, each carrying an id, source, authority, role, retention, and content hash, rendered in a fixed section order: agent identity, enforced policy summary, host and project instructions, skill catalog, active skill bodies, world-state changes, memory and retrieval, then conversation history and the current input.

#### Scenario: Identical inputs render identically
- **WHEN** two turns are assembled from the same artifact, the same eligible skills in any discovery order, and the same retrieval hits
- **THEN** the rendered system prompt is byte-identical and every fragment hash matches

#### Scenario: Within-section ordering is deterministic
- **WHEN** several fragments belong to the same section
- **THEN** they are ordered by fragment id, not by discovery order

### Requirement: Fragment authority is explicit
Every fragment SHALL carry an authority of `System`, `Policy`, `Host`, `Skill`, `Retrieved`, or `User`, and retrieved documents, attachments, and skill bodies SHALL be rendered inside markers that identify them as data rather than instructions.

#### Scenario: Retrieved chunk cannot masquerade as policy
- **WHEN** a retrieved chunk contains text formatted like a system instruction
- **THEN** it is rendered inside `Retrieved` markers with `Retrieved` authority and the policy fragment is unchanged

### Requirement: Every run records a redacted turn manifest
The runtime SHALL record, for each turn, a manifest of fragment ids, hashes, counts, budgets, provenance, selected skills and tools, and warnings, SHALL store it in the run context, SHALL emit it as an additive artifact, and SHALL NOT include prompt bodies, credentials, hidden reasoning, or raw retrieved content in it.

#### Scenario: Manifest is emitted alongside the effective policy artifact
- **WHEN** a run starts
- **THEN** clients receive the existing effective-policy artifact and a `turn_manifest` artifact, and the manifest carries no fragment body text

### Requirement: Artifact instructions are rendered
Instructions declared on an agent artifact SHALL be rendered as a host-authority fragment; the runtime SHALL NOT accept an instructions field it never reads.

#### Scenario: Artifact with instructions
- **WHEN** an artifact declares one or more instructions
- **THEN** they appear in the host-and-project-instructions section of the rendered prompt
