# Decisions

## D-001 · Presentations are reusable UI templates [Spec · 2026-09-04]

TL;DR: The operator selected reusable UI templates, separate from the development-only A2UI tester, in response to the Presentation domain question.

Why: Stable template identity permits persistence, assignment, safe preview and per-run provenance. A renderer catalog or development test session is not the assignable business entity.

Alternatives: Treating design systems/renderers as Presentations would select renderer infrastructure instead of content. Promoting the existing A2UI tester would reverse its intentional development-only boundary. Neither is the selected domain.

Compatibility: Keep legacy chat rendering when clients omit negotiation. New negotiated requests may narrow rendering; they cannot widen host policy. Template preview is local and never executes declared actions.

Scope cut: This phase does not introduce a visual drag-and-drop builder, remote template marketplace, arbitrary JavaScript components or public cross-owner sharing. Use the existing validated declarative A2UI profile.
