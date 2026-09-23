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
            restore_checkpoint: request.checkpoint_resume.is_some(),
            has_memory_hits: !request.memory_hits.is_empty(),
            requested_skill_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::llm::{Message, MessageContent, MessageRole, ToolCall, ToolCallFunction};
    use crate::uar::runtime::graph::GraphState;
    use crate::uar::runtime::turn::{CheckpointResume, RunExecutionRequest, TurnAssemblyPlan};

    #[test]
    fn checkpoint_resume_is_atomic_and_inherited_history_is_not_a_checkpoint() {
        let artifact = crate::uar::defaults::default_agent();
        let mut request = RunExecutionRequest::new(artifact.clone(), "next".to_string());
        let inherited_history = vec![
            Message {
                role: MessageRole::Assistant,
                content: MessageContent::text(""),
                tool_call_id: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call-1".to_string(),
                    call_type: "function".to_string(),
                    function: ToolCallFunction {
                        name: "lookup".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
            },
            Message {
                role: MessageRole::Tool,
                content: MessageContent::text("complete result"),
                tool_call_id: Some("call-1".to_string()),
                tool_calls: None,
            },
        ];
        request.inherited_history = Some(inherited_history.clone());
        assert!(!TurnAssemblyPlan::for_request(&request).restore_checkpoint);
        let preserved = request
            .inherited_history
            .as_ref()
            .expect("inherited child history remains present");
        assert_eq!(preserved.len(), 2);
        assert_eq!(preserved[0].tool_calls.as_ref().unwrap()[0].id, "call-1");
        assert_eq!(preserved[1].tool_call_id.as_deref(), Some("call-1"));

        request.inherited_history = None;
        request.checkpoint_resume = Some(CheckpointResume {
            state: GraphState::default(),
            history: Vec::new(),
            authorization_digest: "0".repeat(64),
        });
        assert!(TurnAssemblyPlan::for_request(&request).restore_checkpoint);
    }
}
