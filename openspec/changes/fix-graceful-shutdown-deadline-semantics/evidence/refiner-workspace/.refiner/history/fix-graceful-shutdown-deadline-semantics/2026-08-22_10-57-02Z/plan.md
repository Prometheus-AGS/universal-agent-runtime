# Plan — `fix-graceful-shutdown-deadline-semantics`

1. Preserve the failed immutable-candidate and baseline process behavior as
   negative evidence before changing runtime code.
2. Implement one crate-private shutdown coordinator and watchdog, start HTTP
   drain at signal observation, and make normal and deadline outcomes exclusive.
3. Give MCP transports and Surreal live-query supervisors explicit terminal
   ownership and wait for every owned resource on the normal path.
4. Prove the implementation with focused real-process, MCP, live-query,
   caller-cancellation, same-path persistence, and paired negative controls.
5. Build the exact source-only candidate into the existing non-root image and
   exercise a held SSE across the 30/35-second boundary.
6. Record Tier 0, strict OpenSpec, shell, dependency, visibility, scope, and
   evidence-schema results without claiming the pending parent soak.
7. Commit only the permitted child surface, close the child canonically, and
   restart parent certification from zero on the new immutable SHA.
