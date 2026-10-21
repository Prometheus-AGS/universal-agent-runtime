# Delivery startup Send repair

## Scope

This source-only repair addresses the two `Send` diagnostics from approved Docker run `39528865081` at source identity `6d50cb8a`, ARM job `118989856797`.

## Changed paths

- `src/server.rs`
  - Await the native skill registry count before the startup `info!` macro.
  - Await the governance policy count before the startup `info!` macro.

Both sites execute in the retained `tokio::spawn` startup owner. Moving the awaited values into local integers prevents tracing macro arguments from retaining `std::fmt::Arguments` or `&dyn tracing::Value` across an await. Readiness, shutdown, and retained-task ownership ordering are unchanged.

## Verification status

No verification command was run. The phase verification gate remains CLOSED, so the repaired startup owner is unverified until the lead obtains authorization for the designated delivery build.
