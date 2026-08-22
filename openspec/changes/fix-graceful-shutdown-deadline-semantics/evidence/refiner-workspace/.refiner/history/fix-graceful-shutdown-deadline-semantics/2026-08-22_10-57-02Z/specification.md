# Specification — `fix-graceful-shutdown-deadline-semantics`

The server-full candidate must interpret the configured shutdown timeout as an
absolute graceful window measured from OS-signal observation. It must stop
accepting HTTP work immediately, complete every owned cleanup before reporting
the graceful outcome, and exit 0 within one second after the deadline when
held work prevents completion. Forced exit must not depend on Tokio or the
ordinary stderr lock. The non-root container must exit at its 30-second
internal limit before Docker's 35-second escalation limit without SIGKILL.

Verification must retain the observed baseline failures and the different-path
persistence failure, report only server-full, introduce no public API or
dependency, and state that the parent 10,800-second certification remains
pending until it restarts on the committed immutable candidate.
