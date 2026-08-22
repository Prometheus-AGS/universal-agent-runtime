# Decisions — `fix-graceful-shutdown-deadline-semantics`

## Iteration 1 — 2026-08-22T10:50:01Z

- **Decision:** converge after deterministic Persist and history replay.
- **Iteration:** 1 of 5.
- **Blocking constraints satisfied locally:** 5 of 5.
- **Rationale:** the implementation replaces an observed mandatory pre-drain
  wait with one absolute signal-to-exit window and closes only lifecycle owners
  shown to retain work. Focused process and container controls exercise the
  failure boundary directly.
- **Negative controls:** baseline process behavior failed 6 intended
  assertions; different-path C-12 failed at the intended 404 assertion; the
  previous immutable candidate recorded external SIGKILL and exit 137.
- **Independent review:** the history-free artifact critic and judge passed the
  corrected specification and plan. When implementation exposed retained
  Surreal lifecycle ownership, a separate history-free critic and judge passed
  the single-file scope expansion and lock-observation design.
- **Uncomfortable result:** nested SurrealKV teardown still warns that no
  runtime is available for closing the store. The receipt does not relabel the
  warning as clean. Pre-exit acquisition of the identical database path is the
  authoritative ownership proof.
- **Container correction:** the first manual control used a non-production port
  and therefore could not support a Docker-health claim. It was superseded by
  the port-1906 control that observed Docker `healthy` before the 30/35-second
  deadline test.
- **Pending boundary:** the full 10,800-second certification must restart after
  commit on the new immutable SHA. It is not part of the current PASS set.
- **Commit exclusions:** root `.refiner`, `.claude/settings.local.json`,
  unrelated KBD churn, static files, dependency manifests, and operator-owned
  changes remain outside this child.
