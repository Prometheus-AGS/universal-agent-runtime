# Plan — `uar-native-service-deployment`

**Date:** 2026-08-23
**Backend:** OpenSpec
**Assessment:** `.kbd-orchestrator/phases/uar-native-service-deployment/assessment.md` with observed probes retained in `assessment-evidence.md`
**Execution:** five serial worktrees under `~/.claude/worktrees/`, one independently committed change at a time

## Objective

Build UAR 1.0.0 and its React application as a native, loopback-only service; install and run it locally as a macOS user LaunchAgent on port 1906; ship equivalent Linux systemd and Windows SCM packaging contracts; preserve operator state; bootstrap supported real-model providers without disclosing credentials; document the deployment; and retain bounded functional evidence.

## Locked decisions

| Decision | Contract |
|---|---|
| macOS service | User LaunchAgent `com.prometheus.universal-agent-runtime`, never a root LaunchDaemon. |
| Listener exposure | HTTP and A2A gRPC both inherit `server.host`; installed defaults are loopback-only. |
| Windows service | Native SCM integration through exactly `windows-service = "=0.8.1"`; no WinSW or NSSM. |
| Service credentials | Generated allowlisted environment file; never source the complete Bash profile during each service start. |
| Operator authority | Preserve existing `~/.uar/config.yaml`, database, selected model, and API/UI-created provider records. Back up before merging phase-owned entries. |
| Operational logs | macOS `~/.prometheus/logs/universal-agent-runtime/`; Linux `/var/lib/uar/.prometheus/logs/`; Windows `%ProgramData%\Prometheus\UniversalAgentRuntime\.prometheus\logs\`. Repository `.prometheus/` remains KBD history only. |
| Verification timing | Code-complete first. During implementation run only cheap Tier 0 compilation/static checks. Run bounded functional verification after all implementation changes land. |
| CI | GitHub Actions remain deployment-only; all development and functional verification is local. |
| Architecture record | Do not modify or stage the operator-owned `versions.toml`; record the Windows dependency pin and native-service decisions through KBD/OpenSpec history. |

## Verified build inputs

- `cargo info windows-service@0.8.1` observed crates.io version 0.8.1, Rust 1.71 compatibility, and the versioned docs.rs URL on 2026-08-23.
- Root `Cargo.toml` defines `server-full = ["minimal", "a2a-transport", "local-models", "cedar-governance", "response-quality", "document-intelligence", "telemetry", "api-docs", "admin-ui", "wasm-runtime"]`; `admin-ui` is the shipped React surface.
- The pinned `ort` prebuilt catalog has no `x86_64-pc-windows-gnu` artifact. Install `x86_64-pc-windows-msvc` and use a target-scoped `cargo-xwin` environment during change 2; compilation remains a compile-only claim. Do not export cargo-xwin's global `TARGET_CC`/`TARGET_CXX`, because host build dependencies must continue to use the macOS compiler.

## Ordered OpenSpec changes

### 1. `establish-native-service-deployment-contract`

Create the normative capability deltas and execution contract for native service lifecycle, platform paths, local-only listeners, configuration precedence, non-destructive provider bootstrap, log placement, bounded real-inference evidence, and platform-specific claim limits.

**Owned surface:** OpenSpec artifacts, this phase's contract/evidence templates, and append-only KBD decision records.
**Done:** `openspec validate establish-native-service-deployment-contract --strict` passes and every later change has an explicit requirement owner.

### 2. `implement-native-service-runtime-support`

- Add `--env-file <path>` with `UAR_ENV_FILE`; an explicitly selected unreadable or invalid file terminates startup.
- Add `UAR_LOG_FILE` as a tracing destination for service environments.
- Add the Windows-only `service` command and native SCM lifecycle adapter. Stop and Shutdown controls cancel through the existing graceful shutdown path.
- Make A2A gRPC resolve its bind address from `server.host` rather than `0.0.0.0`.
- Enrich YAML-defined provider configurations from the embedded catalog before first registration.
- Pin `windows-service` 0.8.1 only for Windows targets and update `Cargo.lock`.

**Owned surface:** root Cargo manifest/lock, CLI entry point, telemetry, server startup/shutdown, provider registry, and Windows service adapter.
**Done:** `cargo check --locked -p universal-agent-runtime --no-default-features --features server-full`, package-scoped Clippy, and target-scoped `cargo check --locked -p universal-agent-runtime --target x86_64-pc-windows-msvc --no-default-features --features server-full` exit zero before the change-2 commit. Source inspection confirms `Cli.env_file`, the `UAR_LOG_FILE` telemetry branch, Windows `Command::Service`, `start_server_with_shutdown`, gRPC host resolution, and the `enrich_provider_config` YAML call site are present. No functional campaign begins here.

### 3. `package-native-service-installers`

Add `packaging/native/` with:

- macOS LaunchAgent template plus install, upgrade, start/stop, and uninstall scripts using `~/.uar` and the macOS `.prometheus` log directory;
- Linux `uar.service` plus install/uninstall scripts using `/etc/uar`, `/var/lib/uar`, direct `ExecStart`, `EnvironmentFile`, `SIGTERM`, and `Restart=on-failure`;
- Windows PowerShell and `.cmd` entrypoints using `%ProgramFiles%` for program files and `%ProgramData%` for mutable state and logs;
- standalone supervisor templates and lifecycle entrypoints that do not reference not-yet-landed provider bootstrap helpers. Change 3 does not implement or duplicate environment-generation or YAML-merge logic.

Installer actions back up existing configuration beneath `~/.prometheus/backups/uar/` on macOS and never replace a pre-existing configuration wholesale.

**Owned surface:** supervisor templates and lifecycle entrypoints below `packaging/native/`; provider environment generation, default YAML, and YAML merge belong exclusively to change 4.
**Done:** templates and scripts are code-complete and pass local syntax/structure checks; Linux and Windows remain template/compile claims on the macOS host.

The packaging/bootstrap interface is fixed before either implementation lands:

- macOS env file: `~/.uar/service.env`; Linux: `/etc/uar/uar.env`; Windows: `%ProgramData%\Prometheus\UniversalAgentRuntime\config\uar.env`.
- Change 4 owns `packaging/native/common/generate-provider-env.sh --output <path>` and `packaging/native/common/merge-provider-config.sh --config <path> --env-file <path> --proxy-url <url>` plus the PowerShell equivalents used by Windows, and owns the narrow installer edits that invoke them.
- Generators return 0 only after an atomic restrictive-permission write, return nonzero without replacing the previous file on failure, and never print secret values. Change 3 remains independently valid without them; change 4 integrates them without duplicating supervisor logic and runs a structural check that every referenced helper exists.

### 4. `bootstrap-native-provider-model-configuration`

Generate an allowlisted service environment without displaying values. Canonical variables take precedence; aliases populate only absent canonical names:

- `KIMI_API_KEY` from `KIMI_CODING_API_KEY`, falling back to `KIMI_CODING_KEY` only when both the canonical name and first alias are absent;
- `MINIMAX_API_KEY` from `MINIMAX_KEY`;
- `DASHSCOPE_API_KEY` from `QWEN_API_KEY` or `QWEN_TOKEN_PLAN_API_KEY`;
- no cross-endpoint aliases for `MOONSHOT_API_KEY` or `ZAI_API_KEY`.

Merge only supported concrete provider/model entries: discovered local OpenAI-proxy models at `http://127.0.0.1:8181/v1`, `kimi-for-coding/k3`, `minimax/MiniMax-M3`, `alibaba/qwen3.8-max`, and matching Z.AI/GLM and Moonshot models when their credentials exist. Exclude tool-only and endpoint-less credentials. YAML stores environment references or non-secret metadata only. The native YAML explicitly sets `server.host: 127.0.0.1`, `server.port: 1906`, and `server.grpc_port: 50051`; server-full starts A2A gRPC, so absence of either configured listener is a failure.

**Owned surface:** provider bootstrap helpers, default native YAML, catalog-linked configuration, and redacted evidence.
**Done:** existing operator configuration and database authority remain intact; generated files have restrictive permissions; no secret appears in command output, tracked files, KBD artifacts, docs, or logs.

### 5. `document-and-deploy-native-services`

- Update root README, configuration/provider/deployment docs, and branded Docusaurus pages for macOS, Linux, and Windows.
- Build the React production bundle, then build UAR with `cargo build --locked --release --no-default-features --features server-full`.
- Install the release binary and bundle beneath `~/.uar`, bootstrap configuration, load the LaunchAgent, and conduct the bounded functional verification matrix below.
- Write row-form `verification.md` evidence and close all five OpenSpec task lists only after observed results satisfy their limits.
- Apply the operator-requested Qwen supersession discovered during installed verification: advance the `models.dev` and `vendor/git/liter-llm` gitlinks to reviewed commits containing released `qwen3.8-max`, refresh UAR's committed offline catalog snapshot from those pinned sources, then migrate only the exact obsolete native values `alibaba/qwen3.7-max`, `QWEN_TOKENPLAN_API_KEY`, and the phase-owned `qwen3-coder-plus` Alibaba seed to `alibaba/qwen3.8-max`, canonical `DASHSCOPE_API_KEY`, and the released Qwen model metadata. Other existing model, credential-reference, and provider values remain protected.

**Owned surface:** README, docs/Docusaurus, release/install evidence, final KBD/OpenSpec verification artifacts, the narrow Unix/PowerShell provider-bootstrap correction named above, the `models.dev` and `vendor/git/liter-llm` parent gitlinks, the resulting `Cargo.lock` path-package refresh, `scripts/refresh-provider-catalog.mjs`, and `catalog/provider_catalog.json` plus `catalog/SNAPSHOT.md`. This exception supersedes earlier ownership only for the observed post-commit defects; no source inside either submodule is authored here.
**Done:** every required macOS result is observed, Linux/Windows claims remain correctly scoped, all five strict validations pass, and no unmet real-provider requirement remains.

## Serial execution protocol

For each change in order:

1. Create `~/.claude/worktrees/<change>` from current `main` using `scripts/worktree-new.sh` and initialize submodules recursively.
2. Complete only that change's owned surface and required cheap Tier 0 checks.
3. Strict-validate the change, complete its implementation/static tasks, then commit it independently. Final cross-change functional task closure and every `verification.md` are explicitly owned by change 5, which may update task checkboxes in changes 1–4 but may not modify their implementation surfaces except for the explicitly documented Qwen 3.8 supersession correction.
4. Merge it forward to `main`, remove the completed worktree through `scripts/worktree-rm.sh`, then create the next worktree from the new `main`.

No sideways lane merges, pushes, tags, releases, PRs, or edits to unrelated/untracked files are permitted.

## Verification matrix

### During implementation — Tier 0 only

- Rust edits: package-scoped `cargo check` and `cargo clippy` commands required by `.claude/rules/rust.md`; change 2 also installs the Windows Rust target and runs its compile-only `cargo check` before committing; no unit-test campaign.
- Frontend/docs edits: required type/static/build checks from `.claude/rules/typescript.md`; the Docusaurus production build waits until code completion.
- Packaging edits: local parser/syntax checks only.

### After code completion — bounded functional verification

| Requirement | Observation | Limit |
|---|---|---|
| Native package structure | Parse shell/PowerShell syntax, plist, and systemd unit; install the Rust `x86_64-pc-windows-msvc` target and compile the Windows-only adapter through a target-scoped cargo-xwin environment; strict-validate all OpenSpec changes; build Docusaurus locally. | Linux templates require root at install time; Windows templates require an elevated PowerShell session. No Linux or Windows runtime claim from macOS. |
| Release installation | Install the release server-full binary and current React bundle beneath `~/.uar`; load the LaunchAgent. | Preserve and back up existing state. |
| Service health | Observe the LaunchAgent remaining running; call `/healthz`, `/readyz`, `/`, and one emitted `/assets/*` path at `http://127.0.0.1:1906`. | Short requests only; no soak. The shipped host family is IPv4 loopback; `localhost` is not used as binding evidence. |
| Listener isolation | Run `/usr/sbin/lsof -nP -iTCP:1906 -sTCP:LISTEN` and the equivalent command for 50051; require at least one LISTEN row for each port and inspect every returned IPv4/IPv6 address. | Zero rows, wildcard addresses, and non-loopback addresses fail. Every observed address must be `127.0.0.1` or `::1`. |
| Native supervisor cancellation | Stop the macOS LaunchAgent and observe the existing process signal path complete its graceful coordinator; compile the Windows handler that maps SCM Stop/Shutdown to the process-scoped `CancellationToken` accepted by `start_server_with_shutdown`. | The macOS observation proves LaunchAgent/SIGTERM behavior only. Windows evidence is source plus cross-compile and does not claim SCM runtime behavior on macOS. |
| Environment/log fail-closed controls | Start the completed binary once with an explicitly missing or malformed env file and once with an unwritable `UAR_LOG_FILE`; inspect exit status and listeners. | Each exits nonzero with contextual output before opening HTTP or gRPC. These are bounded startup controls, not a test campaign. |
| Provider visibility | Observe `/api/uar/providers` and `/api/models`, then the shipped React `/admin/providers` and `/admin/models` surfaces. | Authenticate with the installed configuration's supported JWT/API-key flow only when `security.jwt_required` is true. Tokens are passed through process environment or browser session storage and never appear in command arguments or retained evidence. |
| Required genuine inference | Send genuine requests through installed UAR using the local OpenAI proxy, Kimi K3, and MiniMax M3, and observe model-produced responses through `/v1/chat/completions` and the shipped chat UI. | These three providers are mandatory. At most six total inference requests, each with a 120-second timeout and output capped at 64 tokens; no mock, recorded, replayed, synthetic, or multi-hour run counts. |
| Conditional provider visibility | When matching credentials exist, observe Alibaba/Qwen, Z.AI/GLM, and Moonshot in `/api/uar/providers`, `/api/models`, and the UI. | Visibility is required when its credential is present; inference is not required for these conditional providers in this phase. |
| Release identity | Run the installed binary's `--version` command. | Output must identify `universal-agent-runtime 1.0.0`. |
| Restart behavior | Before restart record one persisted provider ID from `/api/uar/providers` and `/api/config/persistence`; restart the LaunchAgent, fetch both again, then re-observe one representative genuine inference, graceful shutdown, and service logging. | The same provider row remains visible and the existing persistence response remains unchanged; one restart cycle, with additional attempts only to diagnose an observed failure. |
| Log placement | Observe stdout, stderr, and operational output only below the installed `.prometheus` log directory. | Database-engine internal LOG files are state, not operator logs. |

Evidence rows record command, observed output, source SHA, profile/platform, and claim limit. Secret presence is represented only as `<redacted:env-present>`; credentials and bearer values never enter retained commands or output.

## Stop conditions

Stop before reflection and report the exact unmet requirement if:

- a required real-provider credential, capacity, endpoint, model, or network path is unavailable;
- the merge would change any pre-existing YAML mapping/scalar/list value, delete a top-level YAML key, change the selected default model, or modify any already-persisted provider row rather than adding an absent phase-owned seed, except for the operator-authorized exact migration from `alibaba/qwen3.7-max` to `alibaba/qwen3.8-max`, from malformed `QWEN_TOKENPLAN_API_KEY` to canonical `DASHSCOPE_API_KEY`, and from the phase-owned `qwen3-coder-plus` seed to the released Qwen 3.8 model;
- Windows SCM cannot use the runtime's existing cancellation path without a broader architecture change;
- a platform template requires an undeclared third-party service wrapper;
- either installed listener remains non-loopback;
- verification would require printing or retaining a secret;
- implementation crosses the per-change owned surfaces or touches unrelated operator files.

## Review gate

Before Execute, submit this plan and the five OpenSpec contracts as isolated artifacts to:

- MiniMax M3 in the critic role; and
- Kimi K3 in the judge role.

Critical findings block Execute. Warnings are either resolved or explicitly carried into the execution contract. The assessment warning about retained probe evidence is resolved by `assessment-evidence.md`; the warning about log paths is resolved by treating the operator-approved paths above as locked requirements.

### Review disposition after the two-round artifact cap

- MiniMax M3 critic round 1 blocked on missing dependency/profile evidence and an unverified cancellation seam. The plan added observed crate/feature evidence and a process-scoped seam. MiniMax round 2 returned PASS; its two warnings were also resolved by fixing helper ownership and naming `start_server_with_shutdown`.
- Kimi K3 judge round 1 blocked on a vacuous gRPC listener assertion. The plan now requires a non-empty port-50051 LISTEN result and makes the native YAML owner configure that port. Kimi round 2 identified a verification-timing contradiction; the functional checks were moved from change 2 to change 5. Its warnings were resolved by moving Windows compilation/version checks into change 2 and making change 3 independently valid before change 4 integration.
- The artifact review skill permits two correction rounds. No critical or warning is intentionally left unresolved; the final corrections are recorded here rather than initiating an unbounded third model review.
- Installed verification exposed a restart failure caused by the pre-existing malformed `QWEN_TOKENPLAN_API_KEY` reference. Execution stopped as required. On 2026-08-23 the operator superseded the Alibaba selection with Qwen 3.8-Max. Alibaba's current documentation identifies the released API model as `qwen3.8-max`, with the preview retired; environment inspection showed `QWEN_API_KEY` is present and wins the already-locked alias precedence, so the Singapore pay-as-you-go endpoint remains correct. The first corrected restart then exposed that the pinned compile-time catalog predated the release. The operator authorized advancing the `models.dev` pointer and then the updated `liter-llm` pointer. Because release builds embed the reviewed `catalog/provider_catalog.json` rather than reading either submodule at runtime, change 5 owns both gitlink advances, the deterministic offline snapshot refresh path, the narrow native correction, and a no-additional-inference restart check.

## Completion sequence

After every functional exit criterion passes:

1. strict-validate and archive all five OpenSpec changes;
2. transition canonical KBD progress through the runtime;
3. run `/kbd-reflect uar-native-service-deployment` automatically;
4. record plan-versus-delivery differences, failures/fixes, platform limits, and durable lessons in append-only `.prometheus` history;
5. confirm `main` contains every completed change and only then remove all phase worktrees.

The phase ends without push, tag, package publication, release publication, or PR creation.
