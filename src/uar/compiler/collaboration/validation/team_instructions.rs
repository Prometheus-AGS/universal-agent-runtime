//! Exact immutable shared-guidance validation, including UTF-8 byte limits.

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::uar::domain::team_context::{
    MAX_SAFE_COUNTER, MAX_TEAM_INSTRUCTION_BYTES, TeamInstructions,
};

use super::canonical::digest;

pub(in crate::uar::compiler::collaboration) fn resolve_team_instructions(
    document: &Value,
) -> Result<Option<TeamInstructions>> {
    let Some(value) = document.get("instructions") else {
        return Ok(None);
    };
    let instructions: TeamInstructions = serde_json::from_value(value.clone())
        .context("TEAM_CONTEXT_REQUIRED_UNSUPPORTED /instructions: expected revision, digest and text")?;
    if instructions.revision == 0 || instructions.revision > MAX_SAFE_COUNTER {
        bail!("TEAM_CONTEXT_REQUIRED_UNSUPPORTED /instructions/revision: outside revision range");
    }
    if instructions.text.len() > MAX_TEAM_INSTRUCTION_BYTES {
        bail!("TEAM_CONTEXT_REQUIRED_TOO_LARGE /instructions/text: exceeds 16384 UTF-8 bytes");
    }
    if instructions.digest != digest(instructions.text.as_bytes()) {
        bail!("TEAM_CONTEXT_REQUIRED_UNSUPPORTED /instructions/digest: exact UTF-8 text digest mismatch");
    }
    Ok(Some(instructions))
}
