//! LLM-based intent classifier.
//!
//! Sends a structured prompt to the LLM listing available skills,
//! and asks it to rank which skills best match the user query.
//! Falls back to TF-IDF if the LLM call fails.

use super::tfidf::TfIdfClassifier;
use super::{ClassificationResult, IntentClassifier, IntentScore};
use crate::config::LlmConfig;
use crate::llm::{Message, MessageContent, MessageRole, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::uar::domain::skills::Skill;
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;
use futures::StreamExt;
use serde::Deserialize;
use std::sync::Arc;

/// LLM-backed intent classifier.
#[derive(Debug)]
pub struct LlmClassifier {
    llm_config: LlmConfig,
    topk: usize,
    /// Fallback classifier when LLM is unavailable
    fallback: TfIdfClassifier,
}

/// Expected JSON response from the LLM.
#[derive(Deserialize)]
struct LlmRanking {
    matches: Vec<LlmMatch>,
}

#[derive(Deserialize)]
struct LlmMatch {
    skill_id: String,
    score: f32,
}

impl LlmClassifier {
    /// Creates a new LLM classifier with the given settings.
    pub fn new(llm_config: LlmConfig, topk: usize) -> Self {
        Self {
            llm_config,
            topk,
            fallback: TfIdfClassifier::new(topk),
        }
    }

    /// Build the classification prompt.
    fn build_prompt(query: &str, skills: &[Skill]) -> String {
        let mut prompt = String::from(
            "You are a skill classifier. Given a user query and a list of available skills, \
             rank which skills best match the query.\n\n\
             Return ONLY a JSON object with this exact format:\n\
             {\"matches\": [{\"skill_id\": \"...\", \"score\": 0.0}]}\n\n\
             Score from 0.0 (no match) to 1.0 (perfect match). \
             Only include skills with score > 0.1. Order by score descending.\n\n\
             Available skills:\n",
        );

        for skill in skills {
            prompt.push_str(&format!(
                "- ID: {} | Title: {} | Description: {} | Keywords: {}\n",
                skill.skill_id,
                skill.title,
                skill.description,
                skill.triggers.keywords.join(", ")
            ));
        }

        prompt.push_str(&format!("\nUser query: \"{query}\"\n\nJSON response:"));
        prompt
    }
}

#[async_trait]
impl IntentClassifier for LlmClassifier {
    async fn classify(
        &self,
        query: &str,
        context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        let skills = registry.list_enabled();
        if skills.is_empty() {
            return Ok(ClassificationResult::new(Vec::new()));
        }

        let prompt = Self::build_prompt(query, &skills);

        // Create a minimal orchestrator (no tools needed for classification)
        let mcp = Arc::new(McpRegistry::new_empty());
        let native_skills = Arc::new(crate::uar::runtime::native_skill::NativeSkillRegistry::new());
        let orchestrator = Orchestrator::new(self.llm_config.clone(), mcp, native_skills)?;

        let messages = vec![Message {
            role: MessageRole::User,
            content: MessageContent::text(prompt),
            tool_call_id: None,
            tool_calls: None,
        }];

        // Collect the full response
        let mut response = String::new();
        match orchestrator.chat_with_history(messages).await {
            Ok(stream) => {
                futures::pin_mut!(stream);
                while let Some(event) = stream.next().await {
                    if let crate::normalized::NormalizedEvent::MessageDelta { text } = event {
                        response.push_str(&text);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("LLM classification failed, falling back to TF-IDF: {:?}", e);
                return self.fallback.classify(query, context, registry).await;
            }
        }

        // Parse JSON from response (handle markdown code fences)
        let json_str = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let ranking: LlmRanking = match serde_json::from_str(json_str) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(
                    "Failed to parse LLM classification response, falling back to TF-IDF: {:?}",
                    e
                );
                return self.fallback.classify(query, context, registry).await;
            }
        };

        // Map to IntentScores with skill references
        let skill_map: std::collections::HashMap<&str, &Skill> =
            skills.iter().map(|s| (s.skill_id.as_str(), s)).collect();

        let scores: Vec<IntentScore> = ranking
            .matches
            .into_iter()
            .filter(|m| m.score > 0.1)
            .filter_map(|m| {
                skill_map
                    .get(m.skill_id.as_str())
                    .map(|skill| IntentScore::with_skill(m.skill_id, m.score, (*skill).clone()))
            })
            .take(self.topk)
            .collect();

        Ok(ClassificationResult::new(scores))
    }

    async fn rebuild_index(&self, registry: &SkillRegistry) -> anyhow::Result<()> {
        // Also rebuild the TF-IDF fallback index
        self.fallback.rebuild_index(registry).await
    }
}
