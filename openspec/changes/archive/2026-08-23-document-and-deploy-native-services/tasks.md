## 1. Documentation

- [x] 1.1 Update the root README and native deployment/configuration/provider documentation.
- [x] 1.2 Add branded Docusaurus macOS, Linux, and Windows installation pages and navigation.
- [x] 1.3 Document install, upgrade, uninstall, credential refresh, precedence, paths, controls, logs, exposure, troubleshooting, and evidence limits.

## 2. Code-complete build and install

- [x] 2.1 Confirm all preceding implementation tasks are code-complete before starting bounded functional verification.
- [x] 2.2 Build the React production bundle, then `cargo build --locked --release --no-default-features --features server-full`.
- [x] 2.3 Validate installer syntax, plist/systemd structure, Windows target compile/template, all five strict OpenSpec changes, and the local Docusaurus production build.
- [x] 2.4 Back up and merge existing configuration, install the release binary/static bundle beneath `~/.uar`, and load the LaunchAgent.
- [x] 2.5 Run the installed binary with `--version` and observe `universal-agent-runtime 1.0.0`.
- [x] 2.6 Replace the phase-owned Alibaba seed with released `qwen3.8-max` metadata and add an exact-match migration for `alibaba/qwen3.7-max` plus malformed `QWEN_TOKENPLAN_API_KEY`, with Unix/PowerShell parity and a preservation negative control.
- [x] 2.7 Advance the `models.dev` and `vendor/git/liter-llm` parent gitlinks to reviewed commits containing released `qwen3.8-max`, refresh the path package's `Cargo.lock` entry and committed offline UAR catalog snapshot from the pinned sources, leave submodule source unmodified, and preserve the existing catalog-backed `/api/models` implementation.

## 3. Bounded functional verification

- [x] 3.1 Observe LaunchAgent running status, `/healthz`, `/readyz`, shipped UI, and static assets on localhost:1906.
- [x] 3.2 Observe at least one LISTEN row for HTTP 1906 and A2A gRPC 50051 and require every row to be loopback-only; zero rows fail.
- [x] 3.3 Observe configured providers/models through `/api/uar/providers`, `/api/models`, `/admin/providers`, and `/admin/models` without exposing credentials; require conditional Qwen, GLM, and Moonshot visibility when matching credentials exist.
- [x] 3.4 Observe genuine installed-UAR inference through `/v1/chat/completions` and the shipped chat UI for the required local OpenAI proxy, Kimi K3, and MiniMax M3, using no more than six total 120-second/64-token requests.
- [x] 3.5 Before restart record one persisted provider ID and `/api/config/persistence`; restart once, require the same provider and unchanged persistence response, then re-observe one representative genuine inference, graceful shutdown, and `.prometheus` logging.
- [x] 3.6 Write separate row-form server-full/macOS runtime, Linux-template, and Windows compile/template evidence.
- [x] 3.7 Stop the macOS LaunchAgent and observe graceful completion/resource release through the process signal path; pair that observation with the already-passing Windows handler cross-compile and source trace from SCM Stop/Shutdown to `start_server_with_shutdown`, without claiming Windows runtime behavior.
- [x] 3.8 Observe missing/malformed explicit env files and an unwritable `UAR_LOG_FILE` each exit nonzero before either network listener opens.
- [x] 3.9 Refresh the installed native configuration, restart without another inference request, and observe selected `alibaba/qwen3.8-max`, canonical `DASHSCOPE_API_KEY` indirection, Qwen 3.8 provider/model visibility, and service readiness.

## 4. Closeout

- [x] 4.1 As the owner of final OpenSpec verification artifacts, complete remaining functional checkboxes in changes 1–4 only after their observed evidence passes; do not edit their implementation surfaces. Stop before reflection on a required real-provider failure.
- [x] 4.2 Strict-validate and archive all five changes through OpenSpec.
- [x] 4.3 Complete KBD Execute and run `/kbd-reflect uar-native-service-deployment` automatically.
- [x] 4.4 Merge all completed changes to `main`, remove phase worktrees, and confirm no push, tag, publication, or PR occurred.
