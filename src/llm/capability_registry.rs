//! Model capability registry for determining tool call strategy per model.
//!
//! Different LLM providers and models support tool calling with varying degrees
//! of native support. This registry maps model identifiers (using glob-style
//! patterns) to capability profiles that inform the orchestrator how to handle
//! tool definitions and tool call extraction.

/// How a model handles tool calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCallCapability {
    /// Model natively supports function/tool calling (OpenAI, Anthropic, Gemini, Mistral Large).
    Native,
    /// Model uses grammar-constrained decoding (e.g. candle-vllm with constrained output).
    GrammarConstrained,
    /// Model follows instructions to emit structured XML tool calls (Qwen, DeepSeek, Llama).
    InstructionTuned,
    /// Pure text model with no structured output; aggressive parsing needed.
    TextOnly,
}

/// Capability profile describing what a model supports.
#[derive(Debug, Clone)]
pub struct ModelCapabilityProfile {
    /// How the model handles tool calls.
    pub tool_call_capability: ToolCallCapability,
    /// Whether the model supports a dedicated system prompt role.
    pub supports_system_prompt: bool,
    /// Whether the model supports streaming responses.
    pub supports_streaming: bool,
    /// Maximum context window size in tokens.
    pub max_context_tokens: u32,
    /// Whether the model can invoke multiple tools in a single turn.
    pub supports_parallel_tool_calls: bool,
}

impl Default for ModelCapabilityProfile {
    fn default() -> Self {
        Self {
            tool_call_capability: ToolCallCapability::InstructionTuned,
            supports_system_prompt: true,
            supports_streaming: true,
            max_context_tokens: 32_000,
            supports_parallel_tool_calls: false,
        }
    }
}

/// Registry that maps model identifier patterns to capability profiles.
///
/// Patterns are matched in order, so more specific patterns should be
/// registered before broader ones. The built-in [`Self::new`] constructor
/// pre-populates well-known provider patterns.
#[derive(Debug)]
pub struct ModelCapabilityRegistry {
    /// (glob pattern, profile) pairs checked in order.
    profiles: Vec<(String, ModelCapabilityProfile)>,
}

impl ModelCapabilityRegistry {
    /// Create a new registry pre-populated with known model profiles.
    #[must_use]
    pub fn new() -> Self {
        let profiles = vec![
            (
                "anthropic/*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::Native,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 200_000,
                    supports_parallel_tool_calls: true,
                },
            ),
            (
                "openai/*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::Native,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: true,
                },
            ),
            (
                "groq/*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::Native,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: false,
                },
            ),
            (
                "mistral/*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::Native,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: false,
                },
            ),
            (
                "google/*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::Native,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 1_000_000,
                    supports_parallel_tool_calls: true,
                },
            ),
            (
                "qwen*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::InstructionTuned,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: false,
                },
            ),
            (
                "llama*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::InstructionTuned,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: false,
                },
            ),
            (
                "deepseek*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::InstructionTuned,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: false,
                },
            ),
            (
                "phi*".to_string(),
                ModelCapabilityProfile {
                    tool_call_capability: ToolCallCapability::InstructionTuned,
                    supports_system_prompt: true,
                    supports_streaming: true,
                    max_context_tokens: 128_000,
                    supports_parallel_tool_calls: false,
                },
            ),
        ];

        Self { profiles }
    }

    /// Look up the capability profile for the given model identifier.
    ///
    /// Patterns are matched by checking whether the model string starts with
    /// the pattern prefix (everything before a trailing `*`). The first match
    /// wins. If no pattern matches, a sensible default is returned.
    #[must_use]
    pub fn get(&self, model: &str) -> ModelCapabilityProfile {
        for (pattern, profile) in &self.profiles {
            if pattern_matches(pattern, model) {
                return profile.clone();
            }
        }
        ModelCapabilityProfile::default()
    }

    /// Register a custom pattern and profile.
    ///
    /// The new entry is prepended so it takes priority over existing patterns.
    pub fn register(&mut self, pattern: String, profile: ModelCapabilityProfile) {
        self.profiles.insert(0, (pattern, profile));
    }
}

impl Default for ModelCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple glob matching: if the pattern ends with `*`, the model must start
/// with the prefix before the `*`. If the pattern ends with `/*`, the model
/// must start with everything up to and including the `/`. Otherwise, an
/// exact match is required.
fn pattern_matches(pattern: &str, model: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        model.starts_with(prefix)
    } else {
        model == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_providers() {
        let registry = ModelCapabilityRegistry::new();

        let profile = registry.get("openai/gpt-4o");
        assert_eq!(profile.tool_call_capability, ToolCallCapability::Native);
        assert!(profile.supports_parallel_tool_calls);
        assert_eq!(profile.max_context_tokens, 128_000);

        let profile = registry.get("anthropic/claude-3-opus");
        assert_eq!(profile.tool_call_capability, ToolCallCapability::Native);
        assert!(profile.supports_parallel_tool_calls);
        assert_eq!(profile.max_context_tokens, 200_000);

        let profile = registry.get("google/gemini-pro");
        assert_eq!(profile.tool_call_capability, ToolCallCapability::Native);
        assert_eq!(profile.max_context_tokens, 1_000_000);
    }

    #[test]
    fn test_instruction_tuned_models() {
        let registry = ModelCapabilityRegistry::new();

        let profile = registry.get("qwen-72b-chat");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );

        let profile = registry.get("llama-3.1-70b");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );

        let profile = registry.get("deepseek-coder-v2");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );

        let profile = registry.get("phi-3-mini");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );
    }

    #[test]
    fn test_unknown_model_gets_default() {
        let registry = ModelCapabilityRegistry::new();
        let profile = registry.get("some-random-model");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );
        assert_eq!(profile.max_context_tokens, 32_000);
    }

    #[test]
    fn test_custom_registration_takes_priority() {
        let mut registry = ModelCapabilityRegistry::new();
        registry.register(
            "openai/gpt-4o-custom".to_string(),
            ModelCapabilityProfile {
                max_context_tokens: 256_000,
                ..ModelCapabilityProfile::default()
            },
        );
        let profile = registry.get("openai/gpt-4o-custom");
        assert_eq!(profile.max_context_tokens, 256_000);
    }

    #[test]
    fn test_pattern_matching() {
        assert!(pattern_matches("openai/*", "openai/gpt-4o"));
        assert!(pattern_matches("openai/*", "openai/gpt-3.5-turbo"));
        assert!(!pattern_matches("openai/*", "anthropic/claude"));
        assert!(pattern_matches("qwen*", "qwen-72b-chat"));
        assert!(pattern_matches("exact-match", "exact-match"));
        assert!(!pattern_matches("exact-match", "exact-match-not"));
    }
}
