---
target: Skills registry conventions for the Presentation extension
total_score: 23
max_score: 40
na_heuristics:
p0_count: 0
p1_count: 3
target_identity: "file:/Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/features/skills/ui/skills-page.tsx"
target_fingerprint: "sha256:53a9e95fb1040495194004d5e939fdb814163e09bf96e913bf373579b65190ea"
target_path: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/features/skills/ui/skills-page.tsx
timestamp: 2026-09-05T02-04-18Z
slug: frontend-src-features-skills-ui-skills-page-tsx
---
# Presentation preimplementation critique

Method: dual-agent — A: presentation_ui_critique_a; B: presentation_ui_critique_b. Source/design-contract review only; no visual acceptance. A completed before B's findings entered the parent synthesis. Both read the actual incumbent Skills registry independently; no generation history was supplied.

## Design specificity

The Presentation design fits the existing console. It separates reusable templates, safe preview, assignment and actual run output. The incumbent Skills page detector returned `[]`, exit 0, with zero rules/locations or false positives. Source review still identified patterns the new editor must not copy. No unrelated Skills refactor is authorized.

## Usability scores

| Heuristic | Incumbent source | Presentation contract |
| --- | --- | --- |
| System status | 3 | 3 |
| Familiar language | 3 | 3 |
| Control and freedom | 2 | 3 |
| Consistency | 3 | 3 |
| Error prevention | 2 | 3 |
| Recognition | 2 | 3 |
| Efficiency | 1 | 2 |
| Minimalism | 3 | 3 |
| Error recovery | 2 | 3 |
| Help | 2 | 2 |
| Total | 23/40 | 28/40 |

Incumbent source: Acceptable. Proposed contract: Good. Different artifacts, not measured improvement. Visual accessibility, responsiveness and performance are unverified; the technical audit withheld a visual health score rather than inferring it from source.

## Priority issues and dispositions

- P1: Preserve dirty drafts and prevent an older save from clearing newer edits. Incumbent Skills resets on close and save completion. New contract locks editable fields during a save, retains draft text on failure, confirms discard and ignores stale completion. Command: harden.
- P1: Graph-owned drafts and independent field subscriptions. Do not copy page-local entity/form state or whole-registry rerenders from the incumbent. New platform domain owns state/actions. Command: optimize.
- P1: In-editor save errors and recovery. Skills' page ErrorBar is outside its dialog; exposure was not browser-tested. New editor requires focusable summary, associated fields and preserved source. Commands: audit/harden.
- P2: Narrow-screen wrapping and touch-sized controls. Do not inherit h-7 row actions or a nonwrapping multi-action footprint. New rows have one opening target and editor/preview stack. Command: adapt.
- P2: Supported-component discovery. Provide collapsible reference to the actual nine component kinds, child relationships and a data-binding example. This does not expand into a visual builder. Command: clarify.

## Strengths, cognitive load and personas

Useful strengths: explicit preview-only behavior, revision-aware recovery and separation of authoring from assignment. The incumbent has moderate source-level cognitive load (chunking/progressive-disclosure gaps); the new contract keeps four authoring fields and separates Save/Cancel from preview. Actual visual hierarchy remains unverified. Cognitive burden is concentrated in JSON authoring; concise collapsible guidance is warranted, not a wizard.

Alex (power user) needs component discovery without a forced tutorial. Sam (keyboard/screen-reader user) needs focus restoration, associated errors and announced state in the active editor. Jordan (new author) needs the empty state to explain “reusable UI template.” Invalid JSON and revision conflicts are the emotional valleys; preserved source and clear recovery provide reassurance. A saved revision must not imply assignment or rendering.

Minor observations: distinguish an empty library from no search matches; retain incumbent control styling without introducing new decorative borders. The uncomfortable limitation remains technical JSON authoring, not a no-code tool.

Questions skipped: the operator explicitly authorized autonomous implementation within the already settled scope. Findings are incorporated in ui-plan.md; no new product decision is required. The next action is a fresh adversarial design review before UI code, then phase-end bounded polish and actual visual evidence.
