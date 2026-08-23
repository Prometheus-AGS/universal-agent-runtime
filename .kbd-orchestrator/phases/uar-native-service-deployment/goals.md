# Goals

- build and install the UAR 1.0.0 server-full release as a loopback-only macOS LaunchAgent on port 1906
- ship native launchd, systemd, and Windows SCM packaging with graceful shutdown and .prometheus logs
- bootstrap least-privilege provider configuration for the local proxy, Kimi K3, MiniMax M3, Qwen, GLM, and Moonshot where credentials exist
- document native installation and verify short real-model inference without synthetic or long-running tests
