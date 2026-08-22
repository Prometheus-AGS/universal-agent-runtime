# Baseline non-root shutdown failure

Profile: `server-full`
Immutable source SHA: `32afa53d510c8b840b3e98b2be9d9f5dee149531`
Candidate tag: `operational-resilience-32afa53d510c`

## Command

```bash
scripts/certify-operational-resilience-local.sh certify
```

## Observed result

- Command exit: `1`
- Suite outcome: `failed`
- Started: `2026-08-21T18:14:11Z`
- Finished: `2026-08-21T21:37:10Z`
- Docker stop result observed by the invoking terminal: container exit code `137`; the script rejected it at its unchanged `[[ "$container_exit_code" == 0 ]]` assertion.
- `non-root-container.json` is absent because the assertion stopped the script before that success artifact was written.

Durable source-SHA binding:

```text
target/resilience-certification/results.json:
{"source_sha":"32afa53d510c8b840b3e98b2be9d9f5dee149531","outcome":"failed","exit_code":1}

target/resilience-certification/candidate-build.json:
{"source_sha":"32afa53d510c8b840b3e98b2be9d9f5dee149531","candidate_tag":"operational-resilience-32afa53d510c","configured_soak_duration_seconds":10800}
```

The three-hour soak itself completed before the container boundary was reached: 10,196 requests, zero errors, zero duplicate events, p95 13 ms, and peak RSS growth 5,376 KiB.

## Mandatory-wait evidence

The candidate container log records:

```text
2026-08-21T21:36:37.181843Z Shutdown signal received ... timeout_secs=30
2026-08-21T21:37:07.189286Z Server shut down gracefully
```

Elapsed signal-to-Axum-completion: 30.007443 seconds. Source at the immutable SHA cancelled the HTTP token only after ingestion cleanup, and each Axum graceful-shutdown future then slept the full configured timeout before returning. Docker's external stop deadline was also 30 seconds, so the orchestrator escalated at the same boundary and produced exit 137.

Evidence directory:

```text
/Users/gqadonis/.claude/worktrees/uar-operational-resilience-32afa53d-rerun/target/resilience-certification
```

This failed candidate remains immutable and is not reused as proof for the corrected implementation.
