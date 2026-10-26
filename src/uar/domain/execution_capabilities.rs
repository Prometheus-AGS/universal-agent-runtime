//! Transport-free workflow and team execution capability policy.
//! Shared by runtime/compiler admission and server capability discovery.

// C10 customer feedback requires the existing classify/draft interpreter in the normal profile.
// Availability still requires team execution ownership; publication requires separate approval.
pub const WORKFLOW_EXECUTION_QUALIFIED: bool = true;
pub fn workflow_execution_enabled() -> bool {
    WORKFLOW_EXECUTION_QUALIFIED
        || std::env::var("UAR_WORKFLOW_EXECUTION_PROFILE_STAGE").as_deref() == Ok("operation")
}

pub const TEAM_EXECUTION_PROFILE: &str = "urn:prometheus:uar:team-execution:0.1.0";
pub const TEAM_EXECUTION_B_CAPABILITIES: [&str; 3] = [
    "team_execution_peer_tools_v1",
    "team_execution_shared_instructions_v1",
    "team_execution_continuations_v1",
];
/// Operated-source Gate B receipt in agent-fabric-convergence, committed at ad2ad1617.
pub const TEAM_EXECUTION_B_QUALIFIED: bool = true;
pub const TEAM_EXECUTION_B_QUALIFICATION_RECEIPT: Option<&str> =
    Some("sha256:5a96786980812effbd79661290e51dee2ec1cde524854566d588eab934d1a058");
pub fn team_execution_profile_stage() -> &'static str {
    match std::env::var("UAR_TEAM_EXECUTION_PROFILE_STAGE").as_deref() {
        Ok("operation") => "operation",
        _ if TEAM_EXECUTION_B_QUALIFIED => "qualified",
        _ => "unqualified",
    }
}
pub fn team_execution_b_enabled() -> bool {
    team_execution_profile_stage() != "unqualified"
}

