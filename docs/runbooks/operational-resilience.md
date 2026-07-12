# Operational resilience certification

Run `scripts/certify-operational-resilience.sh`. The deterministic suite writes
`target/resilience-certification/results.json` and `test.log`; CI retains both.

## Published limits

- 100 parallel simulated runs, zero errors, p95 under 250 ms.
- Zero duplicate event IDs after reconnect/replay.
- Lifecycle and tool waits are bounded; the certification job has a 60-minute ceiling.
- Provider/MCP failures must reach an explicit error or recover on a later bounded attempt.

For a release candidate, run the scheduled workflow for at least three hours by
setting `UAR_SOAK_DURATION_SECONDS=10800`. The deterministic PR suite is a short
certification of the same reconnect and deduplication invariants, not a claim
that a multi-hour soak ran on every commit.

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
