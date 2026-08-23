---
sidebar_position: 13
title: Upgrade and Rollback
description: Upgrade UAR with verified artifacts, cold backups, functional checks, and an explicit rollback boundary.
source_records:
  - docs/compatibility-policy.md
  - docs/DEPLOYMENT.md
current_authority: /docs/upgrade-guide
---

# Upgrade and rollback

## Boundary statement

**An upgrade is complete only when the target artifact and migrated state pass
representative functional verification, and rollback is possible only within
the release's declared data compatibility boundary.** A build or liveness check
does not prove either condition.

## Support boundary

The current security policy supports the `1.0.x` line. Stable 1.x HTTP routes,
event profiles, configuration keys, release artifact names, and supported
persistent data remain compatible except where a documented security fix must
reject unsafe prior behavior. Preview and Experimental surfaces can change in a
minor release.

Read the target release notes, security notes, compatibility policy, and any
explicit migration before touching the running deployment.

## Backup prerequisite

Before upgrading:

1. record the current immutable artifact digest and effective configuration;
2. stop writes or establish the provider-specific consistent snapshot boundary;
3. back up application persistence, memory when enabled, uploads, policies, and
   operator configuration;
4. restore that backup into an isolated location;
5. perform functional read-back of representative agents, skills, sessions,
   knowledge, and settings.

Archive creation alone is not restore evidence. Follow
[Recovery and shutdown](./operations/recovery-and-shutdown.md).

## Immutable version selection

Verify the target release manifest and checksums, then pin the archive or image
digest. Record both old and new identities in the change record. Do not upgrade
from `main`, a floating tag, or an unverified locally named image.

## Configuration and data compatibility

Compare the target examples and generated configuration schema with the
effective current configuration. Pay particular attention to:

- authentication issuer, audience, JWKS/secret, and tenant behavior;
- persistence provider, URL, credentials, and feature composition;
- embedding model and vector dimension;
- provider/model names and credential resolution;
- enabled native/MCP tools, Cedar policies, and approval behavior;
- retention, backup, and shutdown deadlines.

Keep the persistence provider and location stable unless the release supplies a
tested migration. Never start an older binary against data changed by a newer
engine unless the release explicitly declares backward readability.

## Apply the upgrade

For containers, replace the image with the verified digest and use the
deployment mechanism's documented drain/restart behavior. For source installs,
check out the verified tag, initialize submodules, build from locked
dependencies, and replace the binary through the process manager. Do not
combine an application upgrade with an unrelated datastore, model, policy, or
network migration.

## Functional verification

After startup, check in increasing depth:

1. `/healthz` returns liveness;
2. `/readyz` reports configured dependencies ready;
3. authenticated reads find pre-upgrade resources;
4. one reversible write survives reload/restart;
5. provider routing reaches the intended real model and returns genuine
   inference;
6. a representative skill, knowledge, agent, tool-policy, and realtime path
   behaves as the release requires.

Report each profile and platform separately. A successful documentation build,
binary build, probe, or synthetic response is not this functional evidence.

## Rollback

If the target fails its gate, stop writes and redeploy the prior verified
artifact. Restore the pre-upgrade data when the failed attempt changed state in
a way the prior version cannot read. Reapply the recorded prior configuration
and repeat the same functional checks.

If the release declares an irreversible migration and no tested inverse or
restore is available, rollback is blocked. Do not improvise a data conversion
against production state.

## Profile limits

Server upgrade evidence is separate for `server-full` and `minimal`.
`embedded-mobile` upgrades belong to each host application and platform package
and must cover the supplied persistence and inference adapters. No server,
iOS, Android, or desktop result transfers to another profile or platform.

Next: [Troubleshooting](./troubleshooting.md).
