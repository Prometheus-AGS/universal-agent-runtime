# UAR core team

Seven project-local roles support bounded work on Universal Agent Runtime. Start with one lead and add specialists only for independent work. The default concurrency budget is three active agents, with a maximum of four, including the lead. This is team policy; native scheduler limits and permissions remain authoritative.

| Role | Responsibility |
|---|---|
| `uar-lead` | Restore project position, define acceptance criteria, assign ownership, integrate results, and report evidence. |
| `uar-runtime` | Runtime execution, sessions, scheduling, and lifecycle behavior. |
| `uar-providers` | Provider adapters, model selection, transport, and usage accounting. |
| `uar-trust-tools` | Authentication, authorization, tool execution, and approval boundaries. |
| `uar-state` | Persistence, domain records, memory, and state contracts. |
| `uar-console` | Console UI, platform entity hooks, and user workflows. |
| `uar-verifier` | Independent acceptance, regression, and security-boundary review. |

Assign each writer explicit files or a feature boundary. Serialize builds in a shared target directory. Verification begins after the implementation surface is complete; the verifier reports what actually ran and what remains unverified. The native role files contain the detailed role instructions.

## Start

Run one of these commands from the project root. The launcher resolves the project directory from its own location, so it also works when invoked by absolute path from another directory.

```text
node .agent-team/uar-core/launch.mjs codex
node .agent-team/uar-core/launch.mjs claude --agent uar-lead
node .agent-team/uar-core/launch.mjs kimi --agent uar-lead
node .agent-team/uar-core/launch.mjs opencode --agent uar-lead
node .agent-team/uar-core/launch.mjs minimax
```

Use this opening request, replacing the bracketed task:

> Use uar-lead to handle [task]. Restore the project instructions and active OpenSpec/KBD position, define acceptance evidence, and delegate only bounded independent work to the UAR specialists. Use three active agents by default and never more than four including the lead. Preserve existing changes, name each writer's ownership, and wait for independent verification before reporting completion.

The launcher passes remaining arguments to the native CLI without shell interpolation. It sets the child working directory to UAR, adds Codex `-C`, and defaults Kimi to `--model kimi-code/k3` unless an explicit `--model` or `-m` is supplied. It installs nothing, changes no global settings, and does not start other harnesses.

## Native selection and discovery

| Harness | Project definitions | Selection and inspection |
|---|---|---|
| Codex | `.codex/agents/uar-*.toml` | Start the normal session and request delegation to `uar-lead` by name. There is no `--agent` launcher flag. `/agent` and `codex agents` inspect live sessions. |
| Claude Code | `.claude/agents/uar-*.md` | `--agent uar-lead` selects the main role; interactive `/agents` discovers definitions. In installed 2.1.282, `claude agents` manages background sessions, not role validation. |
| Kimi Code | `.kimi-code/agents/uar-*.md` | `--agent uar-lead` or `--agent-file <path>` selects the main profile. An explicit invalid file errors; invalid directory-discovered files can be skipped with warnings. |
| OpenCode | `.opencode/agents/uar-*.md` | `--agent uar-lead` selects a primary role; delegate specialist subagents through native mechanisms. `opencode agent list` and `opencode debug agent <name>` inspect definitions. |
| MiniMax Code | `.minimax/agents/<role>/agent.md` with the isolated launcher | The CLI source has no `--agent` or `--agent-file` option. Launch the isolated native interface and inspect its available agent/delegation surface. Direct custom-role activation remains unverified. |

Do not interpret a role prompt or file-ownership instruction as an enforced permission boundary. Tool filtering, filesystem sandboxing, external MCP operations, and user approvals have different native semantics. See [harness research](harness-research.md) for the inspected contracts and [model policy](model-policy.md) for routing.

## MiniMax isolation and availability

The MiniMax launcher sets `MINIMAX_DATA_DIR` only in its child environment to this project's `.minimax` directory. MiniMax reads canonical agents relative to its active data directory; merely placing agent files in a project does not select them. Native authentication, settings, sessions, and databases also move into that directory. Do not copy global `~/.minimax` state into the project or commit generated credentials/session data. External user skill discovery may still apply according to native configuration, so this is data-directory isolation rather than a promise of total skill isolation.

At the research boundary, `mcode` was absent from PATH; existing `~/.minimax/bin/minimax` and `mavis` links targeted a missing desktop application. The role files and launcher can be installed, but MiniMax cannot be reported as runnable here. Installing a CLI and configuring its isolated account are separate actions; this launcher performs neither. Once a suitable CLI is available, pass ordinary native commands through this same launcher so they use the same isolated data directory.

## Evidence levels

1. **Files installed:** definitions and launcher exist in the project.
2. **Static validation:** syntax/schema checks accepted the files.
3. **Native discovery:** the installed harness actually loaded the intended definitions.
4. **Inference:** an authorized task successfully ran with the intended role and model.

These levels are distinct. Model catalogs do not prove account access. Research verified versions/help and model discovery for Codex, Kimi, and OpenCode; bounded Codex doctor and OpenCode agent-list probes timed out. MiniMax had no usable executable. No paid model smoke tests were performed in the research task. Follow the installation receipt for any later validation rather than upgrading these claims from file presence alone.

The launcher itself can be checked without starting a harness:

```text
node --check .agent-team/uar-core/launch.mjs
node .agent-team/uar-core/launch.mjs --help
```

At the launcher completion boundary, `node --check` and launcher help passed. Invoking the launcher by absolute path from `/tmp` with each native `--help` returned exit 0 for Codex, Claude, Kimi, and OpenCode (OpenCode writes help to stderr). MiniMax returned exit 1 with the explicit missing-`mcode` message. These checks exercised executable dispatch without starting model sessions; they do not certify role discovery or inference.

## Coordination and regeneration

The interactive parent should adopt uar-lead instructions and dispatch specialists directly. If a harness permits nested delegation, a delegated lead can coordinate; otherwise keep coordination in the parent rather than bypassing native depth limits. These are reusable native agents, not an always-running team or a separate scheduler. Claude experimental team mode is not required.

team.json is canonical. team-request.json and state.json are initialized copies; use the project-local creator team-update command to update durable state deliberately, then export to fresh directories and review native diffs before installation. Existing exports are immutable receipts, not automatically synchronized. CLI: node .agents/skills/agent-team-creator/scripts/cli.mjs. Its project-local references describe task and handoff JSON requests.

Final native discovery: OpenCode loaded all seven installed roles (lead primary, six subagents); see opencode-native-discovery.json. Its piped agent-list output was truncated; direct file output established discovery. Codex doctor loaded configuration but returned unrelated environment/network/state warnings and supplied no custom-role listing. Claude/Kimi role activation and all live role inference remain unverified.
