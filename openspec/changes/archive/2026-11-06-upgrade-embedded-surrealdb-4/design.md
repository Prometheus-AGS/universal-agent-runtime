# Embedded SurrealDB 4.0 design — recovered implementation

This document summarizes the surviving implementation and adopted contract after worktree loss. It does not reconstruct a missing original file or claim fresh acceptance.

The shipping SDK, types, embedded engine and patch closure use 4.0.0. UAR retains owner-scoped persistence and distinct owned/borrowed lifecycle handling. The embedded engine closes through its awaited native shutdown; runtime admission and drain safeguards remain in force. Existing integration fixtures cover retained clients, live subscriptions, producer settlement, borrowed handles and uncertain shutdown.

Ordinary startup refuses legacy 3.3 storage. The source-built `surrealql_store_runner --migrate-3.3` copies a stopped eligible 3.3 snapshot to a new distinct destination, then opts that copy into the typed SDK conversion. The immutable source is never mounted into the 4.0 runtime. The runner rejects overlapping paths and retains interrupted destinations for quarantine. `--restore-3.3` restores only the baseline into another new volume after explicit disposition of post-baseline accepted writes.

The supported baseline is the exact RC2 Linux amd64 artifact and recorded schema/configuration/data classes. The restored runbook and existing parity/recovery scripts define the fixture and assertions. Source equality is insufficient: installed behavior must establish owner isolation, durable identities, query/index membership, ordering, refusal integrity, restart and restored service behavior. The unchanged five adopted scenarios govern closure.

Remote memory adoption, platform expansion, PostgreSQL parity, in-place conversion and live migration remain excluded. Memory-disabled fixture coverage does not qualify enabled memory migration.

The uncomfortable limit is lost original metadata: the deleted source inventory and canonical event journal cannot be recreated from receipts. A new isolated KBD identity and a fresh full source inventory bind current evidence; preserved receipts remain historical. The surviving BuildKit input archive separately binds the running candidate image.
