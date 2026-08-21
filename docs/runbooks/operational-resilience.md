# Operational resilience certification

Run `scripts/certify-operational-resilience.sh`. The deterministic suite writes
`target/resilience-certification/results.json` and `test.log`; CI retains both.

## Published limits

- 100 parallel simulated runs, zero errors, p95 under 250 ms.
- 20 parallel installed-runtime requests, zero failures.
- Installed streaming soak: zero errors, zero duplicate events, p95 at or below
  2,000 ms, and peak RSS growth at or below 262,144 KiB.
- Lifecycle and tool waits are bounded; the certifying job has a 300-minute ceiling.
- Provider/MCP failures must reach an explicit error or recover on a later bounded attempt.

The pull-request lane uses a 60-second deployment-validation preflight. It is
not multi-hour certification evidence. Manual `workflow_dispatch` and scheduled
runs use at least 10,800 seconds; the manual `soak_duration_seconds` input
defaults to 10,800 and rejects a smaller value. The retained `soak.json` must
show both configured and observed duration before the run can satisfy the
multi-hour requirement.

## Backup and recovery

Stop writers, copy the configured data directory, record a cryptographic digest,
and restore into an empty directory. Start UAR and verify `/health`, then compare
the restored digest and a representative run. If the digest differs, quarantine
the restore, retain logs, and retry from the last known-good backup; never start
against a known-corrupt copy.

## Container checks

The certification workflow builds the image, runs it with `--user 65532:65532`,
sends `SIGTERM`, and requires `/health` to become ready. All configured cache,
skill, and data paths must be mounted writable by that UID. Treat permission,
signal, or health failures as release blockers.
