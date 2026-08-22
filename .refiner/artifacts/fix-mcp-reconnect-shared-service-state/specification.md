# Specification — `fix-mcp-reconnect-shared-service-state`

## Clarified intent

Produce a deterministic, reviewable verification artifact proving that an MCP
transport replacement survives UAR registry-view boundaries, failed operations
are not replayed, authorization remains narrow, and the local installed
candidate evidence belongs to the exact committed source.

## Target state

- Later independent filtered requests and pre-existing merged views use the
  replacement service slot.
- An old filtered view uses configuration B after an A→B upsert and cannot
  restore configuration A during a later reconnect.
- A crash and a timeout each appear exactly once as failed normalized
  tool-result events and exactly once in the fixture process trace.
- The process sequence is exactly `echo, crash, echo, timeout, echo`, with a new
  PID after crash and another new PID after timeout.
- Excluded server/tool views remain excluded.
- Raw installed evidence is persisted with replayable hashes and exact source
  identity.
- Reporting stays local and `server-full`-scoped; it does not substitute for
  the parent three-hour soak or any GA/deployment decision.

## Measurable outcomes

- Focused registry tests: 2 passed, 0 failed.
- Focused operational suite: 5 passed, 0 failed.
- Installed boundary: crash failures 1, timeout failures 1, crash calls 1,
  timeout calls 1, reconnects 2.
- Timeout: at least 30 seconds and below the certifier's upper bound.
- Strict OpenSpec validation: child and parent exit 0.
- GitHub Actions deployment-only validator: exit 0.

## Unknowns and limits

- The child preflight lasts 60 seconds; sustained three-hour behavior remains a
  parent-phase requirement.
- Three pre-existing Rust warnings remain outside the child edit.
- Local macOS arm64 and Linux arm64 container results transfer to no other
  runtime profile or platform.

## Execution decision

Deterministic execution is required for raw SSE parsing, process-sequence
validation, source and evidence hashing, strict OpenSpec validation, artifact
schema checks, and scoped-diff checks.
