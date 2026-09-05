//! Per-request attribution to the skills active when that request began.

use crate::normalized::NormalizedEvent;

/// Flushes once on stream completion, failure, or cancellation. Repeated usage
/// reports within one request replace the previous cumulative report.
#[derive(Debug)]
pub struct SkillRequestUsage {
    pub model: String,
    pub skills: Vec<String>,
    cost_tracking: bool,
    usage: Option<(u64, u64, u64)>,
}

impl SkillRequestUsage {
    pub fn new(model: String, skills: Vec<String>, cost_tracking: bool) -> Self {
        Self {
            model,
            skills,
            cost_tracking,
            usage: None,
        }
    }

    pub fn observe(&mut self, event: &NormalizedEvent) {
        if let NormalizedEvent::Usage {
            prompt_tokens,
            completion_tokens,
            cached_tokens,
            ..
        } = event
        {
            self.usage = Some((
                u64::from(*prompt_tokens),
                u64::from(*completion_tokens),
                u64::from(cached_tokens.unwrap_or(0)),
            ));
        }
    }
}

impl Drop for SkillRequestUsage {
    fn drop(&mut self) {
        let Some((input, output, cached)) = self.usage else {
            return;
        };
        let cost = self
            .cost_tracking
            .then(|| crate::llm::catalog::estimate_cost(&self.model, input, output, cached))
            .flatten();
        for skill_id in &self.skills {
            crate::uar::telemetry::metrics::record_skill_request_usage(
                skill_id,
                input.saturating_add(output),
                cost,
            );
        }
    }
}

/// Graph nodes make driver calls directly. This adapter applies the same
/// request-snapshot attribution and bounded body attachment to those calls.
pub struct SkillRequestDriver {
    pub inner: std::sync::Arc<dyn crate::llm::LlmDriver>,
    pub context: std::sync::Arc<tokio::sync::Mutex<super::activation::ActivationContext>>,
    pub model: String,
    pub context_limit: usize,
    pub budget: crate::config::SkillReattachmentBudget,
    pub cost_tracking: bool,
}

impl std::fmt::Debug for SkillRequestDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillRequestDriver")
            .field("model", &self.model)
            .field("context_limit", &self.context_limit)
            .finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl crate::llm::LlmDriver for SkillRequestDriver {
    async fn stream(
        &self,
        mut request: crate::llm::LlmRequest,
    ) -> anyhow::Result<crate::llm::ExternalDriverStream> {
        use futures::StreamExt;
        let active = self.context.lock().await.active();
        let mut usage = SkillRequestUsage::new(
            self.model.clone(),
            active
                .iter()
                .map(|entry| entry.skill.skill_id.clone())
                .collect(),
            self.cost_tracking,
        );
        let history = serde_json::from_value::<Vec<crate::llm::Message>>(
            serde_json::Value::Array(request.messages),
        )?;
        let dialect = crate::llm::prompt_dialect::PromptDialect::detect(&self.model);
        let (attached, _) = super::retention::reattach_skills(
            &history,
            &active,
            &self.model,
            self.context_limit,
            self.budget,
            crate::uar::runtime::prompt::RenderOptions {
                prefers_xml_envelope: dialect.prefers_xml_envelope(),
                markdown_averse: dialect.markdown_averse(),
            },
        );
        request.messages = attached
            .iter()
            .map(|message| serde_json::json!(message))
            .collect();
        let mut stream = self.inner.stream(request).await?;
        Ok(Box::pin(async_stream::stream! {
            while let Some(event) = stream.next().await {
                if let Ok(event) = &event {
                    usage.observe(event);
                }
                yield event;
            }
        }))
    }
}
