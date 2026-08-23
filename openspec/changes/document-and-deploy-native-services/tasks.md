## 1. Documentation

- [ ] 1.1 Update the root README and native deployment/configuration/provider documentation.
- [ ] 1.2 Add branded Docusaurus macOS, Linux, and Windows installation pages and navigation.
- [ ] 1.3 Document install, upgrade, uninstall, credential refresh, precedence, paths, controls, logs, exposure, troubleshooting, and evidence limits.

## 2. Code-complete build and install

- [ ] 2.1 Confirm all preceding implementation tasks are code-complete before starting bounded functional verification.
- [ ] 2.2 Build the React production bundle, then `cargo build --locked --release --no-default-features --features server-full`.
- [ ] 2.3 Validate installer syntax, plist/systemd structure, Windows target compile/template, all five strict OpenSpec changes, and the local Docusaurus production build.
- [ ] 2.4 Back up and merge existing configuration, install the release binary/static bundle beneath `~/.uar`, and load the LaunchAgent.
- [ ] 2.5 Run the installed binary with `--version` and observe `universal-agent-runtime 1.0.0`.

## 3. Bounded functional verification

- [ ] 3.1 Observe LaunchAgent running status, `/healthz`, `/readyz`, shipped UI, and static assets on localhost:1906.
- [ ] 3.2 Observe at least one LISTEN row for HTTP 1906 and A2A gRPC 50051 and require every row to be loopback-only; zero rows fail.
- [ ] 3.3 Observe configured providers/models through `/api/uar/providers`, `/api/models`, `/admin/providers`, and `/admin/models` without exposing credentials; require conditional Qwen, GLM, and Moonshot visibility when matching credentials exist.
- [ ] 3.4 Observe genuine installed-UAR inference through `/v1/chat/completions` and the shipped chat UI for the required local OpenAI proxy, Kimi K3, and MiniMax M3, using no more than six total 120-second/64-token requests.
- [ ] 3.5 Before restart record one persisted provider ID and `/api/config/persistence`; restart once, require the same provider and embedded persistence response, then re-observe one representative genuine inference, graceful shutdown, and `.prometheus` logging.
- [ ] 3.6 Write separate row-form server-full/macOS runtime, Linux-template, and Windows compile/template evidence.
- [ ] 3.7 Drive the completed platform-neutral external cancellation seam through server-full and observe graceful completion/resource release; pair that observation with the already-passing Windows handler compile without claiming Windows runtime behavior.
- [ ] 3.8 Observe missing/malformed explicit env files and an unwritable `UAR_LOG_FILE` each exit nonzero before either network listener opens.

## 4. Closeout

- [ ] 4.1 As the owner of final OpenSpec verification artifacts, complete remaining functional checkboxes in changes 1–4 only after their observed evidence passes; do not edit their implementation surfaces. Stop before reflection on a required real-provider failure.
- [ ] 4.2 Strict-validate and archive all five changes through OpenSpec.
- [ ] 4.3 Complete KBD Execute and run `/kbd-reflect uar-native-service-deployment` automatically.
- [ ] 4.4 Merge all completed changes to `main`, remove phase worktrees, and confirm no push, tag, publication, or PR occurred.
