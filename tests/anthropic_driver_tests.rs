//! Integration tests for the Anthropic native driver and tool normalization.

#[cfg(test)]
mod driver_dispatch {
    use universal_agent_runtime::llm::{anthropic_native_driver_enabled, detect_provider};

    /// Test 10.8: Verify provider detection correctly identifies Anthropic models.
    #[test]
    fn detect_anthropic_provider() {
        assert_eq!(
            detect_provider("anthropic/claude-sonnet-4-20250514"),
            "anthropic"
        );
        assert_eq!(detect_provider("anthropic/claude-opus-4"), "anthropic");
        assert_eq!(detect_provider("openai/gpt-4o"), "openai");
        assert_eq!(detect_provider("groq/llama-3-70b"), "groq");
        assert_eq!(detect_provider("unknown"), "unknown");
    }

    // Both assertions live in one test because `ANTHROPIC_NATIVE_DRIVER` is
    // process-global mutable state; splitting them into separate #[test]
    // functions races under the default parallel test runner.
    #[test]
    fn anthropic_native_driver_env_var_controls_enablement() {
        unsafe { std::env::remove_var("ANTHROPIC_NATIVE_DRIVER") };
        assert!(anthropic_native_driver_enabled());

        unsafe { std::env::set_var("ANTHROPIC_NATIVE_DRIVER", "false") };
        assert!(!anthropic_native_driver_enabled());

        unsafe { std::env::remove_var("ANTHROPIC_NATIVE_DRIVER") };
        assert!(anthropic_native_driver_enabled());
    }
}

#[cfg(test)]
mod capability_registry {
    use universal_agent_runtime::llm::capability_registry::{
        ModelCapabilityRegistry, ToolCallCapability,
    };

    #[test]
    fn anthropic_models_are_native() {
        let registry = ModelCapabilityRegistry::new();
        let profile = registry.get("anthropic/claude-sonnet-4-20250514");
        assert_eq!(profile.tool_call_capability, ToolCallCapability::Native);
        assert!(profile.supports_parallel_tool_calls);
    }

    #[test]
    fn openai_models_are_native() {
        let registry = ModelCapabilityRegistry::new();
        let profile = registry.get("openai/gpt-4o");
        assert_eq!(profile.tool_call_capability, ToolCallCapability::Native);
    }

    #[test]
    fn qwen_models_are_instruction_tuned() {
        let registry = ModelCapabilityRegistry::new();
        let profile = registry.get("qwen2.5-72b-instruct");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );
    }

    #[test]
    fn unknown_models_default_to_instruction_tuned() {
        let registry = ModelCapabilityRegistry::new();
        let profile = registry.get("totally-unknown-model/v1");
        assert_eq!(
            profile.tool_call_capability,
            ToolCallCapability::InstructionTuned
        );
    }
}

#[cfg(test)]
mod tool_extraction {
    use universal_agent_runtime::llm::tool_extractor::ToolCallExtractor;
    use universal_agent_runtime::normalized::NormalizedEvent;

    #[test]
    fn extracts_tool_call_from_text_stream() {
        let mut extractor = ToolCallExtractor::new();

        // Simulate streaming text with an embedded tool call
        let events1 = extractor.process_delta("I'll search for that. ");
        assert!(
            events1
                .iter()
                .any(|e| matches!(e, NormalizedEvent::MessageDelta { .. }))
        );

        let events2 = extractor.process_delta(
            "<tool_call>\n{\"name\": \"search\", \"input\": {\"query\": \"test\"}}\n</tool_call>",
        );
        assert!(
            events2
                .iter()
                .any(|e| matches!(e, NormalizedEvent::ToolCallComplete { .. }))
        );
    }

    #[test]
    fn plain_text_passes_through() {
        let mut extractor = ToolCallExtractor::new();
        let events = extractor.process_delta("Hello, this is just text with no tool calls.");
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], NormalizedEvent::MessageDelta { text } if text == "Hello, this is just text with no tool calls.")
        );
    }
}

#[cfg(test)]
mod xml_injection {
    use universal_agent_runtime::llm::xml_tool_injector::tools_to_xml;

    #[test]
    fn generates_xml_from_openai_tools() {
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "search",
                "description": "Search the web",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"}
                    },
                    "required": ["query"]
                }
            }
        })];

        let xml = tools_to_xml(&tools);
        assert!(xml.contains("<tool_definitions>"));
        assert!(xml.contains("name=\"search\""));
        assert!(xml.contains("Search the web"));
        assert!(xml.contains("<tool_call>"));
    }
}

#[cfg(test)]
mod anthropic_types {
    use universal_agent_runtime::llm::anthropic_types::*;

    #[test]
    fn cache_control_serializes_correctly() {
        let cc = CacheControl::ephemeral();
        let json = serde_json::to_string(&cc).unwrap();
        assert!(json.contains("ephemeral"));
    }

    #[test]
    fn thinking_config_serializes() {
        let tc = ThinkingConfig {
            thinking_type: "enabled".to_string(),
            budget_tokens: 10000,
        };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("10000"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn messages_request_serializes() {
        let req = MessagesRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            system: None,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: serde_json::json!("Hello"),
            }],
            tools: None,
            tool_choice: None,
            stream: true,
            thinking: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("claude-sonnet"));
        assert!(json.contains("\"stream\":true"));
    }
}
