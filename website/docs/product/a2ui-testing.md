---
sidebar_position: 3
title: A2UI Testing
description: Use the development-only A2UI schema, preview, and live-run trigger screen without confusing it with product certification.
source_records:
  - frontend/src/features/a2ui/ui/a2ui-testing-page.tsx
  - frontend/src/pages/admin-page.tsx
  - docs/product-surface-inventory.md
current_authority: /docs/product/a2ui-testing
---

# A2UI testing

## Boundary statement

**`/admin/a2ui-testing` exists only in a development build.** Production builds
do not register the page. Its local preview proves renderer behavior for the
entered data; only a trigger against an active run exercises the UAR artifact
path.

The screen loads registered artifact schemas, displays their IDs and types,
accepts candidate content/metadata, renders a shared-React preview, and can post
a test artifact to an active run through
`/api/uar/runs/{run_id}/a2ui/test-trigger`.

## Live development workflow

1. Start a local development build and a configured UAR server.
2. Start a chat turn so an active run exists.
3. Open `/admin/a2ui-testing`, select a schema and active run, and review the
   prefilled content.
4. Exercise the preview. Preview actions stay local and do not prove the live
   response endpoint.
5. Trigger the artifact, follow **Go to thread**, interact with the block, and
   observe the response return through the run.

No active run means there is no live target. Invalid object data may still be
displayable text but cannot establish a typed round trip. Record the UAR source,
development build, schema, artifact type, run ID boundary, command, and observed
result when retaining evidence.

## Profile limits

This developer page belongs to the `server-full` React source in development
mode. It is not a `minimal`, production-bundle, or embedded-mobile surface.
Renderer semantics in another host need separate evidence.

See [A2UI artifacts](/docs/product/a2ui) for the customer-facing behavior and
[Events, AG-UI, and A2UI](/docs/protocols/events-and-ui) for the protocol.

