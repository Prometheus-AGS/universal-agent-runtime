## Why

A2UI Testing calls services/stores directly and does not yet prove a versioned, safe, shared React rendering path.

## What Changes

- Support A2UI v0.9.1 for GA; label v1.0 candidate experimental.
- Build one validated allowlisted React renderer shared by chat and the testing page.
- Certify updates, actions, progressive rendering, and rejection paths.

## Capabilities
### New Capabilities
- `a2ui-react-conformance`

## Impact
A2UI Rust endpoints/events, React renderer/page/store/service, protocol tests/docs.
