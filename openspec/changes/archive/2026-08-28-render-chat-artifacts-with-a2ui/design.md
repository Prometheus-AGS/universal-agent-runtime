## Context

A2UI v0.9.1 is the current production protocol. A2UI v1.0 is a candidate, so
this change keeps the production wire contract while using the latest installed
upstream processor (`@a2ui/web_core` 0.10.6 through
`@prometheus-ags/a2ui-core`). The repository already owns a certified UAR
catalog and React renderer in `frontend/packages/a2ui-uar`.

## Decisions

### The runtime emits a real surface

The effective policy artifact becomes a three-message A2UI sequence: create the
surface, publish its approved UAR components, then publish the bound data model.
The default presentation summarizes the model route, resource modes and counts,
runtime controls, and warnings. Exhaustive capability identifiers remain in the
run policy record and are not dumped into the primary chat surface.

### The chat client uses the canonical processor and renderer

The display block parses a JSON array, a wrapped message list, a single message,
or JSONL; validates the production version and UAR profile envelope; removes
only the recognized transport-only `profile` field; and hands the messages to
`MessageProcessor`. Every resulting surface is rendered by `UarSurface` with the
active application theme.

### Failure is explicit and inspectable

Parsing, version, catalog, and component failures do not fall back to arbitrary
HTML or a plausible-looking text surface. The artifact reports an invalid
surface and offers the original source in a bounded, wrapping disclosure.

## UI/UX routing distillation

The implementation keeps the incumbent UAR artifact shell and semantic tokens.
Impeccable critique identified the unstructured JSON wall, false success state,
and missing progressive disclosure as the blocking defects. Frontend Design and
UI/UX Pro Max guidance favor a concise operator-oriented hierarchy, bounded long
content, clear state, and stable responsive composition. Vercel React guidance
keeps message processing as memoized derived state with stable surface keys and
no effect-driven duplication. No entity graph state is introduced because the
surface is a transient projection of an immutable artifact payload. The UAR
catalog remains the security boundary; no remote code, HTML, or unknown
component is executed.

## Uncomfortable constraint

A surface generated from an old or malformed artifact may no longer appear to
"work" by displaying its bytes as ordinary text. That apparent compatibility
was the defect: unsupported content must be labeled as source, not represented
as a successfully rendered A2UI surface.
