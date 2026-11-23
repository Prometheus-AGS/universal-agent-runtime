//! Supervised sidecar binary for the Universal Agent Runtime.
//!
//! The entry point lives in the library as
//! [`universal_agent_runtime::sidecar::run`], so the standalone binary can
//! offer the same mode through `universal-agent-runtime --supervised`. This
//! binary exists for hosts that package the sidecar on its own.

use universal_agent_runtime::sidecar::{ImplicitEnvFile, run};

fn main() -> std::process::ExitCode {
    // This binary has always loaded the nearest `.env`; hosts that ship it
    // may rely on that, so it is kept here and not offered by `--supervised`.
    run(ImplicitEnvFile::Load)
}
