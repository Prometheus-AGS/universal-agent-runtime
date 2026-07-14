# 9. Vendor `@a2ui/web_core` and `@a2ui/react` and build a UAR-owned renderer

Date: 2026-07-13

## Status

Accepted

## Context

A2UI v1.0-rc is a Google-led candidate specification. The operator wants to adopt the standard while insulating UAR from upstream churn and retaining control over the component library.

## Decision

- Vendor `@a2ui/web_core` into `frontend/packages/a2ui-core/`.
- Vendor `@a2ui/react` as a reference implementation into `frontend/packages/a2ui-react/`.
- Build a UAR-owned React renderer in `frontend/packages/a2ui-uar/`.
- Preserve Apache-2.0 license headers from Google.
- Record the pinned version and update procedure in `frontend/packages/a2ui-core/UPSTREAM.md`.

## Consequences

- UAR can pin A2UI core to a specific version and upgrade on its own schedule.
- The reference renderer is available for cross-testing but is not the shipping renderer.
- The UAR renderer is the canonical location for React A2UI components.

## Alternatives considered

- Direct npm dependency on `@a2ui/web_core`: rejected because the candidate spec is still iterating and breaking changes are likely.
- Fork and maintain a separate package: rejected because vendoring keeps the dependency graph simpler and local patches easier to track.
