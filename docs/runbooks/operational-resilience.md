# Operational resilience certification

From a clean detached checkout of the candidate commit, run:

```bash
scripts/certify-operational-resilience-local.sh preflight
scripts/certify-operational-resilience-local.sh certify
```

The first command defaults to a 60-second local wiring check. The second
requires at least 10,800 seconds and writes the candidate archive plus complete
machine-readable evidence under `target/`. Neither command runs in GitHub
Actions.

## Published limits

- 100 parallel simulated runs, zero errors, p95 under 250 ms.
- 20 parallel installed-runtime requests, zero failures.
- Installed streaming soak: zero errors, zero duplicate events, p95 at or below
  2,000 ms, and peak RSS growth at or below 262,144 KiB.
- Lifecycle and tool waits are bounded; the local certifying process must be
  allowed to finish the full configured duration.
- Provider/MCP failures must reach an explicit error or recover on a later bounded attempt.

The 60-second local preflight is not multi-hour certification evidence. The
`certify` mode defaults to 10,800 seconds and rejects a smaller
`UAR_SOAK_DURATION_SECONDS`. The retained `soak.json` must show both configured
and observed duration before the run can satisfy the multi-hour requirement.

## Backup and recovery

Stop writers, copy the configured data directory, record a cryptographic digest,
and restore into an empty directory. Start UAR and verify `/health`, then compare
the restored digest and a representative run. If the digest differs, quarantine
the restore, retain logs, and retry from the last known-good backup; never start
against a known-corrupt copy.

## Container checks

The local certification launcher builds the image, runs it with
`--user 65532:65532`, sends `SIGTERM`, and requires `/health` to become ready.
All configured cache, skill, and data paths must be mounted writable by that
UID. Treat permission, signal, or health failures as release blockers.

GitHub Actions are reserved for actual deployment execution and
deployment-specific validation. Product certification, including this runbook,
must remain local even when it builds an installed archive or container image.
