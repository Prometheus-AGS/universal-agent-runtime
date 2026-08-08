## 1. Workflow and dependency decisions

- [x] 1.1 Record C-12 as canonical KBD in-progress and scaffold the `chunk-catalog-renderers` OpenSpec change
- [x] 1.2 Resolve `cand-012` against the installed dependency and current primary documentation, recording the Recharts 3.10.1 keep decision and trust-boundary constraints
- [x] 1.3 Complete the repository UI/UX routing consult or document each unavailable-skill fallback before editing presentation code

## 2. Portable protocol and migration boundary

- [x] 2.1 Add the exact shared camelCase `ContentBlock` union and `assertNever` helper under `frontend/src/shared/content/`
- [x] 2.2 Add fixtures covering all portable variants, including image metadata and a divider spacer
- [x] 2.3 Implement the historical partial-content decoder at the PGlite read boundary and cover every prior discriminant
- [x] 2.4 Update new persisted-message writes to use the canonical portable shape without changing backend or provider contracts

## 3. Complete Chunk model and projections

- [x] 3.1 Define the complete §8 `ChunkKind` and discriminated `Chunk` union in `features/chat/model/chunk.ts`
- [x] 3.2 Implement one deterministic `toChunks` projection for portable blocks, including tool-use/result joining and stable ids
- [x] 3.3 Add exhaustive typed phase, bubble-visibility, renderer-name, and trace-disposition maps
- [x] 3.4 Extend the validated AG-UI normalizer to project known official/custom events into the shared chunk union
- [x] 3.5 Preserve unknown CUSTOM and RAW inputs as hidden-by-default `raw` chunks with durable identity and payload
- [x] 3.6 Add unit and compile-time conformance tests for every portable block and every chunk kind

## 4. Chunk renderer catalog

- [x] 4.1 Add shared Flat 2.0 chunk surface/disclosure primitives with accessible state labels and no decorative borders or shadows
- [x] 4.2 Implement text, markdown, reasoning, thinking, divider, citation, and RAG citation renderers using the existing markdown/source components
- [x] 4.3 Implement tool-call, tool-approval, tool-denied, skill-activation, context-update, and memory renderers with explicit state text
- [x] 4.4 Implement A2UI display/input renderers through the maintained policy-gated A2UI component
- [x] 4.5 Implement artifact MIME dispatch for markdown, code, Mermaid, sanitized SVG, sandboxed HTML, JSON, charts, and download fallback
- [x] 4.6 Implement image, video, and file renderers with accessible labels, reserved dimensions, and safe fallback behavior
- [x] 4.7 Implement usage and error surfaces and intentional no-bubble renderers for state, step, and raw chunks

## 5. Chart and Assistant UI integration

- [x] 5.1 Add a closed chart schema accepting only supported kinds, labels, and finite numeric series
- [x] 5.2 Render valid chart models with responsive and accessibility-enabled Recharts using application-owned tokens
- [x] 5.3 Prove malformed chart payloads fall back to escaped source and never reach CSS/markup injection configuration
- [x] 5.4 Add stable `RichDataRenderers` registrations for all named rich chunk families
- [x] 5.5 Update the external chat runtime projection to emit Assistant UI data parts while preserving native text/reasoning streaming
- [x] 5.6 Remove rich pseudo-tool rendering branches after equivalent data-part coverage is wired

## 6. Production surface consolidation

- [x] 6.1 Derive the A2UI testing destination from a development-only navigation inventory
- [x] 6.2 Prevent production route resolution and command-palette discovery for the A2UI testing page
- [x] 6.3 Preserve live A2UI chat rendering, schemas/store/service, and runtime-console protocol state
- [x] 6.4 Update navigation tests for development and production modes and verify no dangling production destination remains

## 7. Catalog evidence and UI review

- [x] 7.1 Add a catalog story containing every bubble-visible chunk and explicit trace-only dispositions
- [x] 7.2 Add focused renderer tests for divider semantics, collapsed details, textual status, media fallback, and Assistant UI registration
- [x] 7.3 Add security fixtures for sandboxed HTML, sanitized SVG/Mermaid, escaped JSON, A2UI policy routing, and chart payload rejection
- [x] 7.4 Run the manual/available UI audit, critique, accessibility, Flat 2.0, responsive, and reduced-motion checks and record the findings
- [x] 7.5 Run an isolated adversarial review and resolve every critical finding or document evidence-backed nonblocking dispositions

## 8. Verification and closeout

- [x] 8.1 Run frontend typecheck, lint, architecture boundaries, Flat 2.0 gate, and focused C-12 tests
- [x] 8.2 Run the Wave 4 full frontend test suite and production build, including frozen-lockfile and emitted-asset checks
- [x] 8.3 Run strict OpenSpec validation, artifact refinement, schema/state consistency, and scoped diff-integrity checks
- [x] 8.4 Write verification evidence, mark every completed task, and sync the `frontend-content-rendering` capability
- [x] 8.5 Transition canonical KBD C-12 to complete, append the `.prometheus` waypoint, archive this change, and retire the superseded July proposal with an explicit pointer
