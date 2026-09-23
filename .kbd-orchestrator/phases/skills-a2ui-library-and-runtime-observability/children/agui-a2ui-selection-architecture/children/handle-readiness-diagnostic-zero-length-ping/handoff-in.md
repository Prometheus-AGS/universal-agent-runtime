# Handoff in — skills-a2ui-library-and-runtime-observability › agui-a2ui-selection-architecture › handle-readiness-diagnostic-zero-length-ping

**Spawned by:** skills-a2ui-library-and-runtime-observability › agui-a2ui-selection-architecture

## Why this child was spawned

The preceding classification child observed a newly received final, RSV-clear, server-unmasked Ping with opcode 9 and declared length zero. That header is valid under the observable RFC 6455 constraints, but the prior diagnostic collector accepts only a nonempty final Text frame and therefore cannot continue its sign-in wait. The discarded historical frame remains unknown.

## Inputs

- `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/classify-readiness-diagnostic-websocket-frame/reflection.md`
- `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/classify-readiness-diagnostic-websocket-frame/classification.md`
- `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/classify-readiness-diagnostic-websocket-frame/verification.md`
- `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/diagnose-database-readiness-latency/collector.mjs`

## Success criteria

- Assess and Plan define a diagnostic-only collector copy that recognizes only the exact observed zero-length Ping form, sends a zero-length Pong, and resumes the existing bounded sign-in wait.
- No network attempt occurs before an explicit Execute command.
- Execute, if authorized, permits at most one attempt and no retry, product mutation, service restart, generalized WebSocket implementation, or readiness/root-cause claim beyond the resulting evidence.
- Phase-end checks and fresh-context review distinguish current retained artifacts from unavailable collection-time provenance.

## Expected deliverables

- Child-local assessment, plan, collector copy, sanitized evidence, verification, independent-review receipt, reflection, and handoff-out.
- A narrow evidence-backed recommendation for the next database-readiness diagnostic boundary.
