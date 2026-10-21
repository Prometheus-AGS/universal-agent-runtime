# Process entrypoint lifecycle handoff

Date: 2026-10-20. Active phase: `function-first-release-certification`.
Follow-up task: `deterministic-lifecycle-management` 5.2. Verification: **CLOSED**.

Source inspection after the first inventory found that standalone command paths
called `process::exit`, leaving a live configuration watcher outside awaited
cleanup. Telemetry shutdown errors were discarded. The Windows service published
Running before server readiness and could return on a status-reporting error
without awaiting server cleanup. The server installed OS signal observation only
after initialization reached its serving loop. These are source-grounded gaps in
WP5; none is asserted as the historical c9 lock-holder identity.

Changed production files:

- `src/main.rs`: returns `ExitCode` after its runtime settles; command outcomes
  and panics converge on awaited configuration-watcher closure and telemetry
  shutdown. Failure or panic in one finalizer does not skip the other. Existing
  command exit statuses remain the result unless cleanup turns success into
  failure. Configuration-load failure still finalizes initialized telemetry.
  One-shot maintenance/evaluation/compile commands load an immutable configuration
  without starting a filesystem watcher. The long-running server retains the
  watcher under its explicit shutdown authority.
- `src/server/windows_service.rs`: runtime/configuration/server work has a
  retained cleanup boundary. Running follows the common server's actual readiness
  acknowledgement. StopPending reporting errors are retained while the server is
  still awaited. Configuration and telemetry cleanup errors contribute to the
  final service result. Stopped follows cleanup, and its own reporting failure is
  aggregated. Wait hints no longer claim an estimated thirty-second completion.
- `src/bin/uar-sidecar.rs`: metrics initialization is inside its existing caught
  owner boundary, so that failure cannot skip the inherited-input join or
  telemetry finalizer.
- `src/server/process_signals.rs`: new owned signal subscription resource.
  Subscription creation occurs before database initialization. Unix handles
  SIGINT/SIGTERM; Windows console hosts install Ctrl-C. Windows hosts supplying
  their own cancellation token retain SCM/inherited-input authority. The task
  observes actual signal or scope cancellation and is joined through its scope.
- `src/server.rs`: installs that resource before server initialization and uses
  the retained process cancellation event during serving, removing the late,
  separate signal waiter.
- `src/server/lifecycle_graph.rs`: configuration depends on the signal owner so
  signal observation remains available through server resource finalization.

Supporting edits update the OpenSpec task/delta, resource inventory, renewed
source inventory and canonical KBD task 5.2. The earlier task 5.1 is immutable:
the canonical runtime rejected reopening it, so the follow-up is registered
explicitly instead of rewriting history or manually editing generated state.
That rejected metadata command executed no verification.

Authored acceptance scenarios, not executed:

1. Hold startup before database initialization, deliver an actual OS shutdown
   signal, and release the held initialization. Observe cancellation requested,
   no READY publication, registered-resource cleanup and signal-task join.
2. Run credential maintenance, compile and evaluation commands through their
   actual entrypoint. Observe their original outcomes only after the configuration
   watcher is joined. Inject watcher failure and retain a failed exit outcome.
3. Inject a command panic after configuration acquisition. Require configuration
   and telemetry finalization before process exit; preserve the original failure.
4. Fail configuration loading after telemetry initializes. Require telemetry's
   shutdown result to settle before the entrypoint returns.
5. Hold Windows startup readiness. Require StartPending, then Running only after
   the real ready receipt. Fail StopPending publication while resource cleanup is
   held; require cleanup acknowledgement before Stopped and a failed final result.
6. Fail or panic configuration finalization and fail telemetry shutdown. Preserve
   both outcomes; never publish a successful exit by discarding cleanup errors.

No tests, Cargo, compiler probes, formatters, lint, builds, packaging validation,
Windows execution, signal injection or independent review ran. Only source
inspection, source/document authoring and supported metadata commands were used.
The pinned Tokio 1.53.1 signal API declarations were read in the local registry;
registry source was not changed. Telemetry uses its existing SDK shutdown result;
this work does not claim stronger upstream physical-release guarantees. Abrupt
process termination and panic-abort cannot provide graceful completion receipts.
