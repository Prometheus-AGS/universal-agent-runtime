# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Product Purpose

Universal Agent Runtime provides governed agent execution. This frontend is its existing operator workspace and chat interface.

## Capabilities and Constraints

The operator confirmed that an assignable Presentation represents a reusable UI template, separate from the development-only A2UI tester. Presentation management must persist records and support assignment to governed runs. Existing clients must retain their current rendering behavior unless they explicitly negotiate a different mode.

Business entities and editor drafts live in the normalized entity graph and reach components through typed platform domain hooks. Agent kernels do not perform writes; the trusted host owns mutations. A template is declarative UI, not executable code.

## Operating Context

This work extends the existing admin console. It does not redesign the application or promote development tooling into production. The user prioritizes implementation, with tests at phase boundaries.

## Evidence on Hand

Existing admin navigation: `src/app/shell/nav-destinations.ts`. Existing live A2UI rendering: `src/features/a2ui/a2ui-surface-renderer.tsx`. The Presentation domain decision is recorded in the active KBD phase's decision log. No new commercial claims or synthetic usage statistics are authorized.
