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
    PreparedFinalWireBudget, ProviderError,
};

/// Accumulated state for a streaming tool call being assembled from deltas.
#[derive(Default)]
struct ToolAccum {
    id: Option<String>,
    name: Option<String>,
    args: String,
}

// Keep aligned with pinned Liter client::build_provider. DashScope's exact
// matches_model implementation strips only its own prefix before comparison.
fn uses_generic_chat_adapter(config: &ClientConfig, model: &str) -> bool {
    config.base_url.is_some()
        && ![
            "azure/",
            "anthropic/",
            "gemini/",
            "google_ai/",
            "vertex_ai/",
            "bedrock/",
        ]
        .iter()
        .any(|prefix| model.starts_with(prefix))
        && model.strip_prefix("dashscope/").unwrap_or(model) != "tongyi-embedding-vision-plus"
}

const GENERIC_ENDPOINT_HINT: &str = "uar-settings-profile";

/// LLM driver powered by liter-llm's `DefaultClient`.
///
/// Handles provider auto-detection, streaming, and tool-call normalization
/// for 142+ providers through a single unified interface.
pub struct LiterLlmDriver {
    client: Arc<DefaultClient>,
    model: String,
    selected_request_model: Option<SelectedRequestModel>,
    parallel_tool_calls: Option<bool>,
    budget_endpoint_fingerprint: Option<String>,
    endpoint_profile: Option<EndpointRequestProfile>,
    generic_chat_adapter: bool,
}

enum SelectedRequestModel {
    Generic(String),
    Special { provider: String, model: String },
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
    /// Bind a settings-only OpenAI chat profile to the real client and wire alias.
    /// Construction uses the trusted adapter contract, never alias-family detection.
    pub fn from_endpoint_profile(
        mut config: ClientConfig,
        parallel_tool_calls: Option<bool>,
        profile: EndpointRequestProfile,
    ) -> anyhow::Result<Self> {
        super::local_only::check_base_url(config.base_url.as_deref())?;
        let fingerprint = config.base_url.as_deref().map(endpoint_fingerprint);
        validate_endpoint_profile(&profile, &profile.qualified_model, fingerprint.as_deref())?;
        anyhow::ensure!(
            profile.transform == EndpointRequestTransform::OpenAiCompatibleChatV1,
            "TEAM_PROFILE_UNSUPPORTED"
        );
        // The captured endpoint is authoritative; provider environment overrides
        // and redirects must not replace it after admission.
        config.load_env = false;
        config.redact_base_url = true;
        config.transport = config.transport.with_redirects_disabled(true);
        // The successful constructor branch is the adapter fact: its constant
        // hint selects Liter's generic endpoint handling independently of the
        // configured wire model's provider spelling.
        let client = Arc::new(DefaultClient::new(config, Some(GENERIC_ENDPOINT_HINT))?);
        let mut driver = Self::from_client(
            client,
            profile.qualified_model.clone(),
            parallel_tool_calls,
            fingerprint,
        );
        // `from_client` is deliberately opaque. This constructor owns the
        // successful generic-compatible branch, so it alone may record that
        // concrete adapter fact for the captured endpoint profile.
        driver.generic_chat_adapter = true;
        driver.with_endpoint_profile(profile)
    }
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
        // Mirror pinned Liter's client::build_provider explicit-endpoint branch.
        // Its generic adapter has a no-op request transform; the registry ID
        // and the wire model are not evidence of which adapter was constructed.
        let generic_chat_adapter = uses_generic_chat_adapter(&config, &model);
        let client = Arc::new(DefaultClient::new(config, Some(&model))?);
        Ok(Self {
            client,
            model,
            selected_request_model: None,
            parallel_tool_calls,
            budget_endpoint_fingerprint,
            endpoint_profile: None,
            generic_chat_adapter,
        })
    }

    /// Build a selected route with separate accounting and provider-wire identities.
    pub(crate) fn from_selected_route(
        config: ClientConfig,
        previous_model: String,
        adapter_provider: &str,
        destination: String,
        wire_model: String,
        parallel_tool_calls: Option<bool>,
    ) -> anyhow::Result<Self> {
        if config.base_url.is_none() {
            return Self::new(config, previous_model, parallel_tool_calls);
        }
        // The selected provider or explicit host protocol owns the adapter.
        // A gateway's literal alias may start with another provider's name.
        let adapter_model = format!("{adapter_provider}/{wire_model}");
        let generic = uses_generic_chat_adapter(&config, &adapter_model);
        let hint = if generic {
            GENERIC_ENDPOINT_HINT
        } else {
            adapter_model.as_str()
        };
        let fingerprint = config.base_url.as_deref().map(endpoint_fingerprint);
        let client = Arc::new(DefaultClient::new(config, Some(hint))?);
        let mut driver = Self::from_client(client, destination, parallel_tool_calls, fingerprint);
        driver.generic_chat_adapter = generic;
        driver.selected_request_model = Some(if generic {
            SelectedRequestModel::Generic(wire_model)
        } else {
            SelectedRequestModel::Special {
                provider: adapter_provider.to_owned(),
                model: adapter_model,
            }
        });
        Ok(driver)
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
            selected_request_model: None,
            parallel_tool_calls,
            budget_endpoint_fingerprint,
            endpoint_profile: None,
            generic_chat_adapter: false,
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
        let prepared = self.build_chat_request(request)?;
        anyhow::ensure!(
            self.endpoint_profile.is_some(),
            "request has no validated endpoint profile"
        );
        // Liter owns provider routing-prefix normalization and any supported
        // transform. Its prepared bytes are therefore the only body this
        // receipt can certify.
        let (serialized, final_count) = self.prepared_final_wire(request, &prepared)?;
        let counted_tokens = final_count.map(|count| count.tokens);
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

impl LiterLlmDriver {
    fn build_chat_request(&self, request: &LlmRequest) -> anyhow::Result<PreparedChatRequest> {
        build_chat_request_for_model(
            &self.model,
            self.parallel_tool_calls,
            self.budget_endpoint_fingerprint.as_deref(),
            self.endpoint_profile.as_ref(),
            self.generic_chat_adapter,
            self.selected_request_model.as_ref(),
            request,
        )
    }

    /// Prepare Liter's endpoint-bound stream representation for a request that
    /// has already passed UAR's typed request construction.
    fn prepared_final_wire(
        &self,
        source: &LlmRequest,
        prepared: &PreparedChatRequest,
    ) -> anyhow::Result<(Vec<u8>, Option<crate::uar::runtime::context::token_service::CountedTokens>)> {
        let serialized = self.client.chat_stream_request_body(&prepared.request)?;
        let final_count = validate_dispatched_budget(
            &self.model,
            match &self.selected_request_model {
                Some(SelectedRequestModel::Generic(model)) => Some(model.as_str()),
                _ => None,
            },
            self.endpoint_profile.as_ref(),
            source,
            prepared,
            &serialized,
        )?;
        Ok((serialized, final_count))
    }
}

struct PreparedChatRequest {
    request: ChatCompletionRequest,
    /// UAR preflight serialization. The leaf certifies Liter's transformed
    /// bytes separately immediately before dispatch.
    preflight_wire: Option<Vec<u8>>,
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
    exact_wire_model: Option<&str>,
    budget_endpoint_fingerprint: Option<&str>,
    endpoint_profile: Option<&EndpointRequestProfile>,
    source: &LlmRequest,
    request: &mut ChatCompletionRequest,
) -> anyhow::Result<Option<Vec<u8>>> {
    let Some(mut contract) = source.budget_contract.clone() else {
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
    if let Some(cap) = source.output_token_cap {
        contract.limits.output_tokens = contract.limits.output_tokens.min(i64::from(cap));
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
        None => openai_compatible_wire_body(exact_wire_model.unwrap_or(model), request)?,
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

/// Revalidate the bytes produced by Liter's prepared stream request immediately
/// before dispatch. `ChatCompletionRequest` retains the host-selected model,
/// template, settings, output ceiling, and protected messages; Liter owns only
/// the endpoint's literal provider transformation of that immutable request.
fn validate_dispatched_budget(
    model: &str,
    exact_wire_model: Option<&str>,
    endpoint_profile: Option<&EndpointRequestProfile>,
    source: &LlmRequest,
    prepared: &PreparedChatRequest,
    serialized: &[u8],
) -> anyhow::Result<Option<crate::uar::runtime::context::token_service::CountedTokens>> {
    let validates_controls = source.budget_contract.is_some() || endpoint_profile.is_some();
    if !validates_controls && exact_wire_model.is_none() {
        return Ok(None);
    }
    let body: serde_json::Value = serde_json::from_slice(serialized)?;
    let literal_model = body
        .get("model")
        .and_then(serde_json::Value::as_str)
        .filter(|model| !model.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("final provider request has no literal model"))?;
    if let Some(exact_wire_model) = exact_wire_model {
        anyhow::ensure!(
            literal_model == exact_wire_model,
            "final provider request does not preserve the selected wire model"
        );
    }
    if !validates_controls {
        return Ok(None);
    }
    let budget = match source.budget_contract.clone() {
        Some(mut contract) => {
            if let Some(cap) = source.output_token_cap {
                contract.limits.output_tokens = contract.limits.output_tokens.min(i64::from(cap));
            }
            let output_tokens = u64::try_from(contract.limits.output_tokens)
                .map_err(|_| anyhow::anyhow!("invalid negative output reservation"))?;
            Some((contract, output_tokens))
        }
        None => None,
    };
    let expected = match endpoint_profile {
        Some(profile) => profiled_wire_body(
            model,
            profile,
            &prepared.request,
            budget.as_ref().map(|(_, output_tokens)| *output_tokens),
        )?,
        // The supported synthetic contract is generic OpenAI-compatible JSON.
        // Its literal model is selected by the captured Liter adapter; all
        // other request controls must remain byte-for-byte equivalent at the
        // JSON-value layer after that literal selection.
        None => openai_compatible_wire_body(literal_model, &prepared.request)?,
    };
    anyhow::ensure!(
        body == expected,
        "final provider request does not preserve the prepared wire controls"
    );
    // Settings-only profiles still bind the exact model and controls after
    // Liter's transform, even when no token-counting contract is available.
    let Some((contract, output_tokens)) = budget else {
        return Ok(None);
    };
    let (required_field, incompatible_field) = match contract.output_ceiling {
        OutputCeilingField::MaxTokens => ("max_tokens", "max_completion_tokens"),
        OutputCeilingField::MaxCompletionTokens => ("max_completion_tokens", "max_tokens"),
    };
    anyhow::ensure!(
        body.get(required_field).and_then(serde_json::Value::as_u64) == Some(output_tokens)
            && body.get(incompatible_field).is_none(),
        "final provider request does not enforce the reserved {output_tokens}-token output ceiling"
    );
    anyhow::ensure!(
        body.get("stream") == Some(&serde_json::Value::Bool(true)),
        "final provider request does not preserve streaming"
    );
    let count = TokenService::count_serialized(contract.counting, serialized)?;
    let outcome = plan_budget(&BudgetInput {
        limits: contract.limits,
        protected_request: count.clone(),
        eligible_prose: Vec::new(),
    });
    let BudgetOutcome::Fit(plan) = outcome else {
        anyhow::bail!("request budget rejected final provider serialization: {outcome:?}");
    };
    let final_outcome = validate_final_count(&plan, &count);
    anyhow::ensure!(
        matches!(final_outcome, BudgetOutcome::Fit(_)),
        "final provider request validation failed: {final_outcome:?}"
    );
    Ok(Some(count))
}

fn build_chat_request(
    model: &str,
    parallel_tool_calls: Option<bool>,
    budget_endpoint_fingerprint: Option<&str>,
    endpoint_profile: Option<&EndpointRequestProfile>,
    generic_chat_adapter: bool,
    req: &LlmRequest,
) -> anyhow::Result<PreparedChatRequest> {
    build_chat_request_for_model(
        model,
        parallel_tool_calls,
        budget_endpoint_fingerprint,
        endpoint_profile,
        generic_chat_adapter,
        None,
        req,
    )
}

fn build_chat_request_for_model(
    model: &str,
    parallel_tool_calls: Option<bool>,
    budget_endpoint_fingerprint: Option<&str>,
    endpoint_profile: Option<&EndpointRequestProfile>,
    generic_chat_adapter: bool,
    selected_request_model: Option<&SelectedRequestModel>,
    req: &LlmRequest,
) -> anyhow::Result<PreparedChatRequest> {
    anyhow::ensure!(
        req.output_token_cap != Some(0),
        "Output token cap must be positive"
    );
    req.validate_tool_controls()?;
    let messages = convert_messages(&req.messages, req.budget_contract.is_some())?;
    let tools = convert_tools(&req.tools)?;

    let (adapter_model, exact_wire_model) = match selected_request_model {
        Some(SelectedRequestModel::Generic(wire_model)) => (model, Some(wire_model.as_str())),
        Some(SelectedRequestModel::Special { model, .. }) => (model.as_str(), None),
        None => (model, None),
    };
    let mut chat_req = ChatCompletionRequest::default();
    chat_req.model = endpoint_profile.map_or_else(
        || adapter_model.to_owned(),
        |profile| profile.wire_model.clone(),
    );
    chat_req.messages = messages;
    chat_req.tools = if tools.is_empty() { None } else { Some(tools) };
    chat_req.parallel_tool_calls = req.parallel_tool_calls.or(parallel_tool_calls);
    // Liter provider adapters may strip unsupported fields. Refuse explicit
    // caller settings unless this path preserves their exact semantics.
    let provider = if generic_chat_adapter {
        "openai-compatible"
    } else {
        super::detect_provider(adapter_model)
    };
    if req.parallel_tool_calls.is_some()
        && !matches!(
            provider,
            "openai" | "azure" | "azure_openai" | "openai-compatible"
        )
        && endpoint_profile.is_none()
    {
        anyhow::bail!(
            "provider capability error: explicit parallel_tool_calls is unsupported by the {provider} Liter adapter"
        );
    }
    if req.tool_choice.is_some()
        && !matches!(
            provider,
            "openai" | "azure" | "azure_openai" | "openai-compatible" | "anthropic"
        )
        && endpoint_profile.is_none()
    {
        anyhow::bail!(
            "provider capability error: explicit tool_choice is unsupported by the {provider} Liter adapter"
        );
    }
    if !matches!(
        provider,
        "openai" | "azure" | "azure_openai" | "openai-compatible"
    ) && endpoint_profile.is_none()
        && req
            .tools
            .iter()
            .any(|tool| tool.pointer("/function/strict").is_some())
    {
        anyhow::bail!(
            "provider capability error: strict tool semantics are unsupported by the {provider} Liter adapter; Anthropic callers can select the native driver"
        );
    }
    if provider == "anthropic"
        && req.tools.iter().any(|tool| {
            tool.pointer("/function/parameters/type")
                .and_then(serde_json::Value::as_str)
                != Some("object")
        })
    {
        anyhow::bail!(
            "provider capability error: the Anthropic Liter adapter would rewrite the tool input schema; select the native driver"
        );
    }
    if let Some(extra) = req
        .extra_params
        .as_ref()
        .and_then(serde_json::Value::as_object)
    {
        if extra.contains_key("tool_choice")
            || extra.contains_key("parallel_tool_calls")
            || extra.contains_key("tools")
        {
            anyhow::bail!("tool controls must use typed request fields, not extra_params");
        }
        if req.output_token_cap.is_some()
            && (extra.contains_key("max_tokens") || extra.contains_key("max_completion_tokens"))
        {
            anyhow::bail!("output ceiling must use the typed request cap, not extra_params");
        }
    }
    if let Some(cap) = req.output_token_cap {
        if provider == "anthropic"
            && anthropic_reasoning_minimum(req.extra_params.as_ref())?
                .is_some_and(|required| required > u64::from(cap))
        {
            anyhow::bail!(
                "Anthropic thinking configuration would exceed the caller output token cap"
            );
        }
        let output_field = endpoint_profile
            .and_then(|profile| profile.output_ceiling)
            .unwrap_or(if provider == "anthropic" {
                OutputCeilingField::MaxTokens
            } else {
                OutputCeilingField::MaxCompletionTokens
            });
        match output_field {
            OutputCeilingField::MaxTokens => chat_req.max_tokens = Some(u64::from(cap)),
            OutputCeilingField::MaxCompletionTokens => {
                chat_req.max_completion_tokens = Some(u64::from(cap))
            }
        }
    }
    chat_req.stream_options = Some(StreamOptions {
        include_usage: Some(true),
    });

    chat_req.tool_choice = match req.tool_choice.as_ref() {
        Some(super::ToolChoice::Auto) => Some(ToolChoice::Mode(ToolChoiceMode::Auto)),
        Some(super::ToolChoice::None) => Some(ToolChoice::Mode(ToolChoiceMode::None)),
        Some(super::ToolChoice::Required) => Some(ToolChoice::Mode(ToolChoiceMode::Required)),
        Some(super::ToolChoice::Named(name)) => {
            Some(ToolChoice::Specific(liter_llm::SpecificToolChoice {
                choice_type: liter_llm::ToolType::Function,
                function: liter_llm::SpecificFunction { name: name.clone() },
            }))
        }
        None if chat_req.tools.is_some() => Some(ToolChoice::Mode(ToolChoiceMode::Auto)),
        None => None,
    };

    // CH-04: per-model dialect params (extended-thinking budgets, reasoning
    // persistence toggles) computed by `PromptDialectEngine`, merged
    // verbatim into the outbound request body.
    chat_req.extra_body = req.extra_params.clone();
    if let Some(extra) = chat_req.extra_body.as_mut().and_then(serde_json::Value::as_object_mut)
        && let Some(format) = extra.remove("response_format")
    {
        let format: super::response_format::ResponseFormat = serde_json::from_value(format)?;
        format.validate()?;
        anyhow::ensure!(!format.is_structured()
            || matches!(provider, "openai" | "azure" | "azure_openai" | "openai-compatible"),
            "provider capability error: response_format is unsupported by the {provider} Liter adapter");
        chat_req.response_format = Some(serde_json::from_value(serde_json::to_value(format)?)?);
    }
    let mut preflight_wire = enforce_request_budget(
        model,
        exact_wire_model,
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
        preflight_wire = Some(serde_json::to_vec(&final_body)?);
    }
    if let Some(wire_model) = exact_wire_model {
        // Pinned Liter strips the generic adapter's `custom/` prefix once.
        // Supply that transport prefix even when the literal itself starts with it.
        chat_req.model = format!("custom/{wire_model}");
    }
    Ok(PreparedChatRequest {
        request: chat_req,
        preflight_wire,
    })
}

fn request_contains_image_parts(request: &LlmRequest) -> bool {
    request.messages.iter().any(|message| {
        message
            .get("content")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|parts| {
                parts.iter().any(|part| {
                    part.get("type").and_then(serde_json::Value::as_str) == Some("image_url")
                        && part.get("image_url").is_some()
                })
            })
    })
}

/// Validate the existing auxiliary request contract before its governed leaf
/// dispatch. This preflight has no bound Liter client, so it is not durable
/// final-wire reservation evidence; the leaf validates actual transformed
/// bytes immediately before dispatch.
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
        false,
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
    fn supports_admitted_inline_images(&self) -> bool {
        // Only constructors that own the successful pinned `DefaultClient::new`
        // branch set this. `from_client` stays opaque and false, so this reports
        // actual adapter construction independently of catalog capabilities.
        self.generic_chat_adapter
    }

    fn prepared_final_wire_budget(
        &self,
        request: &LlmRequest,
    ) -> anyhow::Result<PreparedFinalWireBudget> {
        let contract = request
            .budget_contract
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("prepared final-wire budget requires a request budget contract"))?;
        anyhow::ensure!(
            !request_contains_image_parts(request),
            "final-wire JSON does not prove a finite image accounting bound"
        );
        let prepared = self.build_chat_request(request)?;
        let (_, final_count) = self.prepared_final_wire(request, &prepared)?;
        let input_tokens = final_count
            .ok_or_else(|| anyhow::anyhow!("prepared final-wire budget was not validated"))?
            .tokens;
        let effective_output = request
            .output_token_cap
            .map_or(contract.limits.output_tokens, |cap| {
                contract.limits.output_tokens.min(i64::from(cap))
            });
        let output_tokens = u64::try_from(effective_output)
            .map_err(|_| anyhow::anyhow!("invalid negative output reservation"))?;
        Ok(PreparedFinalWireBudget {
            input_tokens,
            output_tokens,
        })
    }

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
        driver.generic_chat_adapter = self.generic_chat_adapter;
        driver.selected_request_model = match &self.selected_request_model {
            Some(SelectedRequestModel::Generic(_)) => {
                Some(SelectedRequestModel::Generic(requested_model.to_owned()))
            }
            Some(SelectedRequestModel::Special { provider, .. }) => {
                Some(SelectedRequestModel::Special {
                    provider: provider.clone(),
                    model: format!("{provider}/{requested_model}"),
                })
            }
            None => None,
        };
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
        super::orchestrator::record_provider_preparation();
        let has_image_parts = request_contains_image_parts(&req);
        let prepared = self
            .build_chat_request(&req)
            .map_err(|error| ProviderError::invalid_request(error.to_string()))?;
        // Count the same preserving final wire representation used for dispatch.
        // Real model JSON tokenization is an estimate unless an installed exact
        // destination contract proves the framing; catalog metadata cannot do so.
        let wire_receipt = if prepared.preflight_wire.is_some() || self.generic_chat_adapter
            || matches!(super::detect_provider(&self.model), "openai" | "azure" | "azure_openai") {
            let (wire, final_count) = self
                .prepared_final_wire(&req, &prepared)
                .map_err(|error| ProviderError::invalid_request(error.to_string()))?;
            let counting = req.budget_contract.as_ref().map(|contract| contract.counting)
                .unwrap_or(crate::uar::runtime::context::token_service::SerializedCountingContract::ApproximateCl100kJsonV1);
            let count = match final_count {
                Some(count) => count,
                None => TokenService::count_serialized(counting, &wire)?,
            };
            let quality = match count.quality {
                crate::uar::runtime::context::token_service::CountQuality::Exact => "exact",
                crate::uar::runtime::context::token_service::CountQuality::ValidatedUpperBound => "validated_upper_bound",
                crate::uar::runtime::context::token_service::CountQuality::Approximate => "approximate",
            };
            Some(serde_json::json!({
                "schema_version":"uar.request-budget.v1", "representation":"final_provider_request",
                "destination_model":self.model,
                "serialized_request":true, "count_quality":quality, "counting_revision":count.revision,
                "counted_input_tokens":count.tokens, "serialized_bytes":wire.len(),
                "serialized_sha256":Sha256::digest(&wire).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                "tools_included":true, "instructions_included":true,
                // A final JSON count bounds the serialized wire body, not a
                // real provider's image-token accounting. No inspected
                // destination contract establishes that missing evidence.
                "fit_guarantee":req.budget_contract.is_some() && !has_image_parts,
                "reserved_output_tokens":req.budget_contract.as_ref().map(|contract| contract.limits.output_tokens),
                "output_ceiling_enforced":prepared.request.max_tokens.is_some() || prepared.request.max_completion_tokens.is_some(),
            }))
        } else { None };
        let chat_req = prepared.request;

        // liter-llm returns an owned `'static` chunk stream. Return the normalized
        // stream immediately so the orchestrator's stream-start timeout covers
        // only request establishment, not the full model completion.
        let metrics_model = self.model.clone();

        // Time the full LLM call (request → stream completion) and record it as
        // a per-call latency histogram when the returned stream finishes.
        let call_start = std::time::Instant::now();
        let call_span = tracing::Span::current();
        super::orchestrator::record_provider_submission();
        let mut chunk_stream = self
            .client
            .chat_stream(chat_req)
            .await
            .map_err(ProviderError::from_liter)?;

        let out = async_stream::stream! {
            if let Some(payload) = wire_receipt {
                yield Ok(NormalizedEvent::Custom { source:"uar.request".into(), event_name:"budget_receipt".into(), payload });
            }
            let mut tool_accum: BTreeMap<u32, ToolAccum> = BTreeMap::new();
            let mut chunk_count: u64 = 0;
            let mut event_count: u64 = 0;
            let mut saw_terminal = false;
            let mut saw_tool_finish = false;

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
                    if let Some(ref reasoning) = choice.delta.reasoning_content {
                        if !reasoning.is_empty() {
                            event_count += 1;
                            yield Ok(NormalizedEvent::ReasoningDelta {
                                text: reasoning.clone(),
                            });
                        }
                    }

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

                            if let Err(reason) = super::tool_call_identity::merge(&mut accum.id, tc.id.as_deref()) {
                                record_call_latency(&metrics_model, call_start);
                                yield Err(super::tool_call_identity::failure(reason));
                                return;
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
                        saw_terminal = true;
                        // Gemini completes function calls with STOP. The actual
                        // protocol terminal and trailing framing still gate EOF.
                        saw_tool_finish |= matches!(reason, FinishReason::ToolCalls)
                            || (matches!(reason, FinishReason::Stop) && !tool_accum.is_empty());
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
                            uncached_input_tokens: None,
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


            if !saw_terminal {
                yield Err(ProviderError::new(None, super::ProviderErrorKind::Stream, None, "Provider stream ended without a terminal completion").into());
                return;
            }
            if saw_tool_finish {
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

    fn assert_final_wire_budget_receipt(
        event: Option<anyhow::Result<NormalizedEvent>>,
        destination: &str,
    ) {
        let event = event.expect("budget receipt event").expect("valid budget receipt");
        let NormalizedEvent::Custom {
            source,
            event_name,
            payload,
        } = &event
        else {
            panic!("expected the final-wire budget receipt before provider events: {event:?}");
        };
        assert_eq!(source, "uar.request");
        assert_eq!(event_name, "budget_receipt");
        assert_eq!(payload["schema_version"], "uar.request-budget.v1");
        assert_eq!(payload["representation"], "final_provider_request");
        assert_eq!(payload["destination_model"], destination);
        assert_eq!(payload["serialized_request"], true);
    }

    fn request(cache_strategy: Option<CacheStrategy>) -> LlmRequest {
        LlmRequest {
            messages: vec![serde_json::json!({"role": "user", "content": "hello"})],
            tools: Vec::new(),
            tool_choice: None,
            parallel_tool_calls: None,
            output_token_cap: None,
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
            false,
            &request(Some(CacheStrategy::default())),
        )
        .expect("enabled request")
        .request;
        let disabled = build_chat_request(
            "openai/gpt-test",
            Some(true),
            None,
            None,
            false,
            &request(None),
        )
        .expect("disabled request")
        .request;

        assert_eq!(
            serde_json::to_value(enabled).expect("serialize enabled request"),
            serde_json::to_value(disabled).expect("serialize disabled request"),
            "UAR prompt-caching policy must not alter OpenAI-compatible bodies"
        );
    }

    fn settings_only_profile_request(wire_model: &str) -> (LiterLlmDriver, LlmRequest) {
        let base_url = "http://127.0.0.1:1/v1";
        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url(base_url)
            .build();
        let profile = EndpointRequestProfile {
            profile_id: "settings-only-test".into(),
            profile_revision: "1".into(),
            provider_id: "gateway".into(),
            endpoint_kind: "openai-compatible-chat".into(),
            endpoint_fingerprint: endpoint_fingerprint(base_url),
            qualified_model: "gateway/custom/model-a".into(),
            wire_model: wire_model.into(),
            model_revision: "1".into(),
            settings_revision: "1".into(),
            transform: EndpointRequestTransform::OpenAiCompatibleChatV1,
            allowed_request_fields: [
                "model", "messages", "stream", "stream_options", "tools", "tool_choice",
                "parallel_tool_calls", "temperature", "response_format",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            output_ceiling: None,
        };
        let driver = LiterLlmDriver::from_endpoint_profile(config, None, profile)
            .expect("construct settings-only profile driver");
        let mut req = request(None);
        req.parallel_tool_calls = Some(false);
        req.tool_choice = Some(crate::llm::ToolChoice::Named("lookup".into()));
        req.tools = vec![serde_json::json!({"type":"function","function":{
            "name":"lookup", "strict":true,
            "parameters":{"type":"object","properties":{},"additionalProperties":false}
        }})];
        req.extra_params = Some(serde_json::json!({
            "temperature":0.2,
            "response_format":{"type":"json_schema","json_schema":{
                "name":"answer", "strict":true,
                "schema":{"type":"object","properties":{},"additionalProperties":false}
            }}
        }));
        (driver, req)
    }

    #[test]
    fn settings_only_profile_refuses_liter_rewritten_wire_alias() {
        let (driver, req) = settings_only_profile_request("custom/model-a");
        assert!(req.budget_contract.is_none());
        let prepared = build_chat_request(
            driver.model(),
            driver.parallel_tool_calls,
            driver.budget_endpoint_fingerprint.as_deref(),
            driver.endpoint_profile.as_ref(),
            driver.generic_chat_adapter,
            &req,
        )
        .expect("settings-only profile passes preflight");
        let preflight: serde_json::Value = serde_json::from_slice(
            prepared.preflight_wire.as_deref().expect("profile preflight"),
        )
        .expect("decode preflight");
        let actual = driver.client.chat_stream_request_body(&prepared.request)
            .expect("prepare actual Liter body without dispatch");
        let body: serde_json::Value = serde_json::from_slice(&actual).expect("decode Liter body");
        assert_eq!(preflight["model"], "custom/model-a");
        assert_eq!(body["model"], "model-a");
        let error = driver.prepared_final_wire(&req, &prepared)
            .expect_err("actual prepared alias must match the captured profile");
        assert!(error.to_string().contains("does not preserve the prepared wire controls"));
        assert!(driver.profiled_wire_receipt(&req).is_err());
    }

    #[test]
    fn settings_only_profile_preserves_prepared_alias_controls_and_schema() {
        let (driver, req) = settings_only_profile_request("vendor/model-a");
        assert!(req.budget_contract.is_none());
        let prepared = build_chat_request(
            driver.model(),
            driver.parallel_tool_calls,
            driver.budget_endpoint_fingerprint.as_deref(),
            driver.endpoint_profile.as_ref(),
            driver.generic_chat_adapter,
            &req,
        )
        .expect("prepare settings-only profile");
        let (actual, count) = driver.prepared_final_wire(&req, &prepared)
            .expect("preserved actual Liter body must pass without a budget contract");
        let body: serde_json::Value = serde_json::from_slice(&actual).expect("decode Liter body");
        assert!(count.is_none());
        assert_eq!(body["model"], "vendor/model-a");
        assert_eq!(body["parallel_tool_calls"], false);
        assert_eq!(body["tool_choice"]["function"]["name"], "lookup");
        assert_eq!(body["tools"], serde_json::json!(req.tools));
        assert_eq!(body["response_format"], req.extra_params.as_ref().unwrap()["response_format"]);
        let receipt = driver.profiled_wire_receipt(&req).expect("preserved profile receipt");
        assert_eq!(receipt.serialized_bytes, actual.len());
        assert!(receipt.counted_tokens.is_none());

        for (field, value) in [
            ("parallel_tool_calls", serde_json::json!(true)),
            ("response_format", serde_json::json!({"type":"json_object"})),
            ("unapproved_control", serde_json::json!(true)),
        ] {
            let mut changed = body.clone();
            changed[field] = value;
            let serialized = serde_json::to_vec(&changed).expect("serialize changed prepared body");
            let error = validate_dispatched_budget(
                driver.model(), None, driver.endpoint_profile.as_ref(), &req, &prepared, &serialized,
            )
            .expect_err("settings-only profile must refuse altered final controls");
            assert!(error.to_string().contains("does not preserve the prepared wire controls"));
        }
    }

    #[test]
    fn selected_generic_routes_preserve_literal_aliases_and_exact_budget_identity() {
        use crate::uar::runtime::context::budget::RequestBudgetContract;

        let base_url = "http://127.0.0.1:1/v1";
        for (provider, alias, host_protocol) in [
            ("openai", "gpt-4o-mini", None),
            ("siliconflow-cn", "Pro/deepseek-ai/DeepSeek-V3", None),
            ("siliconflow-cn", "deepseek-ai/DeepSeek-V3", None),
            ("groq", "groq/compound", None),
            ("openrouter", "openrouter/auto", None),
            ("openrouter", "anthropic/claude-haiku-4.5", None),
            (
                "anthropic",
                "anthropic/claude-haiku-4.5",
                Some("openai_compatible"),
            ),
            (
                "bedrock",
                "bedrock/anthropic.claude-v2",
                Some("openai_compatible"),
            ),
            ("gateway", "custom/route", None),
            ("gateway", "uar-settings-profile/route", None),
        ] {
            let config = liter_llm::ClientConfigBuilder::new("test-key")
                .base_url(base_url)
                .build();
            let destination = format!("{provider}/{alias}");
            let driver = LiterLlmDriver::from_selected_route(
                config,
                alias.to_owned(),
                host_protocol.unwrap_or(provider),
                destination.clone(),
                alias.to_owned(),
                None,
            )
            .expect("construct selected generic endpoint");
            let mut request = request(None);
            request.output_token_cap = Some(128);
            let mut contract = RequestBudgetContract::synthetic_exact(base_url);
            contract.destination_model.clone_from(&destination);
            request.budget_contract = Some(contract);

            let prepared = driver.build_chat_request(&request).unwrap();
            let (wire, count) = driver.prepared_final_wire(&request, &prepared).unwrap();
            let body: serde_json::Value = serde_json::from_slice(&wire).unwrap();
            let bound = driver.prepared_final_wire_budget(&request).unwrap();
            assert_eq!(driver.model(), destination);
            assert_eq!(body["model"], alias);
            assert_eq!(body["max_completion_tokens"], 128);
            assert_eq!(bound.output_tokens, 128);
            assert_eq!(bound.input_tokens, count.unwrap().tokens);

            request.budget_contract = None;
            request.extra_params = Some(serde_json::json!({"model": "different-route"}));
            let prepared = driver.build_chat_request(&request).unwrap();
            let error = driver.prepared_final_wire(&request, &prepared).unwrap_err();
            assert!(error.to_string().contains("does not preserve the selected wire model"));
        }
    }

    #[test]
    fn selected_anthropic_protocol_retains_the_messages_adapter() {
        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url("http://127.0.0.1:1/v1")
            .build();
        let driver = LiterLlmDriver::from_selected_route(
            config,
            "claude-haiku-4.5".into(),
            "anthropic",
            "host-provider/claude-haiku-4.5".into(),
            "claude-haiku-4.5".into(),
            None,
        )
        .expect("construct selected Anthropic endpoint");
        assert_eq!(driver.model(), "host-provider/claude-haiku-4.5");
        assert!(!driver.supports_admitted_inline_images());

        let mut request = request(None);
        request.output_token_cap = Some(128);
        let prepared = driver.build_chat_request(&request).unwrap();
        let (wire, _) = driver.prepared_final_wire(&request, &prepared).unwrap();
        let body: serde_json::Value = serde_json::from_slice(&wire).unwrap();
        assert_eq!(body["model"], "claude-haiku-4.5");
        assert_eq!(body["max_tokens"], 128);
        assert!(body.get("stream_options").is_none());
    }

    #[test]
    fn prepared_final_wire_budget_uses_liter_bytes_and_refuses_image_liability() {
        use crate::uar::runtime::context::budget::{
            RequestBudgetContract, SYNTHETIC_EXACT_MODEL,
        };
        use crate::uar::runtime::context::token_service::TokenService;

        let base_url = "http://127.0.0.1:1/v1";
        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url(base_url)
            .build();
        let driver = LiterLlmDriver::new(
            config,
            SYNTHETIC_EXACT_MODEL.to_string(),
            Some(true),
        )
        .expect("construct synthetic exact driver");
        let mut request = request(None);
        request.output_token_cap = Some(128);
        request.budget_contract = Some(RequestBudgetContract::synthetic_exact(base_url));

        let prepared = build_chat_request(
            driver.model(),
            driver.parallel_tool_calls,
            driver.budget_endpoint_fingerprint.as_deref(),
            driver.endpoint_profile.as_ref(),
            driver.generic_chat_adapter,
            &request,
        )
        .expect("prepare synthetic exact request");
        let actual = driver
            .client
            .chat_stream_request_body(&prepared.request)
            .expect("prepare Liter stream body");
        let body: serde_json::Value = serde_json::from_slice(&actual).expect("decode Liter body");
        let expected = TokenService::count_serialized(
            request
                .budget_contract
                .as_ref()
                .expect("exact contract")
                .counting,
            &actual,
        )
        .expect("count Liter body");
        let bound = driver
            .prepared_final_wire_budget(&request)
            .expect("bound prepared Liter body");
        assert_eq!(body["model"], "uar-exact-cl100k-json-v1");
        assert_eq!(body["max_completion_tokens"], 128);
        assert_eq!(bound.input_tokens, expected.tokens);
        assert_eq!(bound.output_tokens, 128);

        request.messages = vec![serde_json::json!({
            "role":"user",
            "content":[{
                "type":"image_url",
                "image_url":{"url":"data:image/png;base64,iVBORw0KGgo="}
            }]
        })];
        let error = driver
            .prepared_final_wire_budget(&request)
            .expect_err("JSON counting must not certify finite image accounting");
        assert!(
            error
                .to_string()
                .contains("does not prove a finite image accounting bound")
        );
    }

    async fn identity_stream(
        deltas: Vec<serde_json::Value>,
    ) -> Vec<anyhow::Result<NormalizedEvent>> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = socket.read(&mut buffer).await.unwrap();
                assert!(read > 0);
                request.extend_from_slice(&buffer[..read]);
            }
            let header_end = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .unwrap()
                + 4;
            let content_length = std::str::from_utf8(&request[..header_end])
                .unwrap()
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .map(|(_, value)| value.trim().parse::<usize>().unwrap())
                .unwrap();
            while request.len() < header_end + content_length {
                let read = socket.read(&mut buffer).await.unwrap();
                assert!(read > 0);
                request.extend_from_slice(&buffer[..read]);
            }
            let mut body = String::new();
            for (delta, finish) in deltas
                .into_iter()
                .map(|delta| (delta, serde_json::Value::Null))
                .chain([(serde_json::json!({}), serde_json::json!("tool_calls"))])
            {
                body.push_str(&format!(
                    "data: {}\n\n",
                    serde_json::json!({
                        "id":"identity-fixture", "object":"chat.completion.chunk", "created":0,
                        "model":"test", "choices":[{"index":0,"delta":delta,"finish_reason":finish}]
                    })
                ));
            }
            body.push_str("data: [DONE]\n\n");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        let config = liter_llm::ClientConfigBuilder::new("identity-fixture-key")
            .base_url(format!("http://{address}/v1"))
            .max_retries(0)
            .build();
        let driver = LiterLlmDriver::new(config, "openai/test".into(), None).unwrap();
        let events = driver
            .stream(request(None))
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        server.await.unwrap();
        events
    }

    #[tokio::test]
    async fn typed_sse_preserves_interleaved_identity_and_arguments() {
        let events = identity_stream(vec![
            serde_json::json!({"tool_calls":[{"index":0,"id":" call-authentic ","type":"function","function":{"name":"first","arguments":"{\"a\":"}}]}),
            serde_json::json!({"tool_calls":[{"index":1,"id":"call-second","type":"function","function":{"name":"second","arguments":"{\"b\":"}}]}),
            serde_json::json!({"tool_calls":[{"index":0,"function":{"arguments":"1"}},{"index":1,"id":"","function":{"arguments":"2"}}]}),
            serde_json::json!({"tool_calls":[{"index":0,"id":"","function":{"arguments":"}"}},{"index":1,"id":"call-second","function":{"arguments":"}"}}]}),
            serde_json::json!({"tool_calls":[{"index":0,"id":" \t"},{"index":1}]}),
        ]).await;
        let events = events
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap();
        let calls = events
            .iter()
            .filter_map(|event| match event {
                NormalizedEvent::ToolCallComplete {
                    call_index,
                    id,
                    name,
                    arguments_json,
                } => Some((
                    *call_index,
                    id.as_str(),
                    name.as_str(),
                    arguments_json.as_str(),
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            calls,
            vec![
                (0, " call-authentic ", "first", "{\"a\":1}"),
                (1, "call-second", "second", "{\"b\":2}")
            ]
        );
        assert!(matches!(events.last(), Some(NormalizedEvent::Done)));
    }

    #[tokio::test]
    async fn typed_sse_conflict_is_a_fixed_typed_failure_without_completion() {
        let events = identity_stream(vec![
            serde_json::json!({"tool_calls":[{"index":0,"id":"private-first","type":"function","function":{"name":"first","arguments":"{}"}}]}),
            serde_json::json!({"tool_calls":[{"index":0,"id":"private-second"}]}),
        ]).await;
        let error = events.last().unwrap().as_ref().unwrap_err();
        let typed = error
            .downcast_ref::<ProviderError>()
            .expect("typed stream failure");
        assert_eq!(typed.kind, super::super::ProviderErrorKind::Stream);
        assert_eq!(
            typed.message,
            "Provider tool call identity changed within one stream index"
        );
        assert!(events.iter().all(|event| !matches!(
            event,
            Ok(NormalizedEvent::ToolCallComplete { .. } | NormalizedEvent::Done)
        )));
    }

    #[test]
    fn explicit_endpoint_bare_model_preserves_tool_controls() {
        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url("http://127.0.0.1:1/v1")
            .build();
        let driver = LiterLlmDriver::new(config, "gpt-4o-mini".into(), None)
            .expect("construct generic endpoint adapter");
        let mut req = request(None);
        req.parallel_tool_calls = Some(false);
        req.tool_choice = Some(crate::llm::ToolChoice::Named("lookup".into()));
        req.tools = vec![serde_json::json!({"type":"function","function":{
            "name":"lookup", "strict":true,
            "parameters":{"type":"object","properties":{},"additionalProperties":false}
        }})];
        let prepared = build_chat_request(
            &driver.model,
            None,
            driver.budget_endpoint_fingerprint.as_deref(),
            None,
            driver.generic_chat_adapter,
            &req,
        )
        .expect("preserve explicit controls");
        let body = serde_json::to_value(prepared.request).expect("serialize typed request");
        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["parallel_tool_calls"], false);
        assert_eq!(body["tool_choice"]["function"]["name"], "lookup");
        assert_eq!(body["tools"][0]["function"]["strict"], true);

        // Explicit endpoint native adapters must retain their refusal semantics.
        let config = liter_llm::ClientConfigBuilder::new("test-key")
            .base_url("http://127.0.0.1:1/v1")
            .build();
        assert!(!uses_generic_chat_adapter(&config, "anthropic/claude-test"));
        let failure = build_chat_request(
            "anthropic/claude-test",
            None,
            None,
            None,
            uses_generic_chat_adapter(&config, "anthropic/claude-test"),
            &req,
        )
        .err()
        .expect("native unsupported parallel control must fail");
        assert!(
            failure
                .to_string()
                .contains("explicit parallel_tool_calls is unsupported")
        );
        for model in [
            "tongyi-embedding-vision-plus",
            "dashscope/tongyi-embedding-vision-plus",
            "azure/deployment",
            "gemini/model",
            "google_ai/model",
            "vertex_ai/model",
            "bedrock/model",
        ] {
            assert!(
                !uses_generic_chat_adapter(&config, model),
                "native adapter: {model}"
            );
        }
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

        let first_event = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            assert_final_wire_budget_receipt(stream.next().await, "openai/test");
            stream.next().await
        })
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

        let error = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            assert_final_wire_budget_receipt(stream.next().await, &driver.model);
            stream.next().await
        })
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
