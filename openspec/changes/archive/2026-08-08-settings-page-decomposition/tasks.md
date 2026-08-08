## 1. Workflow and baseline

- [x] 1.1 Record canonical C-14b in progress, create the OpenSpec artifacts, run mandated UI/UX routing, and validate the `frontend-configuration-surfaces` delta before implementation
- [x] 1.2 Inventory the 3,336-line source, navigation and panel registry, existing public export, focused-test gap, bundle constraint, and protected paths
- [x] 1.3 Capture a deterministic source-structure baseline and exact current navigation/panel keys before extraction

## 2. Shared settings UI boundaries

- [x] 2.1 Extract the typed settings navigation inventory without changing category order, item keys, labels, subtitles, or icons
- [x] 2.2 Extract shared field, toggle, select, header, loading/error/saved, and advanced-section primitives without changing JSX or classes
- [x] 2.3 Extract namespace and schema-driven panel rendering, retaining metadata, conflict, dirty/default, sensitive, textarea, JSON, numeric, enum, array, and generic fallback behavior
- [x] 2.4 Extract pure resilience preview helpers and pass typecheck, lint, architecture, Flat 2.0, and token gates for the shared boundary

## 3. Domain panel decomposition

- [x] 3.1 Extract AI/LLM provider, vision, context-management, RAG, and knowledge-base panels without behavior changes
- [x] 3.2 Extract file-processing, Unstructured, Mistral OCR, and Kreuzberg panels without behavior changes
- [x] 3.3 Extract the resilience panel with all validation, recommended defaults, status parsing, saved/error, and advanced retry behavior intact
- [x] 3.4 Extract intent-classifier, governance, agent-configuration, and skill-configuration panels without behavior changes
- [x] 3.5 Extract memory, prompt-caching, and JWT-gated user-settings panels without behavior changes

## 4. Registry and route composition

- [x] 4.1 Extract the internal panel registry with the same custom renderers, generic namespace fallbacks, keys, titles, and subtitles
- [x] 4.2 Reduce `settings-page.tsx` to responsive navigation, metadata availability, active-panel selection, and content composition while retaining the feature root export
- [x] 4.3 Add deterministic module-size validation and prove no settings page or panel module exceeds approximately 600 lines
- [x] 4.4 Add focused React tests for default composition, custom-panel navigation, unavailable items, and generic fallback behavior

## 5. Verification, review, and closeout

- [x] 5.1 Run typecheck, lint, architecture-boundary, Flat 2.0, token, module-size, and focused settings tests after the decomposed call graph is wired
- [x] 5.2 Run the full frontend suite, production manifest build, and bundle budget at the C-14b completion boundary
- [x] 5.3 Run strict OpenSpec change/capability validation, manual UI audit/critique/polish, responsive smoke checks, and scoped diff integrity
- [x] 5.4 Write `files.txt`, retained verification evidence, and explicit C-14c handoff constraints; confirm protected/staged paths remain untouched
- [x] 5.5 Run a fresh isolated artifact-only adversarial review and resolve every critical finding
- [x] 5.6 Transition canonical KBD C-14b to complete, append the `.prometheus` waypoint, sync and archive the OpenSpec change, emit the step-18 completion signal, and advance automatically to C-14c
