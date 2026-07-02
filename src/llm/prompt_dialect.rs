//! Per-model prompt dialect engine (uar-next-fable §2.1).
//!
//! Different model families have documented, web-verified preferences for how
//! prompts and reasoning are expressed at the API layer. Treating every model
//! as an interchangeable black box leaves capability on the table. This module
//! detects a model's dialect from its id and produces the extra request
//! parameters that dialect wants — reasoning-persistence toggles, structured
//! output hints, thinking-effort levels — as a JSON object the driver merges
//! into the outbound request body.
//!
//! Only web-verified parameters are encoded here (see the fable doc's §2.3:
//! encode NO numbers from the model-comparison document). Values that vary by
//! provider deployment (e.g. Qwen's DashScope `extra_body` vs vLLM
//! `chat_template_kwargs` split) are surfaced as flags for the driver, not
//! hardcoded, since the correct wrapper depends on the endpoint.

use serde_json::{Value, json};

/// The prompt dialect a model family prefers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDialect {
    /// Anthropic Claude: XML structure, extended-thinking budgets.
    AnthropicXml,
    /// OpenAI GPT: Responses-API `text.format` structured output, avoid CoT prose.
    OpenAiJson,
    /// Moonshot Kimi: preserved thinking (`thinking.keep`), Markdown headings.
    KimiMarkdown,
    /// Zhipu/Z.ai GLM: `thinking_mode` high/max.
    GlmThinking,
    /// Alibaba Qwen: `enable_thinking` / `preserve_thinking` (endpoint-dependent wrapper).
    QwenHybrid,
    /// MiniMax M-series: Markdown-averse — prefer XML/JSON structure.
    MiniMaxStructured,
    /// Fallback for unknown families.
    Generic,
}

impl PromptDialect {
    /// Detect the dialect from a `provider/model` or bare model id.
    #[must_use]
    pub fn detect(model_id: &str) -> Self {
        let m = model_id.to_ascii_lowercase();
        if m.contains("claude") || m.contains("anthropic") {
            Self::AnthropicXml
        } else if m.contains("gpt") || m.contains("openai") || m.contains("o1") || m.contains("o3")
        {
            Self::OpenAiJson
        } else if m.contains("kimi") || m.contains("moonshot") {
            Self::KimiMarkdown
        } else if m.contains("glm") || m.contains("zhipu") || m.contains("z.ai") {
            Self::GlmThinking
        } else if m.contains("qwen") || m.contains("alibaba") || m.contains("dashscope") {
            Self::QwenHybrid
        } else if m.contains("minimax") {
            Self::MiniMaxStructured
        } else {
            Self::Generic
        }
    }

    /// Whether this dialect prefers XML-structured prompt envelopes
    /// (`<context>`, `<instructions>`) over plain text.
    #[must_use]
    pub fn prefers_xml_envelope(self) -> bool {
        matches!(self, Self::AnthropicXml | Self::MiniMaxStructured)
    }

    /// Whether Markdown structure degrades this model's output
    /// (MiniMax's documented Markdown-aversion).
    #[must_use]
    pub fn markdown_averse(self) -> bool {
        matches!(self, Self::MiniMaxStructured)
    }
}

/// Options controlling dialect parameter generation for one request.
#[derive(Debug, Clone, Copy, Default)]
pub struct DialectRequest {
    /// The task benefits from reasoning/extended thinking.
    pub wants_reasoning: bool,
    /// This is a multi-turn conversation (reasoning persistence matters).
    pub multi_turn: bool,
    /// Hard problem — request the highest thinking effort where supported.
    pub hard: bool,
}

/// The prompt dialect engine: detect dialect, emit per-model request params.
#[derive(Debug, Clone, Copy, Default)]
pub struct PromptDialectEngine;

impl PromptDialectEngine {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Detect the dialect for a model id.
    #[must_use]
    pub fn dialect_for(&self, model_id: &str) -> PromptDialect {
        PromptDialect::detect(model_id)
    }

    /// Produce the extra request-body parameters this model's dialect wants,
    /// as a JSON object to merge into the outbound completion request. Returns
    /// an empty object when the dialect needs no extra params for this request.
    ///
    /// All parameter names/shapes are web-verified (fable §2.1):
    /// - Anthropic: `thinking: {type: "enabled", budget_tokens}` for reasoning.
    /// - OpenAI: no reasoning body param here (Responses `text.format` is a
    ///   structured-output concern set by the caller's schema, not reasoning).
    /// - Kimi: `thinking: {type: "enabled", keep: "all"}` to preserve reasoning
    ///   across turns; on k2.7-code this is always-on and harmless to send.
    /// - GLM: `thinking: {type: "enabled"}` + `reasoning_effort: "max"|"high"`.
    /// - Qwen: `enable_thinking: true` (+ `preserve_thinking` for multi-turn);
    ///   the DashScope-vs-vLLM wrapper is the driver's concern.
    #[must_use]
    pub fn request_params(&self, model_id: &str, req: DialectRequest) -> Value {
        let dialect = PromptDialect::detect(model_id);
        match dialect {
            PromptDialect::AnthropicXml if req.wants_reasoning => json!({
                "thinking": { "type": "enabled", "budget_tokens": if req.hard { 8192 } else { 2048 } }
            }),
            PromptDialect::KimiMarkdown if req.wants_reasoning || req.multi_turn => json!({
                "thinking": { "type": "enabled", "keep": "all" }
            }),
            PromptDialect::GlmThinking if req.wants_reasoning => json!({
                "thinking": { "type": "enabled" },
                "reasoning_effort": if req.hard { "max" } else { "high" }
            }),
            PromptDialect::QwenHybrid if req.wants_reasoning => {
                if req.multi_turn {
                    json!({ "enable_thinking": true, "preserve_thinking": true })
                } else {
                    json!({ "enable_thinking": true })
                }
            }
            _ => json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_families() {
        assert_eq!(
            PromptDialect::detect("anthropic/claude-sonnet-5"),
            PromptDialect::AnthropicXml
        );
        assert_eq!(
            PromptDialect::detect("openai/gpt-5.5"),
            PromptDialect::OpenAiJson
        );
        assert_eq!(
            PromptDialect::detect("moonshot/kimi-k2.7-code"),
            PromptDialect::KimiMarkdown
        );
        assert_eq!(
            PromptDialect::detect("z.ai/glm-5.2"),
            PromptDialect::GlmThinking
        );
        assert_eq!(
            PromptDialect::detect("alibaba/qwen3.7-max"),
            PromptDialect::QwenHybrid
        );
        assert_eq!(
            PromptDialect::detect("minimax/m3"),
            PromptDialect::MiniMaxStructured
        );
        assert_eq!(
            PromptDialect::detect("meta/llama-4"),
            PromptDialect::Generic
        );
    }

    #[test]
    fn anthropic_reasoning_emits_thinking_budget() {
        let e = PromptDialectEngine::new();
        let p = e.request_params(
            "anthropic/claude-opus-4-8",
            DialectRequest {
                wants_reasoning: true,
                hard: true,
                ..Default::default()
            },
        );
        assert_eq!(p["thinking"]["type"], "enabled");
        assert_eq!(p["thinking"]["budget_tokens"], 8192);
    }

    #[test]
    fn kimi_multi_turn_preserves_thinking() {
        let e = PromptDialectEngine::new();
        let p = e.request_params(
            "moonshot/kimi-k2.6",
            DialectRequest {
                multi_turn: true,
                ..Default::default()
            },
        );
        assert_eq!(p["thinking"]["keep"], "all");
    }

    #[test]
    fn qwen_multi_turn_adds_preserve_thinking() {
        let e = PromptDialectEngine::new();
        let p = e.request_params(
            "alibaba/qwen3.7-max",
            DialectRequest {
                wants_reasoning: true,
                multi_turn: true,
                ..Default::default()
            },
        );
        assert_eq!(p["enable_thinking"], true);
        assert_eq!(p["preserve_thinking"], true);
    }

    #[test]
    fn no_params_when_reasoning_not_wanted() {
        let e = PromptDialectEngine::new();
        let p = e.request_params("openai/gpt-5.5", DialectRequest::default());
        assert_eq!(p, serde_json::json!({}));
    }

    #[test]
    fn minimax_is_markdown_averse() {
        assert!(PromptDialect::MiniMaxStructured.markdown_averse());
        assert!(!PromptDialect::AnthropicXml.markdown_averse());
        assert!(PromptDialect::AnthropicXml.prefers_xml_envelope());
    }
}
