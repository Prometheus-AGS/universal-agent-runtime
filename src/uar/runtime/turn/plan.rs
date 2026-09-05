use super::request::RunExecutionRequest;

/// Decisions available before session, policy, credential, or provider I/O.
#[derive(Debug, Clone)]
pub struct TurnAssemblyPlan {
    pub append_input: bool,
    pub restore_checkpoint: bool,
    pub has_memory_hits: bool,
    pub requested_skill_ids: Vec<String>,
}

impl TurnAssemblyPlan {
    pub fn for_request(request: &RunExecutionRequest) -> Self {
        let mut requested_skill_ids = Vec::new();
        for id in &request.skill_attachments {
            if !requested_skill_ids.contains(id) {
                requested_skill_ids.push(id.clone());
            }
        }
        Self {
            append_input: request.input.is_some(),
            restore_checkpoint: request.checkpoint_history.is_some(),
            has_memory_hits: !request.memory_hits.is_empty(),
            requested_skill_ids,
        }
    }
}
