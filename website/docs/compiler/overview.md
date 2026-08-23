---
sidebar_position: 1
title: Compiler Sessions
description: Inspect the experimental compiler-session control surface and its current limits.
source_records:
  - frontend/src/features/compiler/ui/compiler-page.tsx
  - frontend/src/features/compiler/api/compiler-api.ts
  - docs/product-surface-inventory.md
current_authority: /docs/compiler/overview
---

# Compiler sessions

## Boundary statement

**The compiler screen is experimental session management, not a certified
compile/package/deploy pipeline.** The current UI can list, refresh, and create
compiler sessions through `/api/compiler/sessions`.

Open `/admin/compiler` in the `server-full` operator application. The screen
shows each known session ID and status, exposes explicit loading and error
states, and labels the entire feature **Experimental**. Creating a session
records a new workflow boundary; it does not by itself prove that a skill was
compiled, packaged, signed, installed, or executed.

## Operator intent

Use this surface to inspect the current compiler-session API while the full
lifecycle remains under development. Evidence for any later stage must name the
actual produced component, source input, toolchain, capability contract,
installation path, and observed runtime execution.

The uncomfortable current limit is visible in the product: packaged output is
not GA-certified. Do not present the empty-state description or a session status
as proof that portable WASM output exists.

## Profile limits

The packaged compiler screen belongs to `server-full`. `minimal` and
`embedded-mobile` carry no compiler UI or compilation-availability claim from
this page.

Continue with [Tools](/docs/tools/overview) and
[A2UI artifacts](/docs/product/a2ui) for shipped execution and rendering
boundaries.

