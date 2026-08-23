## 1. macOS

- [ ] 1.1 Add the user LaunchAgent template and install/uninstall/control scripts for `com.prometheus.universal-agent-runtime`.
- [ ] 1.2 Render loopback host, port 1906, explicit config/env/log paths, restart-on-failure behavior, and restrictive permissions.

## 2. Linux

- [ ] 2.1 Add `uar.service` with direct `ExecStart`, `WorkingDirectory`, `EnvironmentFile`, SIGTERM, and `Restart=on-failure`.
- [ ] 2.2 Add install/uninstall/control scripts for `/etc/uar` and `/var/lib/uar`, preserving mutable state by default.

## 3. Windows

- [ ] 3.1 Add PowerShell install/uninstall/control entrypoints and `.cmd` wrappers using Program Files and ProgramData.
- [ ] 3.2 Register the native UAR SCM command and configure the required `.prometheus` log directory.

## 4. Safety and cheap verification

- [ ] 4.1 Back up existing configuration before merge and prohibit wholesale replacement.
- [ ] 4.2 Run shell/PowerShell syntax and plist/systemd structure checks only after packaging is code-complete.
- [ ] 4.3 Strict-validate the change before committing it independently.
- [ ] 4.4 Confirm no change-3 entrypoint references a helper that is not present in the same commit.
