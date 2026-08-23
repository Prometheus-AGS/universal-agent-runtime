ASSESSMENT: uar-native-service-deployment
Project: universal-agent-runtime
Date: 2026-08-23
Codebase baseline: main at the completed UAR 1.0 documentation boundary; native service packaging is absent, while the server-full runtime and React production bundle already exist.
Cross-tool progress: none for this new phase

IMPLEMENTATION STATUS
- Release build surface: [PARTIAL] — Cargo defines universal-agent-runtime 1.0.0 and server-full; the React build emits static/, but no target/release binary is currently installed.
- macOS service: [MISSING] — no UAR LaunchAgent is loaded and no tracked launchd template or installer exists. An observed `lsof -nP -iTCP:1906 -sTCP:LISTEN` probe returned no listener on 2026-08-23. An observed `find ~/.uar -maxdepth 3` inventory found config.yaml, data/, mcp.json, runtime/, static/, and uploads/.
- Linux service: [MISSING] — no tracked systemd unit or native Linux installer exists.
- Windows service: [MISSING] — no SCM adapter or installer exists. `vendor/git/prometheus-parking-lot-rs/src/core/shutdown.rs::wait_for_signal` handles SIGTERM only under Unix and waits only for Ctrl-C under non-Unix, which does not establish correct SCM Stop handling.
- Native configuration: [PARTIAL] — explicit --config/CONFIG_FILE, ./config.yaml, and ~/.uar/config.yaml are supported. There is no explicit service environment-file input.
- Service logging: [PARTIAL] — `src/uar/telemetry/mod.rs::init` writes tracing output to stdout/stderr and supports JSON/compact/pretty formatting, but it has no file writer for a Windows service without inherited standard handles. Repository `.prometheus/` is version-controlled KBD history and must not receive machine runtime logs. The required native destinations are external per-install `.prometheus/logs` directories: `~/.prometheus/logs/universal-agent-runtime/` on macOS, `/var/lib/uar/.prometheus/logs/` on Linux, and `%ProgramData%\\Prometheus\\UniversalAgentRuntime\\.prometheus\\logs\\` on Windows. No current service sink targets them.
- Listener isolation: [PARTIAL] — HTTP honors server.host, but `src/server.rs` at the A2A gRPC startup constructs `0.0.0.0:{grpc_port}`, so a loopback HTTP configuration still exposes gRPC.
- Provider bootstrap: [PARTIAL] — `catalog/provider_catalog.json` contains Kimi k3, MiniMax-M3, Moonshot, Alibaba/Qwen, and Z.AI/GLM. `src/llm/registry.rs::seed_from_llm_config` calls `enrich_provider_config`, but `seed_from_configs` stores YAML provider entries without calling it, so omitted catalog keys/models are not resolved on the YAML path.
- Provider persistence: [DONE] — settings storage intentionally seeds missing providers once and then gives API/UI-managed database rows precedence. Installation must preserve that authority rather than overwrite it.
- Local model proxy: [DONE] — an observed `curl -fsS --max-time 5 http://127.0.0.1:8181/v1/models` probe on 2026-08-23 returned gpt-5.5, gpt-5.4 variants, and gpt-5.6 sol/terra/luna.
- Bounded real-model verification: [PARTIAL] — UAR exposes `/v1/chat/completions` and the shipped React chat/provider UI. Existing `test:e2e:real-provider-route` and `test:e2e:real-knowledge-rag` scripts explicitly build `stub-llm`, and `scripts/live-integration.sh` supports non-certifying recorded behavior; neither can prove this goal. The phase therefore needs short post-install API and browser requests whose evidence identifies the real provider/model and response.
- Native deployment documentation: [MISSING] — current README, docs, and Docusaurus coverage addresses source, container, and Kubernetes deployment but not launchd, systemd, or Windows SCM installation.

CROSS-TOOL PROGRESS
- NONE — the new phase progress ledger contains no registered changes.

SPEC GAP SUMMARY
- No canonical native-service capability specifies platform paths, service identifiers, graceful stop semantics, log placement, or install/upgrade/uninstall behavior.
- The graceful-shutdown contract does not cover Windows SCM control events.
- The A2A gRPC specification does not require the transport to inherit server.host, leaving a concrete local exposure defect.
- Provider/model settings do not specify catalog enrichment for YAML-defined providers or least-privilege service credential aliases.
- Customer documentation does not cover native service installation or its profile/platform evidence limits.

BUILD HEALTH
- build check: [UNKNOWN] — intentionally not run during Assess; the approved phase requires implementation to become code-complete before bounded functional verification.
- known violations: server-full gRPC binds 0.0.0.0 independently of server.host; Windows service stops cannot currently reach the runtime cancellation token; YAML provider seeding bypasses catalog enrichment.
- test coverage: [PARTIAL] — shutdown, configuration, providers, and release checks exist, but there are no native service installation scenarios.

CONSTRAINT CHECK
- AGENTS.md violations: NONE introduced by this phase. `.github/workflows/deploy.yml` and `.github/workflows/docs.yml` are the only allowed workflow names enforced by `scripts/validate-github-actions-policy.mjs`; this phase must not add development tests to either workflow.
- constraints.md violations: N/A — no additional phase constraint file exists.
- dirty-worktree constraint: the KBD runtime refreshed canonical projections across prior phases. Unrelated untracked files and the operator-owned versions.toml must remain unstaged and unmodified.

GOAL PROGRESS
- Build and install UAR 1.0.0 as a loopback-only macOS LaunchAgent on port 1906: [NOT MET] — no release binary or LaunchAgent is installed.
- Ship launchd, systemd, and Windows SCM packaging with graceful shutdown and .prometheus logs: [NOT MET] — all three native packages are absent.
- Bootstrap least-privilege supported provider/model configuration: [PARTIAL] — catalog entries and credential sources exist, but the service env file, aliases, YAML seed, and installed provider state do not.
- Document native installation and run bounded real-model verification: [NOT MET] — native docs and installed-service evidence are absent, and the existing stub/recorded scripts do not satisfy the real-inference rule. Evidence must come from short API and browser requests through the installed service.

UNCOMFORTABLE FINDING
- A configuration that appears local-only today is not local-only under server-full because gRPC ignores server.host. Starting the requested service without correcting this would silently expose an authenticated execution surface beyond the machine.

ASSESSMENT COMPLETE
