## Context

All platforms launch the same binary but differ in supervisor syntax and writable-path policy. Templates remain source-controlled while installers render absolute local paths and restrictive permissions.

## Decisions

- macOS installs the binary, static bundle, YAML, and environment beneath `~/.uar`; plist logs point to `~/.prometheus/logs/universal-agent-runtime/`.
- Linux installs configuration beneath `/etc/uar`, mutable state beneath `/var/lib/uar`, and uses direct systemd `ExecStart`, `WorkingDirectory`, `EnvironmentFile`, `Restart=on-failure`, and SIGTERM.
- Windows stores immutable program files beneath `%ProgramFiles%` and mutable state beneath `%ProgramData%`, and registration calls UAR's native `service` command.
- Change 3's lifecycle entrypoints are independently runnable without provider-bootstrap helpers. Change 4 owns the later narrow integration that invokes its generated environment and additive YAML merge commands.
- Uninstall defaults to preserving mutable state; destructive state removal requires a separate explicit operator action outside this change.

## Risks

- Supervisor templates cannot contain user-specific unresolved paths at install time.
- Shell or PowerShell output must never print generated credential values.
