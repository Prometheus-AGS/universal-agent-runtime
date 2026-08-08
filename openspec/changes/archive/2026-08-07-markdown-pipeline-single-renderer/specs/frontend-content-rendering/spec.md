## ADDED Requirements

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
