use universal_agent_runtime::llm::{
    LlmRequest, Message, MessageContent, MessageRole, ToolCall, ToolCallFunction,
};

pub const TOOL_CALL_ID: &str = "protected-call-17";
pub const PAYLOAD_HEAD: &str = "receipt-head\r\n";
pub const PAYLOAD_MIDDLE: &str = "PROTECTED-MIDDLE-雪🙂é";
pub const PAYLOAD_TAIL: &str = "\r\nreceipt-tail";

/// Supply live capability metadata while the history fixture streams synthetic events.
pub async fn tool_capable_provider_registry() -> std::sync::Arc<universal_agent_runtime::llm::ProviderRegistry> {
    use universal_agent_runtime::llm::registry::{ModelConfig, ProtocolSetting, ProviderConfig};

    let registry = std::sync::Arc::new(universal_agent_runtime::llm::ProviderRegistry::new());
    registry.register(ProviderConfig {
        id: "openai".into(), display_name: "History fixture".into(),
        base_url: String::new(), api_key: Some("test-key".into()),
        protocol: ProtocolSetting::Chat, default_model: Some("gpt-4o".into()), enabled: true,
        models: vec![ModelConfig {
            id: "gpt-4o".into(), pricing_identity: None, execution_profile: None,
            display_name: None, context_window: None, supports_vision: false,
            supports_tools: true, supports_reasoning: false, supports_structured_output: false,
            supports_streaming: true, max_output_tokens: None, enabled: true,
        }],
    }).await.expect("register history fixture capable destination");
    registry
}

pub async fn verified_fixture_user(
    subject: &str,
) -> universal_agent_runtime::uar::security::claims::UserContext {
    use universal_agent_runtime::config::SecurityConfig;
    use universal_agent_runtime::uar::security::{
        claims::UserClaims, middleware::authenticate_bearer_context,
    };

    let secret = "legacy-checkpoint-signed-fixture";
    let security = SecurityConfig {
        provider_credentials: Default::default(),
        deployment_profile: Default::default(), jwt_algorithm: None, workspace_authorities: Vec::new(),
        skill_required_roles: Default::default(),
        api_key_delegable_roles: SecurityConfig::default_api_key_delegable_roles(), api_key_admin_principals: Vec::new(),
        trusted_host_principals: Vec::new(),
        registered_hosts: Vec::new(),
        jwt_required: true, jwt_secret: secret.to_owned().into(),
        jwks_url: None, jwt_issuer: None, jwt_audience: None, jwt_validate_nbf: true,
        settings_mutation_auth_required: true, settings_admin_key: None,
        local_loopback_owner: false,
    };
    // Enter UAR's verifier before the fixture signer can initialize the
    // process-wide jsonwebtoken provider. Verification still supplies proof.
    assert!(matches!(
        authenticate_bearer_context(&security, "context-history-malformed-fixture-token").await,
        Err(axum::http::StatusCode::UNAUTHORIZED)
    ));
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &UserClaims {
        sub: subject.into(), name: None, roles: Some(vec!["user".into()]), tenant_id: None, uar_instance_id: None,
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    }, &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())).unwrap();
    authenticate_bearer_context(&security, &token)
        .await.expect("actual verified context-history fixture principal")
}

pub fn text(role: MessageRole, content: impl Into<String>) -> Message {
    Message {
        role,
        content: MessageContent::text(content),
        tool_call_id: None,
        tool_calls: None,
    }
}

pub fn protected_tool_group() -> Vec<Message> {
    vec![
        Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(""),
            tool_call_id: None,
            tool_calls: Some(vec![ToolCall {
                id: TOOL_CALL_ID.to_string(),
                call_type: "function".to_string(),
                function: ToolCallFunction {
                    name: "fixture_tool".to_string(),
                    arguments: "{\"preserve\":true}".to_string(),
                },
            }]),
        },
        Message {
            role: MessageRole::Tool,
            content: MessageContent::text(protected_payload()),
            tool_call_id: Some(TOOL_CALL_ID.to_string()),
            tool_calls: None,
        },
    ]
}

pub fn protected_payload() -> String {
    format!(
        "{PAYLOAD_HEAD}{}{PAYLOAD_MIDDLE}{}{PAYLOAD_TAIL}",
        "a".repeat(20_000),
        "z".repeat(20_000)
    )
}

pub fn contains_complete_tool_group(messages: &[Message]) -> bool {
    let Some(call_index) = messages.iter().position(|message| {
        message
            .tool_calls
            .iter()
            .flatten()
            .any(|call| call.id == TOOL_CALL_ID)
    }) else {
        return false;
    };
    messages.get(call_index + 1).is_some_and(|message| {
        message.role == MessageRole::Tool
            && message.tool_call_id.as_deref() == Some(TOOL_CALL_ID)
            && message.content.as_text() == Some(protected_payload().as_str())
    })
}

pub fn decode_request_messages(request: &LlmRequest) -> Vec<Message> {
    request
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .expect("captured provider request contains typed messages")
}
