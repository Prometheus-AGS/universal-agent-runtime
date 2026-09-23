//! Contract tests for the OpenSpec change `sidecar-launch-security`
//! (tasks.md section 2). Every test enters through a real binary: the
//! `uar-sidecar` process, the standalone `universal-agent-runtime` process, or
//! the `uar-env-probe` child those processes spawn.

#[path = "support/sidecar_process.rs"]
mod sidecar_process;
#[path = "integration/live/stub_llm.rs"]
mod stub_llm;

#[path = "sidecar_launch_security/capabilities.rs"]
mod capabilities;
#[path = "sidecar_launch_security/confinement.rs"]
mod confinement;
#[path = "sidecar_launch_security/features.rs"]
mod features;
#[path = "sidecar_launch_security/governance.rs"]
mod governance;
#[path = "sidecar_launch_security/guard.rs"]
mod guard;
#[path = "sidecar_launch_security/surface.rs"]
mod surface;
#[path = "sidecar_launch_security/token.rs"]
mod token;

use std::path::{Path, PathBuf};

use sidecar_process::{
    ConfigOptions, LaunchOptions, STUB_MODEL, ServerProcess, Workspace, launch_sidecar,
    probe_binary, random_token, render_config,
};
use stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint, StubLlmServer, start_stub_llm};

/// Add a two-step stub conversation for `input`: the model calls
/// `terminal_exec` with `command`, then answers `done` after the tool result.
pub(crate) fn with_terminal_call(set: FixtureSet, input: &str, command: &str) -> FixtureSet {
    let fingerprint = |has_tool_result| RequestFingerprint {
        model: STUB_MODEL.to_owned(),
        last_user_message: input.to_owned(),
        has_tools: true,
        has_tool_result,
    };
    set.with(
        fingerprint(false),
        FixtureResponse::ToolCall {
            name: "terminal_exec".to_owned(),
            arguments: serde_json::json!({ "command": command }).to_string(),
        },
    )
    .with(
        fingerprint(true),
        FixtureResponse::Content("done".to_owned()),
    )
}

/// Add a one-step stub conversation for `input` that answers `reply`.
pub(crate) fn with_reply(set: FixtureSet, input: &str, reply: &str) -> FixtureSet {
    set.with(
        RequestFingerprint {
            model: STUB_MODEL.to_owned(),
            last_user_message: input.to_owned(),
            has_tools: true,
            has_tool_result: false,
        },
        FixtureResponse::Content(reply.to_owned()),
    )
}

/// A shell command that runs the environment probe, appending to `output`.
pub(crate) fn probe_command(output: &Path) -> String {
    format!("'{}' '{}'", probe_binary(), output.display())
}

/// A running sidecar with the workspace, token and stub LLM it depends on.
pub(crate) struct Sidecar {
    pub process: ServerProcess,
    pub token: String,
    pub workspace: Workspace,
    /// Kept alive for the sidecar's lifetime; dropping it stops the stub.
    pub _stub: StubLlmServer,
}

impl Sidecar {
    pub fn port(&self) -> u16 {
        self.process.port()
    }

    pub fn host(&self) -> String {
        host(self.port())
    }

    pub fn bearer(&self) -> String {
        bearer(&self.token)
    }

    pub fn base_url(&self) -> String {
        self.process.base_url()
    }
}

pub(crate) fn host(port: u16) -> String {
    format!("127.0.0.1:{port}")
}

pub(crate) fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

/// Write the standard sidecar configuration with `extra_yaml` appended.
pub(crate) fn write_config(workspace: &Workspace, stub_url: &str, extra_yaml: &str) -> PathBuf {
    let mut options = ConfigOptions::new(stub_url);
    options.extra_yaml = extra_yaml.to_owned();
    workspace.write_config("config.yaml", &render_config(workspace, &options))
}

/// Boot a sidecar with a fresh token. `customize` may add environment or
/// arguments before launch.
pub(crate) async fn boot_sidecar(
    fixtures: FixtureSet,
    extra_yaml: &str,
    customize: impl FnOnce(LaunchOptions) -> LaunchOptions,
) -> Sidecar {
    let stub = start_stub_llm(fixtures).await;
    let workspace = Workspace::new();
    let config = write_config(&workspace, &stub.base_url, extra_yaml);
    let token = random_token();
    let options = customize(LaunchOptions::new(config, "sidecar").with_token(&token));
    let process = launch_sidecar(&workspace, &options).await;
    Sidecar {
        process,
        token,
        workspace,
        _stub: stub,
    }
}
