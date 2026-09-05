## Context

The approved Presentation domain is a reusable UI template. Existing artifact schemas and development-only A2UI testing do not provide owner-scoped production template management.

## Goals / Non-Goals

Introduce owner-scoped revisioned Presentation records, safe template validation, persistence and authenticated CRUD, plus a production graph-backed registry/editor/preview. Preserve the development-only tester.

No arbitrary executable templates, no visual drag-and-drop builder, no public sharing or marketplace, and no production promotion of development testing tools.

## Decisions

Use the existing validated A2UI profile and trusted-host mutation boundary. Preserve legacy clients. Persistent records and drafts reach UI only through typed normalized graph domain hooks. Follow the ordered phase plan in .kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/plan.md.

## Risks / Trade-offs

Revision conflicts must be visible and must not silently overwrite another editor's changes. A JSON editor plus safe preview is initially less approachable than a no-code builder; actionable validation and accessible controls are required. Current ledger completion is not evidence of these observed missing contracts.
