## Why

React is the implemented first-party UI, but public docs still describe HTMX/Web Components and live React code bypasses mandatory layer boundaries.

## What Changes

- Declare React 19 + TypeScript as the primary UI in an ADR and frontend architecture document.
- Inventory live routes, actions, APIs, specs, maturity, and tests.
- Add a blocking no-new-violations boundary checker with a shrinking legacy allowlist.

## Capabilities

### New Capabilities
- `react-product-contract`

## Impact

Architecture docs, CI scripts, frontend import policy; no behavior changes.
