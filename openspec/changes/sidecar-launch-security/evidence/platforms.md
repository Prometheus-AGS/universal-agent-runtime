# Per-platform verification (task 4.2)

2026-09-23. Tasks 2.13 (listener surface) and 2.19 (token absent from argv/env of the sidecar and its
children) were run on one platform only.

| Platform | 2.13 | 2.19 | Evidence |
|---|---|---|---|
| macOS arm64 (Darwin 25.6.0) | passed | passed | `gate-run-1.txt`; re-run independently by the coordinating session: `sidecar_launch_security` 34/34, `mcp_child_environment` 10/10 |
| Linux x64 | unverified | unverified | no Linux machine used in this session |
| Windows x64 | unverified | unverified | no Windows machine used in this session |
| Windows ARM64 | unverified | unverified | no Windows machine used in this session |

Windows is the platform the-boss's mini skill pack targets first, so the Windows rows stay open risks
until they are run on Windows hardware (not GitHub Actions — UAR's policy keeps tests local).
