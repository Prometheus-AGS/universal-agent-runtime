# Crash-report token confinement (task 2.23)

Manual evidence, 2026-09-23, macOS (Darwin 25.6.0, arm64).

## Procedure

1. Binary: `uar_sidecar-c9587cd5498d5ae3` from the gate build of commit `e44846af`
   (`/Volumes/my-passport/cargo-build/4a/8e0717ca42c200/debug/deps/`).
2. Sanity: empty stdin → exit status 2 (no token line, no `READY`).
3. Fresh random 64-hex-character token written as the first stdin line through a FIFO held open;
   working directory a fresh temp dir. Sidecar printed `READY:60022`.
4. `kill -ABRT <pid>` → the sidecar terminated.
5. macOS ReportCrash wrote `~/Library/Logs/DiagnosticReports/uar_sidecar-c9587cd5498d5ae3-2026-09-23-134639.ips`.
6. Byte search for the token in every new crash report, and in the sidecar's stdout, stderr and working
   directory (data files).

## Result

| Location | Token occurrences |
|---|---|
| New crash report (`.ips`) | 0 |
| stdout, stderr | 0 |
| Working directory / data files | 0 |

## Not tested

- **Linux:** no core-dump or systemd-coredump search was run (no Linux machine in this session).
- **Windows x64 / ARM64:** not tested. `taskkill /F` is not a crash, and no debug-only abort hook was
  used, so no Windows Error Reporting dump was produced.

A core dump that includes process memory would contain the token while the process is live; this check
covers the crash *report*, not a full memory dump, which is outside the threat model (same-user access to
process memory).
