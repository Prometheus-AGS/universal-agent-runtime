# frontend-content-rendering Specification

## Purpose
TBD - created by archiving change markdown-pipeline-single-renderer. Update Purpose after archive.
## Requirements
### Requirement: Single shared markdown renderer
The frontend SHALL expose one shared `MarkdownBubble` renderer and SHALL use its shared plugin and component configuration for both assistant-ui message parts and explicit markdown sources.

#### Scenario: Assistant message renders through the shared component
- **WHEN** assistant or user message text is rendered inside the assistant-ui thread
- **THEN** the message part MUST use `MarkdownBubble` with deferred assistant-ui parsing
- **AND** it MUST NOT instantiate a separate markdown plugin chain

#### Scenario: Skills preview renders through the shared component
- **WHEN** a skill description or prompt overlay is previewed
- **THEN** the preview MUST pass its source to `MarkdownBubble`
- **AND** the Skills page MUST NOT directly instantiate `ReactMarkdown`

### Requirement: Complete ordered markdown pipeline
The shared renderer SHALL use GFM, hard-break, and math remark plugins and SHALL use raw parsing, sanitization, and KaTeX rehype plugins in the security-preserving order.

#### Scenario: Remark features render consistently
- **WHEN** markdown contains a GFM table, a single newline, and inline or display math
- **THEN** the renderer MUST produce the corresponding table, line break, and KaTeX content

#### Scenario: Raw HTML is followed immediately by sanitization
- **WHEN** the rehype plugin chain is configured
- **THEN** `rehype-raw` MUST be immediately followed by `rehype-sanitize` with the shared schema
- **AND** `rehype-katex` MUST run after sanitization

### Requirement: Untrusted markdown HTML is sanitized
The shared renderer SHALL treat markdown and embedded HTML from users, providers, tools, and A2UI payloads as untrusted input and SHALL permit only the explicit schema vocabulary, attributes, and protocols.

#### Scenario: Executable elements are removed
- **WHEN** markdown contains `script`, `iframe`, `object`, or equivalent executable HTML
- **THEN** the rendered DOM MUST NOT contain those elements
- **AND** their executable content MUST NOT run

#### Scenario: Handler and presentation attributes are removed
- **WHEN** raw HTML contains event-handler attributes or an unapproved `style` attribute
- **THEN** those attributes MUST be absent from the rendered DOM

#### Scenario: Unsafe URL protocol is removed
- **WHEN** a link or media source uses a `javascript:` or another unapproved protocol
- **THEN** the unsafe URL attribute MUST be absent from the rendered DOM

#### Scenario: Approved limited HTML survives
- **WHEN** markdown contains schema-approved semantic HTML or limited SVG with approved attributes
- **THEN** the approved nodes and attributes MUST remain renderable
- **AND** all non-approved attributes MUST be removed

### Requirement: Math rendering preserves the sanitizer boundary
The frontend SHALL allow only KaTeX input marker classes through the untrusted-input sanitizer and SHALL let the trusted KaTeX transform generate its output after sanitization.

#### Scenario: KaTeX input is accepted
- **WHEN** markdown contains inline or display math
- **THEN** its sanitized intermediate nodes MUST retain the required math marker classes
- **AND** KaTeX MUST produce accessible rendered math

#### Scenario: Arbitrary classes are rejected
- **WHEN** raw HTML supplies class names outside the sanitizer allowlist
- **THEN** those class names MUST be removed before trusted transforms run

### Requirement: Provider and realtime contracts remain unchanged
The markdown migration SHALL remain presentation-only and SHALL preserve provider-neutral text inputs, assistant streaming behavior, and persisted message/realtime state.

#### Scenario: Provider text uses the same render boundary
- **WHEN** text from any configured provider reaches a user, assistant, or preview surface migrated by this change
- **THEN** the renderer MUST consume the existing string/message-part contract without provider-specific branches

#### Scenario: Streaming state is not mutated by rendering
- **WHEN** a message streams through assistant-ui
- **THEN** deferred rendering MUST project the current message text without writing to stores, services, or persistence

### Requirement: Completion is evidence-gated
KBD change C-08 SHALL transition to complete only after focused security/renderer tests, frontend cheap gates, strict OpenSpec validation, and the configured per-change quality review pass.

#### Scenario: Required evidence passes
- **WHEN** all C-08 implementation tasks and required validation gates pass
- **THEN** canonical KBD state MUST record C-08 as complete before the OpenSpec change is archived

#### Scenario: Security regression blocks completion
- **WHEN** an XSS fixture renders an executable element, unsafe attribute, or unsafe protocol
- **THEN** C-08 MUST remain non-complete and the change MUST NOT be archived

### Requirement: Rich markdown blocks render only after finalization
The shared markdown renderer SHALL keep fenced code and Mermaid content as escaped source while an assistant message is running and SHALL activate lazy rich-block rendering only after the message is finalized. Explicit read-only `source` rendering SHALL be treated as finalized.

#### Scenario: Closed fence remains source during streaming
- **WHEN** a running assistant message contains a syntactically closed code or Mermaid fence
- **THEN** the block MUST render as escaped source
- **AND** Mermaid and Shiki MUST NOT be loaded for that block

#### Scenario: Finalized fence activates its renderer
- **WHEN** the owning message transitions from running to complete
- **THEN** a Mermaid fence MUST activate the Mermaid block
- **AND** every other fenced language MUST activate the Shiki code block

#### Scenario: Explicit preview is finalized
- **WHEN** `MarkdownBubble` receives an explicit `source` outside assistant message context
- **THEN** its completed fences MUST be eligible for lazy rich-block rendering

### Requirement: Mermaid and Shiki stay outside the initial graph
The frontend SHALL load Mermaid and Shiki only through dynamic imports and SHALL emit their library modules into auditable `vendor-mermaid` and `vendor-shiki` chunks that are not statically reachable from the initial application entry.

#### Scenario: Initial entry excludes rich-block engines
- **WHEN** the production frontend graph is built
- **THEN** the initial entry MUST NOT statically import Mermaid or Shiki
- **AND** the build MUST emit separate `vendor-mermaid` and `vendor-shiki` chunks

#### Scenario: Plain markdown does not request engine modules
- **WHEN** a finalized markdown bubble contains no fenced block
- **THEN** neither Mermaid nor Shiki MUST be requested by the renderer

### Requirement: Mermaid rendering is strict and accessible
The Mermaid block SHALL render only finalized diagram syntax, SHALL initialize Mermaid with `startOnLoad: false` and `securityLevel: "strict"`, SHALL sanitize the renderer-produced SVG before DOM insertion, and SHALL retain a readable text alternative.

#### Scenario: Strict configuration owns untrusted diagram syntax
- **WHEN** a finalized Mermaid fence is rendered
- **THEN** Mermaid MUST receive `securityLevel: "strict"`
- **AND** caller-controlled directives MUST NOT relax the configured security policy

#### Scenario: Diagram succeeds
- **WHEN** Mermaid returns a valid SVG
- **THEN** the sanitized diagram MUST be displayed with an accessible name
- **AND** the original diagram source MUST remain available as a text disclosure

#### Scenario: Diagram parse or sanitization fails
- **WHEN** Mermaid rejects the diagram or sanitization produces no SVG
- **THEN** the block MUST display a concise failure status and escaped diagram source
- **AND** sibling markdown content MUST remain rendered

### Requirement: Shiki highlighting preserves source safety and theme
The code block SHALL tokenize finalized code through a lazily loaded, cached Shiki highlighter, SHALL render token content as React text nodes for the resolved light or dark theme, and SHALL preserve escaped source for loading, unsupported languages, and failures.

#### Scenario: Supported language highlights after loading
- **WHEN** a finalized supported code fence loads Shiki successfully
- **THEN** its tokens MUST render in the resolved application theme
- **AND** untrusted code MUST NOT be inserted as highlighter-produced HTML

#### Scenario: Theme changes
- **WHEN** the resolved application theme changes while a highlighted block is mounted
- **THEN** the block MUST refresh token colors for the new theme without changing its source

#### Scenario: Unsupported language degrades to source
- **WHEN** Shiki cannot resolve the fence language
- **THEN** the block MUST remain readable as escaped source with its language label

### Requirement: Asynchronous block failures are isolated
Every lazy Mermaid and Shiki block SHALL have its own Suspense fallback and React error boundary using the escaped source contract.

#### Scenario: One block renderer crashes
- **WHEN** one lazy block throws during module load or React rendering
- **THEN** only that block MUST fall back to escaped source
- **AND** other blocks and prose in the same bubble MUST remain available

#### Scenario: Renderer is loading
- **WHEN** a lazy engine module has not resolved
- **THEN** the block MUST show its escaped source rather than a blank region or spinner-only placeholder

### Requirement: Math assets load once through the shared renderer
The frontend SHALL import KaTeX CSS once at the shared markdown entry and SHALL let Vite emit the package CSS font URLs as local fingerprinted assets. Lazy code and Mermaid blocks MUST NOT load separate KaTeX styles or remote math fonts.

#### Scenario: Markdown entry owns KaTeX assets
- **WHEN** a page renders one or more math expressions
- **THEN** all expressions MUST use the shared KaTeX stylesheet import
- **AND** no block-level or remote KaTeX font request MUST be introduced

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
