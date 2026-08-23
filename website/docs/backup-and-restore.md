---
sidebar_position: 5
title: Backup and Restore
description: Compatibility entry point for the current recovery and shutdown runbook.
---

# Backup and Restore

The current runbook is [Recover and Shut Down](/docs/operations/recovery-and-shutdown). It identifies the state owner for embedded SurrealKV, memory, remote Surreal, and PostgreSQL configurations; defines cold-backup discipline; and requires an isolated functional read-back before a restore is considered proven.

Stop an embedded datastore cleanly before copying it. Preserve companion files and required encryption-key custody separately. An archive listing or checksum proves copy integrity, not application recovery.
