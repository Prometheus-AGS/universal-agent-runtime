# Artifact critic — Surreal lifecycle scope expansion

Date: 2026-08-22

Verdict: PASS

The observed same-path failure is valid. The second UAR fails while the first
helper remains alive. Current ownership confirms that untracked
`LiveQueryBus` supervisors retain cloned Surreal clients and the detached
ingestion watcher retains `IngestService`.

Adding only `src/uar/realtime/surreal_bus.rs` is the minimum product-scope
expansion. `src/server.rs` is already authorized for watcher ownership and
completion sequencing. The realtime file is the only additional location
needed to retain, cancel, and join topic supervisors without changing the
public `RealtimeBus` trait.

`std::fs::File::try_lock` is compatible with SurrealKV's exclusive `LOCK`
operation. The observer must open the existing file read/write without create
or truncate, treat `WouldBlock` as cleanup incomplete, surface other I/O
errors, and unlock/drop immediately after success.
