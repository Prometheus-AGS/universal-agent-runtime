## ADDED Requirements

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
