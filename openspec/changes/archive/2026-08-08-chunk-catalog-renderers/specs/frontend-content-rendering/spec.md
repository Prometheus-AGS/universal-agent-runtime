## ADDED Requirements

### Requirement: Shared ContentBlock protocol remains portable and exhaustive
The frontend SHALL define the portable `ContentBlock` wire and storage contract as the exact discriminated union of text, thinking, code, citation, memory, toolUse, toolResult, skill, artifact, image, and divider variants, and every projection over that union MUST fail compilation when a variant is unhandled.

#### Scenario: Every portable variant projects
- **WHEN** one instance of every `ContentBlock` variant is passed to the shared projection
- **THEN** each block MUST produce the corresponding typed view chunk or intentional spacer treatment
- **AND** no variant MUST be silently dropped

#### Scenario: New portable variant is unhandled
- **WHEN** a new discriminant is added to the portable union without updating the projection
- **THEN** TypeScript compilation MUST fail at the exhaustive switch

#### Scenario: Historical local content is loaded
- **WHEN** PGlite returns a message written with the previous partial local content shape
- **THEN** the persistence boundary MUST decode every known legacy discriminant into canonical blocks or typed runtime chunks
- **AND** the transcript MUST remain readable without widening the canonical wire union

### Requirement: Complete runtime Chunk catalog uses one typed model
The frontend SHALL define every §8 runtime chunk kind in one discriminated `Chunk` union and SHALL use exhaustive phase, bubble-visibility, renderer, and trace mappings for that union.

#### Scenario: Complete catalog is mapped
- **WHEN** the catalog conformance fixture enumerates text, markdown, reasoning, thinking, tool-call, tool-approval, tool-denied, skill-activation, memory-recall, memory-mutation, memory-update, citation, rag-citations, context-update, a2ui-display, a2ui-input, artifact, image, video, file, state-snapshot, state-delta, step, usage, error, and raw chunks
- **THEN** every kind MUST have a renderer disposition, phase disposition, bubble-visibility disposition, and trace disposition

#### Scenario: Runtime catalog grows
- **WHEN** a new `ChunkKind` is introduced without updating one of the required maps
- **THEN** TypeScript compilation MUST fail for the incomplete record

### Requirement: AG-UI and persisted inputs converge on the same chunks
The frontend SHALL normalize portable persisted blocks and validated official, custom, pseudo-tool, and raw AG-UI inputs into the same `Chunk` union with stable run/message identity and ordering.

#### Scenario: Known runtime event arrives
- **WHEN** a known official or UAR custom event is ingested
- **THEN** the normalizer MUST emit its documented typed chunk with the durable run id and sequence
- **AND** chat, trace, and inspector consumers MUST share that semantic projection

#### Scenario: Unknown custom or raw event arrives
- **WHEN** a validated CUSTOM name is unknown or a RAW event is ingested
- **THEN** the normalizer MUST emit a `raw` chunk that preserves the payload for trace and inspector use
- **AND** the raw payload MUST remain hidden from the conversation bubble by default

#### Scenario: Tool result follows tool use
- **WHEN** a toolResult block or event references a prior toolUse id
- **THEN** the projection MUST attach the result and final status to that logical tool-call chunk
- **AND** MUST NOT render a duplicate unrelated tool call

### Requirement: Rich chunks register as Assistant UI data parts
The frontend SHALL register stable named Assistant UI data-part renderers for rich chunk families and SHALL emit rich message parts through those registrations rather than encoding them as pseudo-tool calls.

#### Scenario: Rich persisted message rehydrates
- **WHEN** a completed message containing memory, citation, skill, context, artifact, or approval chunks is restored
- **THEN** Assistant UI MUST resolve each named data part through the registered renderer
- **AND** native text and reasoning parts MUST preserve their existing streaming behavior

#### Scenario: Trace-only chunk reaches a message
- **WHEN** a state snapshot, state delta, step, or raw chunk is associated with a message
- **THEN** its bubble renderer MUST return no visible prose
- **AND** the chunk MUST remain available to the trace or inspector projection

### Requirement: Chunk renderers use Flat 2.0 and accessible semantics
Every bubble-visible chunk SHALL use filled surface levels, spacing, text, and state labels without decorative borders or shadows, and interactive controls SHALL expose keyboard and assistive-technology semantics.

#### Scenario: Protocol divider renders
- **WHEN** a divider block is rendered
- **THEN** the DOM MUST contain a spacing `<div role="separator">`
- **AND** MUST NOT contain an `<hr>` element or visible rule

#### Scenario: Secondary detail starts collapsed
- **WHEN** completed reasoning, thinking, tool details, or source detail first renders
- **THEN** secondary content MUST be collapsed by default
- **AND** its disclosure control MUST expose the expanded state

#### Scenario: Status is conveyed
- **WHEN** a skill, memory operation, tool, approval, denial, or error state renders
- **THEN** the state MUST be written in visible text
- **AND** MUST NOT depend on color alone

### Requirement: Artifacts and media preserve established trust boundaries
Artifact and media chunks SHALL dispatch by an application-owned MIME/kind allowlist and SHALL reuse the shared markdown, lazy code/Mermaid, sanitized SVG, policy-gated A2UI, escaped JSON, and sandboxed HTML boundaries.

#### Scenario: HTML artifact renders
- **WHEN** an artifact declares an HTML MIME type
- **THEN** it MUST render in a sandboxed iframe without script or same-origin privileges
- **AND** the source MUST remain available through a safe fallback or download action

#### Scenario: SVG or Mermaid artifact renders
- **WHEN** an artifact contains SVG or Mermaid source
- **THEN** the existing sanitizer or strict lazy renderer MUST own DOM insertion
- **AND** unsanitized provider content MUST NOT be inserted into the page

#### Scenario: Image lacks useful alternative text
- **WHEN** a visible image chunk has no usable alt value
- **THEN** the renderer MUST show an explicit non-visual fallback instead of an unlabeled image

### Requirement: Chart artifacts use a closed application-owned model
The frontend SHALL retain exact Recharts 3.10.1 for chart artifacts and SHALL accept only a closed, validated application chart model containing supported chart kinds, labels, and finite numeric series.

#### Scenario: Valid chart artifact renders
- **WHEN** an artifact contains a valid supported chart model
- **THEN** the chart MUST resize with its container, expose Recharts accessibility support, and use application-selected design tokens
- **AND** provider data MUST be limited to labels and finite numeric values

#### Scenario: Malformed or unsupported chart artifact arrives
- **WHEN** chart JSON is invalid, exceeds the closed schema, or contains non-finite values
- **THEN** the renderer MUST fall back to escaped JSON/source content
- **AND** MUST NOT execute formatter functions or inject payload-controlled CSS or markup

### Requirement: A2UI testing is absent from production navigation
The standalone A2UI round-trip tester SHALL remain a development-only surface, while production SHALL retain live A2UI chat rendering, schema/service support, and runtime-console protocol state.

#### Scenario: Production navigation is built
- **WHEN** the frontend runs or builds in production mode
- **THEN** the navigation inventory, command palette, and route resolution MUST NOT expose the A2UI testing destination

#### Scenario: Development navigation is used
- **WHEN** the frontend runs in development mode
- **THEN** the live A2UI round-trip tester MAY remain reachable for developer verification

#### Scenario: Production A2UI event arrives
- **WHEN** a valid A2UI display or input event arrives in production
- **THEN** chat and runtime-console surfaces MUST still render or inspect it through the maintained policy-gated A2UI path

### Requirement: Wave 4 completion is evidence-gated
KBD change C-12 SHALL transition to complete only after focused catalog/security tests, frontend cheap gates, the full Wave 4 frontend test and production build, strict OpenSpec validation, and the configured quality review pass.

#### Scenario: Required evidence passes
- **WHEN** all C-12 implementation tasks and Wave 4 validation gates pass
- **THEN** canonical KBD state MUST record C-12 as complete before the OpenSpec change is archived

#### Scenario: Exhaustiveness or security regression fails
- **WHEN** a catalog variant is dropped, a production tester route remains, or an untrusted artifact bypasses its trust boundary
- **THEN** C-12 MUST remain non-complete and the change MUST NOT be archived
