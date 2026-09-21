//! LLM driver implementation backed by [`liter_llm::DefaultClient`].
//!
//! This replaces the legacy `ChatCompletionsDriver` and `ResponsesDriver` with a
//! single unified driver that handles all 142+ providers via liter-llm's provider
//! abstraction, including tool-call normalization across Anthropic, Google, Mistral,
//! and all OpenAI-compatible APIs.

use std::collections::BTreeMap;
use std::sync::Arc;

use futures::Stream;
use liter_llm::{
    ChatCompletionChunk, ChatCompletionRequest, ClientConfig, DefaultClient, FinishReason,
    LlmClient, StreamOptions, ToolChoice, ToolChoiceMode,
};
use sha2::{Digest, Sha256};

use crate::normalized::NormalizedEvent;
use crate::uar::runtime::context::budget::{
    BudgetInput, BudgetOutcome, OutputCeilingField, WireContract, endpoint_fingerprint,
    plan_budget, validate_final_count,
};
use crate::uar::runtime::context::token_service::TokenService;
use crate::uar::telemetry::metrics as telemetry_metrics;

use super::{
    EndpointRequestProfile, EndpointRequestTransform, LlmDriver, LlmRequest, ProfiledWireReceipt,
    ProviderError,
};

/// Accumulated state for a streaming tool call being assembled from deltas.
#[derive(Default)]
struct ToolAccum {
    id: Option<String>,
    name: Option<String>,
    args: String,
}

/// LLM driver powered by liter-llm's `DefaultClient`.
///
/// Handles provider auto-detection, streaming, and tool-call normalization
/// for 142+ providers through a single unified interface.
pub struct LiterLlmDriver {
    client: Arc<DefaultClient>,
    model: String,
    parallel_tool_calls: Option<bool>,
    budget_endpoint_fingerprint: Option<String>,
    endpoint_profile: Option<EndpointRequestProfile>,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for LiterLlmDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiterLlmDriver")
            .field("model", &self.model)
            .field("parallel_tool_calls", &self.parallel_tool_calls)
            .field(
                "has_budget_endpoint_binding",
                &self.budget_endpoint_fingerprint.is_some(),
            )
            .field(
                "endpoint_profile",
                &self
                    .endpoint_profile
                    .as_ref()
                    .map(|profile| profile.profile_id.as_str()),
            )
            .finish()
    }
}

impl LiterLlmDriver {
    /// Create a new driver from a `ClientConfig` and model identifier.
    ///
    /// The `model` should use liter-llm's `provider/model` naming convention
    /// (e.g., `"openai/gpt-4o"`, `"anthropic/claude-sonnet-4"`).
    /// # Errors
    ///
    /// Returns an error if the `DefaultClient` cannot be constructed (e.g.
    /// invalid headers or provider validation failure).
    pub fn new(
        config: ClientConfig,
        model: String,
        parallel_tool_calls: Option<bool>,
    ) -> anyhow::Result<Self> {
        let budget_endpoint_fingerprint = config.base_url.as_deref().map(endpoint_fingerprint);
        let client = Arc::new(DefaultClient::new(config, Some(&model))?);
        Ok(Self {
            client,
            model,
            parallel_tool_calls,
            budget_endpoint_fingerprint,
            endpoint_profile: None,
        })
    }

    /// Create a driver from a pre-built `DefaultClient`.
    #[must_use]
    pub fn from_client(
        client: Arc<DefaultClient>,
        model: String,
        parallel_tool_calls: Option<bool>,
        budget_endpoint_fingerprint: Option<String>,
    ) -> Self {
        Self {
            client,
            model,
            parallel_tool_calls,
            budget_endpoint_fingerprint,
            endpoint_profile: None,
        }
    }

    /// Bind an exact, host-resolved endpoint settings profile to this driver.
    ///
    /// # Errors
    ///
    /// Returns an error when the profile is incomplete or belongs to another
    /// exact model. Request fields are checked immediately before dispatch.
    pub fn with_endpoint_profile(
        mut self,
        profile: EndpointRequestProfile,
    ) -> anyhow::Result<Self> {
        validate_endpoint_profile(
            &profile,
            &self.model,
            self.budget_endpoint_fingerprint.as_deref(),
        )?;
        self.endpoint_profile = Some(profile);
        Ok(self)
    }

    /// Get the model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Return a redacted receipt for the exact profiled serialization that the
    /// driver validates before dispatch.
    ///
    /// # Errors
    ///
    /// Returns an error when no endpoint profile is bound or preparation fails.
    pub fn profiled_wire_receipt(
        &self,
        request: &LlmRequest,
    ) -> anyhow::Result<ProfiledWireReceipt> {
        let prepared = build_chat_request(
            &self.model,
            self.parallel_tool_calls,
            self.budget_endpoint_fingerprint.as_deref(),
            self.endpoint_profile.as_ref(),
            request,
        )?;
        let serialized = prepared
            .validated_wire
            .ok_or_else(|| anyhow::anyhow!("request has no validated endpoint profile"))?;
        let counted_tokens = request
            .budget_contract
            .as_ref()
            .map(|contract| TokenService::count_serialized(contract.counting, &serialized))
            .transpose()?
            .map(|count| count.tokens);
        let digest = Sha256::digest(&serialized);
        let mut sha256 = String::with_capacity(digest.len() * 2);
        use std::fmt::Write as _;
        for byte in digest {
            write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Ok(ProfiledWireReceipt {
            sha256,
            serialized_bytes: serialized.len(),
            counted_tokens,
        })
    }
}

struct PreparedChatRequest {
    request: ChatCompletionRequest,
    validated_wire: Option<Vec<u8>>,
}

fn validate_endpoint_profile(
    profile: &EndpointRequestProfile,
    model: &str,
    bound_endpoint_fingerprint: Option<&str>,
) -> anyhow::Result<()> {
    for (field, value) in [
        ("profile_id", profile.profile_id.as_str()),
        ("profile_revision", profile.profile_revision.as_str()),
        ("provider_id", profile.provider_id.as_str()),
        ("endpoint_kind", profile.endpoint_kind.as_str()),
        (
            "endpoint_fingerprint",
            profile.endpoint_fingerprint.as_str(),
        ),
        ("qualified_model", profile.qualified_model.as_str()),
        ("wire_model", profile.wire_model.as_str()),
        ("model_revision", profile.model_revision.as_str()),
        ("settings_revision", profile.settings_revision.as_str()),
    ] {
        if value.trim().is_empty() {
            anyhow::bail!("endpoint request profile has an empty `{field}`");
        }
    }
    if profile.qualified_model != model {
        anyhow::bail!(
            "endpoint request profile is for {}, but the driver is bound to {model}",
            profile.qualified_model
        );
    }
    let Some((bound_provider_id, _)) = model.split_once('/') else {
        anyhow::bail!("profiled model must have a qualified provider identity");
    };
    if profile.provider_id != bound_provider_id {
        anyhow::bail!(
            "endpoint request profile is for provider {}, but the driver is bound to {bound_provider_id}",
            profile.provider_id
        );
    }
    let Some(bound_endpoint_fingerprint) = bound_endpoint_fingerprint else {
        anyhow::bail!("endpoint request profile requires an explicitly bound endpoint");
    };
    if profile.endpoint_fingerprint != bound_endpoint_fingerprint {
        anyhow::bail!("endpoint request profile does not match the driver endpoint");
    }
    if profile.transform == EndpointRequestTransform::OpenAiCompatibleChatV1
        && profile.wire_model != profile.qualified_model
    {
        anyhow::bail!(
            "OpenAI-compatible exact profiles require identical qualified and wire model values"
        );
    }
    let allowed = profile
        .allowed_request_fields
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    if allowed.len() != profile.allowed_request_fields.len()
        || profile
            .allowed_request_fields
            .iter()
            .any(|field| field.trim().is_empty())
    {
        anyhow::bail!("endpoint request profile contains blank or duplicate request fields");
    }
    for required in ["model", "messages", "stream"] {
        if !allowed.contains(required) {
            anyhow::bail!("endpoint request profile does not allow required field `{required}`");
        }
    }
    Ok(())
}

fn openai_compatible_wire_body(
    model: &str,
    request: &ChatCompletionRequest,
) -> anyhow::Result<serde_json::Value> {
    let mut body = serde_json::to_value(request)?;
    let Some(object) = body.as_object_mut() else {
        anyhow::bail!("final chat request did not serialize to an object");
    };
    object.insert(
        "model".to_string(),
        serde_json::Value::String(model.to_string()),
    );
    object.insert("stream".to_string(), serde_json::Value::Bool(true));
    if let Some(extra_body) = object.remove("extra_body") {
        let serde_json::Value::Object(extra_fields) = extra_body else {
            anyhow::bail!("budgeted extra_body must be a JSON object");
        };
        const RESERVED_FIELDS: &[&str] = &[
            "model",
            "messages",
            "tools",
            "tool_choice",
            "parallel_tool_calls",
            "response_format",
            "stream",
            "stream_options",
            "max_tokens",
            "max_completion_tokens",
        ];
        if let Some(field) = extra_fields
            .keys()
            .find(|field| RESERVED_FIELDS.contains(&field.as_str()))
        {
            anyhow::bail!("budgeted extra_body cannot override reserved field `{field}`");
        }
        object.extend(extra_fields);
        object.insert("stream".to_string(), serde_json::Value::Bool(true));
    }
    Ok(body)
}

fn anthropic_reasoning_minimum(
    extra_body: Option<&serde_json::Value>,
) -> anyhow::Result<Option<u64>> {
    let Some(extra_body) = extra_body.and_then(serde_json::Value::as_object) else {
        return Ok(None);
    };
    let explicit_budget = extra_body
        .get("thinking")
        .and_then(|thinking| thinking.get("budget_tokens"))
        .and_then(serde_json::Value::as_u64);
    let effort_budget = extra_body
        .get("reasoning_effort")
        .and_then(serde_json::Value::as_str)
        .map(|effort| match effort {
            "minimal" | "low" => 1_024,
            "medium" => 4_096,
            "high" => 16_384,
            "max" => 32_768,
            _ => 4_096,
        });
    explicit_budget
        .or(effort_budget)
        .map(|budget| {
            budget
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Anthropic thinking budget overflows max_tokens"))
        })
        .transpose()
}

fn profiled_wire_body(
    model: &str,
    profile: &EndpointRequestProfile,
    request: &ChatCompletionRequest,
    reserved_output_tokens: Option<u64>,
) -> anyhow::Result<serde_json::Value> {
    validate_endpoint_profile(profile, model, Some(&profile.endpoint_fingerprint))?;
    let body = match profile.transform {
        EndpointRequestTransform::OpenAiCompatibleChatV1 => {
            openai_compatible_wire_body(&profile.wire_model, request)?
        }
        EndpointRequestTransform::AnthropicMessagesV1 => {
            if let (Some(required), Some(reserved)) = (
                anthropic_reasoning_minimum(request.extra_body.as_ref())?,
                reserved_output_tokens,
            ) && required > reserved
            {
                anyhow::bail!(
                    "Anthropic transform would raise max_tokens to {required}, above the reserved output ceiling {reserved}"
                );
            }
            anyhow::bail!(
                "Anthropic profile lacks a supported complete final-wire counting contract"
            );
        }
    };
    let Some(object) = body.as_object() else {
        anyhow::bail!("profiled final request did not serialize to an object");
    };
    for field in object.keys() {
        if !profile
            .allowed_request_fields
            .iter()
            .any(|allowed| allowed == field)
        {
            anyhow::bail!(
                "unsupported setting `{field}` for endpoint profile `{}`",
                profile.profile_id
            );
        }
    }
    Ok(body)
}

fn enforce_request_budget(
    model: &str,
    budget_endpoint_fingerprint: Option<&str>,
    endpoint_profile: Option<&EndpointRequestProfile>,
    source: &LlmRequest,
    request: &mut ChatCompletionRequest,
) -> anyhow::Result<Option<Vec<u8>>> {
    let Some(contract) = source.budget_contract.clone() else {
        return Ok(None);
    };
    if contract.destination_model != model {
        anyhow::bail!(
            "request budget destination mismatch: prepared for {}, dispatching to {model}",
            contract.destination_model
        );
    }
    let Some(endpoint_fingerprint) = budget_endpoint_fingerprint else {
        anyhow::bail!("request budget requires an explicitly bound endpoint");
    };
    if contract.destination_endpoint_fingerprint != endpoint_fingerprint {
        anyhow::bail!("request budget endpoint mismatch");
    }
    if contract.wire != WireContract::SyntheticOpenAiCompatibleChatV1 {
        anyhow::bail!("unsupported final request wire contract");
    }
    let output_tokens = u64::try_from(contract.limits.output_tokens)
        .map_err(|_| anyhow::anyhow!("invalid negative output reservation"))?;
    if let Some(profile) = endpoint_profile
        && profile.output_ceiling != Some(contract.output_ceiling)
    {
        anyhow::bail!("endpoint profile output ceiling does not match the request budget contract");
    }
    match contract.output_ceiling {
        OutputCeilingField::MaxTokens => {
            request.max_tokens = Some(output_tokens);
            request.max_completion_tokens = None;
        }
        OutputCeilingField::MaxCompletionTokens => {
            request.max_tokens = None;
            request.max_completion_tokens = Some(output_tokens);
        }
    }

    let final_body = match endpoint_profile {
        Some(profile) => profiled_wire_body(model, profile, request, Some(output_tokens))?,
        None => openai_compatible_wire_body(model, request)?,
    };
    let (required_field, incompatible_field) = match contract.output_ceiling {
        OutputCeilingField::MaxTokens => ("max_tokens", "max_completion_tokens"),
        OutputCeilingField::MaxCompletionTokens => ("max_completion_tokens", "max_tokens"),
    };
    if final_body
        .get(required_field)
        .and_then(serde_json::Value::as_u64)
        != Some(output_tokens)
        || final_body.get(incompatible_field).is_some()
    {
        anyhow::bail!(
            "final request does not enforce the reserved {output_tokens}-token output ceiling"
        );
    }

    let serialized = serde_json::to_vec(&final_body)?;
    let count = TokenService::count_serialized(contract.counting, &serialized)?;
    let outcome = plan_budget(&BudgetInput {
        limits: contract.limits,
        protected_request: count.clone(),
        eligible_prose: Vec::new(),
    });
    let BudgetOutcome::Fit(plan) = outcome else {
        anyhow::bail!("request budget rejected final serialization: {outcome:?}");
    };
    let final_outcome = validate_final_count(&plan, &count);
    if !matches!(final_outcome, BudgetOutcome::Fit(_)) {
        anyhow::bail!("final request validation failed: {final_outcome:?}");
    }
    Ok(Some(serialized))
}

fn build_chat_request(
    model: &str,
    parallel_tool_calls: Option<bool>,
    budget_endpoint_fingerprint: Option<&str>,
    endpoint_profile: Option<&EndpointRequestProfile>,
    req: &LlmRequest,
) -> anyhow::Result<PreparedChatRequest> {
    let messages = convert_messages(&req.messages, req.budget_contract.is_some())?;
    let tools = convert_tools(&req.tools)?;

    let mut chat_req = ChatCompletionRequest::default();
    chat_req.model = model.to_owned();
    chat_req.messages = messages;
    chat_req.tools = if tools.is_empty() { None } else { Some(tools) };
    chat_req.parallel_tool_calls = parallel_tool_calls;
    chat_req.stream_options = Some(StreamOptions {
        include_usage: Some(true),
    });

    if chat_req.tools.is_some() {
        chat_req.tool_choice = Some(ToolChoice::Mode(ToolChoiceMode::Auto));
    }

    // CH-04: per-model dialect params (extended-thinking budgets, reasoning
    // persistence toggles) computed by `PromptDialectEngine`, merged
    // verbatim into the outbound request body.
    chat_req.extra_body = req.extra_params.clone();
    let mut validated_wire = enforce_request_budget(
        model,
        budget_endpoint_fingerprint,
        endpoint_profile,
        req,
        &mut chat_req,
    )?;
    if req.budget_contract.is_none()
        && let Some(profile) = endpoint_profile
    {
        if profile.output_ceiling.is_some() {
            anyhow::bail!(
                "endpoint profile declares an output ceiling but the request has no budget contract"
            );
        }
        let final_body = profiled_wire_body(model, profile, &chat_req, None)?;
        validated_wire = Some(serde_json::to_vec(&final_body)?);
    }
    Ok(PreparedChatRequest {
        request: chat_req,
        validated_wire,
    })
}

/// Prove that a budgeted request fits the supported final-wire contract before
/// handing it to a driver. The leaf repeats this check with its actual endpoint
/// binding immediately before dispatch; this early pass lets host-owned helper
/// calls reject an oversized chunk without opening a provider connection.
pub(crate) fn preflight_budgeted_request(model: &str, req: &LlmRequest) -> anyhow::Result<()> {
    let contract = req
        .budget_contract
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("request budget contract is required"))?;
    build_chat_request(
        model,
        // Including the optional field is conservative for a tool-free helper
        // request and cannot undercount the leaf's final JSON envelope.
        Some(true),
        Some(&contract.destination_endpoint_fingerprint),
        None,
        req,
    )?;
    Ok(())
}

fn record_call_latency(model: &str, call_start: std::time::Instant) {
    let (provider, model_name) = model.split_once('/').unwrap_or(("unknown", model));
    telemetry_metrics::record_llm_call_latency(
        provider,
        model_name,
        call_start.elapsed().as_secs_f64(),
    );
}

#[async_trait::async_trait]
impl LlmDriver for LiterLlmDriver {
    fn with_bound_model(&self, model: &str) -> anyhow::Result<Arc<dyn LlmDriver>> {
        let (provider, _) = self.model.split_once('/').ok_or_else(|| {
            anyhow::anyhow!("Inherited driver has no qualified provider identity")
        })?;
        let (requested_provider, requested_model) = model.split_once('/').ok_or_else(|| {
            anyhow::anyhow!("Child model must have a qualified provider identity")
        })?;
        if provider != requested_provider
            || provider.is_empty()
            || requested_model.trim().is_empty()
        {
            anyhow::bail!("Child model is outside the inherited provider binding");
        }
        if self.endpoint_profile.is_some() && model != self.model {
            anyhow::bail!("An exact endpoint profile cannot authorize another model");
        }
        // Reuse the captured DefaultClient, including its provider/credential
        // bindings. Calling new() here would repeat environment resolution.
        let mut driver = Self::from_client(
            Arc::clone(&self.client),
            model.to_string(),
            self.parallel_tool_calls,
            self.budget_endpoint_fingerprint.clone(),
        );
        driver.endpoint_profile = self.endpoint_profile.clone();
        Ok(Arc::new(driver))
    }

    #[tracing::instrument(
        name = "llm.call",
        skip(self, req),
        fields(model = %self.model),
    )]
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
    {
        let chat_req = build_chat_request(
            &self.model,
            self.parallel_tool_calls,
            self.budget_endpoint_fingerprint.as_deref(),
            self.endpoint_profile.as_ref(),
            &req,
        )
        .map_err(|error| ProviderError::invalid_request(error.to_string()))?
        .request;

        // liter-llm returns an owned `'static` chunk stream. Return the normalized
        // stream immediately so the orchestrator's stream-start timeout covers
        // only request establishment, not the full model completion.
        let metrics_model = self.model.clone();

        // Time the full LLM call (request → stream completion) and record it as
        // a per-call latency histogram when the returned stream finishes.
        let call_start = std::time::Instant::now();
        let call_span = tracing::Span::current();
        let mut chunk_stream = self
            .client
            .chat_stream(chat_req)
            .await
            .map_err(ProviderError::from_liter)?;

        let out = async_stream::stream! {
            let mut tool_accum: BTreeMap<u32, ToolAccum> = BTreeMap::new();
            let mut chunk_count: u64 = 0;
            let mut event_count: u64 = 0;

            loop {
                let next_chunk = futures::future::poll_fn(|cx| {
                    call_span.in_scope(|| chunk_stream.as_mut().poll_next(cx))
                })
                .await;
                let Some(chunk_result) = next_chunk else {
                    break;
                };
                let chunk: ChatCompletionChunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        record_call_latency(&metrics_model, call_start);
                        yield Err(ProviderError::from_liter(e).into());
                        return;
                    }
                };

                chunk_count += 1;

                for choice in &chunk.choices {
                    if let Some(ref text) = choice.delta.content {
                        if !text.is_empty() {
                            event_count += 1;
                            yield Ok(NormalizedEvent::MessageDelta {
                                text: text.clone(),
                            });
                        }
                    }

                    if let Some(ref tool_calls) = choice.delta.tool_calls {
                        for tc in tool_calls {
                            let idx = tc.index;
                            let accum = tool_accum.entry(idx).or_default();

                            if let Some(ref id) = tc.id {
                                accum.id = Some(id.clone());
                            }
                            if let Some(ref func) = tc.function {
                                if let Some(ref name) = func.name {
                                    accum.name = Some(name.clone());
                                }
                                if let Some(ref args) = func.arguments {
                                    accum.args.push_str(args);
                                }
                            }

                            event_count += 1;
                            yield Ok(NormalizedEvent::ToolCallDelta {
                                call_index: idx as usize,
                                id: tc.id.clone(),
                                name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                arguments_delta: tc.function.as_ref().and_then(|f| f.arguments.clone()),
                            });
                        }
                    }

                    if let Some(ref reason) = choice.finish_reason {
                        if matches!(reason, FinishReason::ToolCalls) {
                            for (idx, accum) in &tool_accum {
                                if let (Some(id), Some(name)) = (&accum.id, &accum.name) {
                                    event_count += 1;
                                    yield Ok(NormalizedEvent::ToolCallComplete {
                                        call_index: *idx as usize,
                                        id: id.clone(),
                                        name: name.clone(),
                                        arguments_json: accum.args.clone(),
                                    });
                                }
                            }
                        }
                    }
                }

                if let Some(ref usage) = chunk.usage {
                    event_count += 1;
                    let cached_read = usage
                        .prompt_tokens_details
                        .as_ref()
                        .map(|d| d.cached_tokens);
                    #[expect(clippy::cast_possible_truncation, reason = "token counts fit in u32")]
                    {
                        yield Ok(NormalizedEvent::Usage {
                            prompt_tokens: usage.prompt_tokens as u32,
                            completion_tokens: usage.completion_tokens as u32,
                            total_tokens: usage.total_tokens as u32,
                            cached_tokens: cached_read.map(|t| t as u32),
                            cache_creation_tokens: None,
                        });
                    }

                    // Record LLM token metrics
                    let (provider, model_name) = metrics_model
                        .split_once('/')
                        .unwrap_or(("unknown", &metrics_model));
                    telemetry_metrics::record_llm_tokens(
                        provider,
                        model_name,
                        usage.prompt_tokens,
                        usage.completion_tokens,
                    );
                    // Cache-read tokens only: liter exposes the cached (read)
                    // portion via prompt_tokens_details; cache-creation/write is
                    // folded into provider billing and not separately reported.
                    if let Some(read) = cached_read {
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "token counts fit in u32"
                        )]
                        telemetry_metrics::record_cache_tokens(provider, model_name, 0, read as u32);
                    }
                }
            }

            record_call_latency(&metrics_model, call_start);

            call_span.in_scope(|| {
                tracing::info!(
                    total_chunks = chunk_count,
                    total_events = event_count,
                    "liter-llm stream complete"
                );
            });

            yield Ok(NormalizedEvent::Done);
        };

        Ok(Box::pin(out))
    }
}

#[cfg(test)]
mod prompt_caching_tests {
    use super::*;
    use crate::llm::anthropic_cache::CacheStrategy;
    use futures::StreamExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn request(cache_strategy: Option<CacheStrategy>) -> LlmRequest {
        LlmRequest {
            messages: vec![serde_json::json!({"role": "user", "content": "hello"})],
            tools: Vec::new(),
            cache_strategy,
            thinking_config: None,
            anthropic_system: None,
            extra_params: Some(serde_json::json!({"temperature": 0.2})),
            budget_contract: None,
        }
    }

    #[test]
    fn openai_compatible_body_is_unchanged_by_uar_cache_strategy() {
        let enabled = build_chat_request(
            "openai/gpt-test",
            Some(true),
            None,
            None,
            &request(Some(CacheStrategy::default())),
        )
        .expect("enabled request")
        .request;
        let disabled =
            build_chat_request("openai/gpt-test", Some(true), None, None, &request(None))
                .expect("disabled request")
                .request;

        assert_eq!(
            serde_json::to_value(enabled).expect("serialize enabled request"),
            serde_json::to_value(disabled).expect("serialize disabled request"),
            "UAR prompt-caching policy must not alter OpenAI-compatible bodies"
        );
    }

    #[tokio::test]
    async fn driver_returns_before_provider_stream_finishes() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock provider");
        let address = listener.local_addr().expect("mock provider address");

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = socket.read(&mut buffer).await.expect("read request");
                assert!(read > 0, "provider request ended before its headers");
                request_bytes.extend_from_slice(&buffer[..read]);
            }

            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\n\
                      Content-Type: text/event-stream\r\n\
                      Transfer-Encoding: chunked\r\n\
                      Connection: close\r\n\r\n",
                )
                .await
                .expect("write response headers");

            let first = concat!(
                "data: {\"id\":\"stream-test\",\"object\":\"chat.completion.chunk\",",
                "\"created\":0,\"model\":\"test\",\"choices\":[{\"index\":0,",
                "\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n\n"
            );
            let first_frame = format!("{:X}\r\n{first}\r\n", first.len());
            socket
                .write_all(first_frame.as_bytes())
                .await
                .expect("write first stream chunk");
            socket.flush().await.expect("flush first stream chunk");

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let done = "data: [DONE]\n\n";
            let done_frame = format!("{:X}\r\n{done}\r\n0\r\n\r\n", done.len());
            socket
                .write_all(done_frame.as_bytes())
                .await
                .expect("finish provider stream");
        });

        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url(format!("http://{address}/v1"))
            .max_retries(0)
            .build();
        let driver = LiterLlmDriver::new(config, "openai/test".to_string(), Some(false))
            .expect("build Liter driver");

        let mut stream = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            driver.stream(request(None)),
        )
        .await
        .expect("driver must return after the upstream stream is established")
        .expect("create normalized stream");

        let first_event = tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
            .await
            .expect("first upstream chunk must be forwarded without full-response buffering")
            .expect("normalized event")
            .expect("successful normalized event");
        assert!(
            matches!(first_event, NormalizedEvent::MessageDelta { ref text } if text == "hello"),
            "unexpected first normalized event: {first_event:?}"
        );

        server.await.expect("mock provider task");
    }

    /// Asserts a Prometheus counter incremented, so it needs a real recorder.
    ///
    /// `metrics_handle` returns `&'static PrometheusHandle`, a type that only
    /// exists with the metrics-exporter dependency, so the no-telemetry facade
    /// (`telemetry_disabled.rs`) cannot offer it and deliberately does not.
    /// Ungated, this one test made the WHOLE default-feature test binary fail
    /// to compile -- `cargo test` was broken for the entire crate, while
    /// `cargo check` passed, so nothing surfaced it until someone ran the
    /// tests. Gated the same way `api_metrics` is in server.rs.
    #[cfg(feature = "telemetry")]
    #[tokio::test]
    async fn failed_provider_stream_records_latency_before_error_is_yielded() {
        telemetry_metrics::init();
        let model = "metric-error-test";
        let metric_count = || {
            telemetry_metrics::metrics_handle()
                .render()
                .lines()
                .find(|line| {
                    line.starts_with("uar_llm_call_duration_seconds_count")
                        && line.contains("provider=\"openai\"")
                        && line.contains(&format!("model=\"{model}\""))
                })
                .and_then(|line| line.split_whitespace().last())
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0)
        };
        let count_before = metric_count();

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock provider");
        let address = listener.local_addr().expect("mock provider address");
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = socket.read(&mut buffer).await.expect("read request");
                assert!(read > 0, "provider request ended before its headers");
                request_bytes.extend_from_slice(&buffer[..read]);
            }

            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\n\
                      Content-Type: text/event-stream\r\n\
                      Transfer-Encoding: chunked\r\n\
                      Connection: close\r\n\r\n",
                )
                .await
                .expect("write response headers");
            let invalid = "data: not-json\n\n";
            let invalid_frame = format!("{:X}\r\n{invalid}\r\n0\r\n\r\n", invalid.len());
            socket
                .write_all(invalid_frame.as_bytes())
                .await
                .expect("write invalid stream chunk");
        });

        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url(format!("http://{address}/v1"))
            .max_retries(0)
            .build();
        let driver = LiterLlmDriver::new(config, format!("openai/{model}"), Some(false))
            .expect("build Liter driver");
        let mut stream = driver
            .stream(request(None))
            .await
            .expect("create normalized stream");

        let error = tokio::time::timeout(std::time::Duration::from_secs(1), stream.next())
            .await
            .expect("invalid provider chunk must produce an event")
            .expect("normalized event")
            .expect_err("invalid provider chunks must remain typed stream errors");
        let provider_error = ProviderError::from_anyhow(&error)
            .expect("liter stream failures must preserve provider error metadata");
        assert_eq!(provider_error.kind, crate::llm::ProviderErrorKind::Stream);
        drop(stream);

        assert_eq!(
            metric_count(),
            count_before + 1,
            "failed streams must record latency before consumers can drop the normalized stream"
        );
        server.await.expect("mock provider task");
    }
}

/// Convert UAR's JSON messages to liter-llm's typed `Message` enum.
fn convert_messages(
    messages: &[serde_json::Value],
    require_lossless: bool,
) -> anyhow::Result<Vec<liter_llm::Message>> {
    messages
        .iter()
        .enumerate()
        .map(|(index, msg)| {
            let converted =
                serde_json::from_value::<liter_llm::Message>(msg.clone()).map_err(|error| {
                    anyhow::anyhow!(
                        "Failed to convert message at index {index} to liter-llm format: {error}"
                    )
                })?;
            let rendered = serde_json::to_value(&converted)?;
            if require_lossless && !json_contains_source(&rendered, msg) {
                anyhow::bail!("Message conversion at index {index} would discard protected fields");
            }
            Ok(converted)
        })
        .collect()
}

fn json_contains_source(rendered: &serde_json::Value, source: &serde_json::Value) -> bool {
    match (rendered, source) {
        (serde_json::Value::Object(rendered), serde_json::Value::Object(source)) => {
            source.iter().all(|(key, value)| {
                rendered
                    .get(key)
                    .is_some_and(|item| json_contains_source(item, value))
            })
        }
        (serde_json::Value::Array(rendered), serde_json::Value::Array(source)) => {
            rendered.len() == source.len()
                && rendered
                    .iter()
                    .zip(source)
                    .all(|(item, value)| json_contains_source(item, value))
        }
        _ => rendered == source,
    }
}

/// Convert UAR's JSON tool schemas to liter-llm's typed `ChatCompletionTool`.
fn convert_tools(
    tools: &[serde_json::Value],
) -> anyhow::Result<Vec<liter_llm::ChatCompletionTool>> {
    tools
        .iter()
        .enumerate()
        .map(|(index, tool)| {
            serde_json::from_value::<liter_llm::ChatCompletionTool>(tool.clone()).map_err(|error| {
                anyhow::anyhow!("Failed to convert tool schema at index {index}: {error}")
            })
        })
        .collect()
}
