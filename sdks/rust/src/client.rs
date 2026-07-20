//! Async HTTP client for the UAR and its OpenAI-compatible surface.

use crate::{
    error::{Error, Result, UarError},
    types::*,
};
use futures::{Stream, StreamExt};
use reqwest::{RequestBuilder, Response, multipart};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::VecDeque, pin::Pin};
use url::Url;

/// A fallible stream of decoded UAR events.
pub type EventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

/// Cloneable asynchronous UAR client.
#[derive(Debug, Clone)]
pub struct Client {
    base_url: Url,
    http: reqwest::Client,
    api_key: Option<String>,
}

impl Client {
    /// Create a client for a runtime URL.
    ///
    /// # Errors
    /// Returns [`Error::InvalidUrl`] when `base_url` is invalid.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            base_url: Url::parse(base_url.as_ref())?,
            http: reqwest::Client::new(),
            api_key: None,
        })
    }

    /// Use a custom HTTP client.
    pub fn with_client(base_url: impl AsRef<str>, http: reqwest::Client) -> Result<Self> {
        Ok(Self {
            base_url: Url::parse(base_url.as_ref())?,
            http,
            api_key: None,
        })
    }

    /// Attach a bearer API key to subsequent requests.
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Runtime base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
    /// Chat API.
    #[must_use]
    pub fn chat(&self) -> ChatApi<'_> {
        ChatApi(self)
    }
    /// Provider catalog and persisted provider administration API.
    #[must_use]
    pub fn providers(&self) -> ProvidersApi<'_> {
        ProvidersApi(self)
    }
    /// Scoped chat policy and discovery control plane.
    #[must_use]
    pub fn control_plane(&self) -> ControlPlaneApi<'_> {
        ControlPlaneApi(self)
    }
    /// UAR-owned administration API for settings and runtime resources.
    #[must_use]
    pub fn admin(&self) -> AdminApi<'_> {
        AdminApi(self)
    }
    /// Run API.
    #[must_use]
    pub fn runs(&self) -> RunsApi<'_> {
        RunsApi(self)
    }
    /// A2UI artifact/component library API.
    #[must_use]
    pub fn a2ui(&self) -> A2uiApi<'_> {
        A2uiApi(self)
    }
    /// Knowledge API.
    #[must_use]
    pub fn knowledge(&self) -> KnowledgeApi<'_> {
        KnowledgeApi(self)
    }
    /// Ingestion API.
    #[must_use]
    pub fn ingest(&self) -> IngestApi<'_> {
        IngestApi(self)
    }
    /// Tool execution API.
    #[must_use]
    pub fn tools(&self) -> ToolsApi<'_> {
        ToolsApi(self)
    }
    /// Embeddings API.
    #[must_use]
    pub fn embeddings(&self) -> EmbeddingsApi<'_> {
        EmbeddingsApi(self)
    }

    fn url(&self, path: &str) -> Result<Url> {
        Ok(self.base_url.join(path.trim_start_matches('/'))?)
    }
    fn request(&self, method: reqwest::Method, path: &str) -> Result<RequestBuilder> {
        let request = self.http.request(method, self.url(path)?);
        Ok(match &self.api_key {
            Some(key) => request.bearer_auth(key),
            None => request,
        })
    }
    async fn json<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        decode(request.send().await?).await
    }
    async fn no_content(&self, request: RequestBuilder) -> Result<()> {
        let response = request.send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(api_error(response).await)
        }
    }
    async fn stream(&self, request: RequestBuilder) -> Result<EventStream> {
        let response = request
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(api_error(response).await);
        }
        let state = (
            response.bytes_stream(),
            String::new(),
            VecDeque::<StreamEvent>::new(),
            false,
        );
        Ok(Box::pin(futures::stream::unfold(
            state,
            |(mut source, mut buffer, mut queued, done)| async move {
                loop {
                    if let Some(event) = queued.pop_front() {
                        return Some((Ok(event), (source, buffer, queued, done)));
                    }
                    if done {
                        return None;
                    }
                    match source.next().await {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));
                            let normalized = buffer.replace("\r\n", "\n");
                            let mut pieces: Vec<&str> = normalized.split("\n\n").collect();
                            let tail = pieces.pop().unwrap_or_default().to_owned();
                            queued.extend(pieces.into_iter().filter_map(parse_event));
                            buffer = tail;
                        }
                        Some(Err(error)) => {
                            return Some((Err(Error::Http(error)), (source, buffer, queued, true)));
                        }
                        None => {
                            if let Some(event) = parse_event(&buffer) {
                                queued.push_back(event);
                            }
                            if let Some(event) = queued.pop_front() {
                                return Some((Ok(event), (source, String::new(), queued, true)));
                            }
                            return None;
                        }
                    }
                }
            },
        )))
    }
}

/// A2UI component library operations.
pub struct A2uiApi<'a>(&'a Client);
impl A2uiApi<'_> {
    pub async fn list_components(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/a2ui/components")?,
            )
            .await
    }

    pub async fn promote_component(
        &self,
        title: &str,
        source: &str,
        description: Option<&str>,
    ) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/uar/a2ui/components")?
                    .json(&serde_json::json!({
                        "title": title,
                        "source": source,
                        "description": description,
                    })),
            )
            .await
    }
}

/// Runtime administration operations used by first-party KnowMe settings UIs.
pub struct AdminApi<'a>(&'a Client);
impl AdminApi<'_> {
    /// Return all persisted UAR settings with schema/source metadata.
    pub async fn settings(&self) -> Result<Value> {
        self.0
            .json(self.0.request(reqwest::Method::GET, "/api/uar/settings")?)
            .await
    }

    /// Return registered settings namespaces and their JSON schemas.
    pub async fn setting_types(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/settings/types")?,
            )
            .await
    }

    /// Update one dotted UAR setting key.
    pub async fn update_setting(&self, key: &str, value: &Value) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::PUT, &format!("/api/uar/settings/{key}"))?
                    .json(&serde_json::json!({ "value": value })),
            )
            .await
    }

    /// Create a UAR-owned agent definition.
    pub async fn create_agent(&self, agent: &Value) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/agents")?
                    .json(agent),
            )
            .await
    }

    /// Replace a UAR-owned agent definition.
    pub async fn update_agent(&self, id: &str, agent: &Value) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::PUT, &format!("/api/agents/{id}"))?
                    .json(agent),
            )
            .await
    }

    /// Delete a non-protected agent.
    pub async fn delete_agent(&self, id: &str) -> Result<()> {
        self.0
            .no_content(
                self.0
                    .request(reqwest::Method::DELETE, &format!("/api/agents/{id}"))?,
            )
            .await
    }

    /// Return configured MCP servers and live connection/tool status.
    pub async fn mcp_servers(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/mcp/servers")?,
            )
            .await
    }

    /// Add or replace an MCP server and connect it immediately when enabled.
    pub async fn save_mcp_server(&self, name: &str, server: &Value) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::PUT,
                        &format!("/api/uar/mcp/servers/{name}"),
                    )?
                    .json(server),
            )
            .await
    }

    /// Delete an MCP server from UAR storage and the live registry.
    pub async fn delete_mcp_server(&self, name: &str) -> Result<Value> {
        self.0
            .json(self.0.request(
                reqwest::Method::DELETE,
                &format!("/api/uar/mcp/servers/{name}"),
            )?)
            .await
    }

    /// List user-visible memories owned by UAR.
    pub async fn memories(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/admin/memories")?,
            )
            .await
    }

    /// Delete one governed memory record.
    pub async fn delete_memory(&self, id: &str) -> Result<Value> {
        self.0
            .json(self.0.request(
                reqwest::Method::DELETE,
                &format!("/api/admin/memories/{id}"),
            )?)
            .await
    }

    /// Create a UAR-owned knowledge base.
    pub async fn create_knowledge_base(&self, request: &Value) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/uar/knowledge-bases/")?
                    .json(request),
            )
            .await
    }

    /// Delete a UAR-owned knowledge base and its governed documents.
    pub async fn delete_knowledge_base(&self, id: &str) -> Result<()> {
        self.0
            .no_content(self.0.request(
                reqwest::Method::DELETE,
                &format!("/api/uar/knowledge-bases/{id}"),
            )?)
            .await
    }
}

/// Agent/resource discovery and scoped conversation-policy operations.
pub struct ControlPlaneApi<'a>(&'a Client);
impl ControlPlaneApi<'_> {
    /// Runtime agents, including the protected orchestrator/default agents.
    pub async fn agents(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/discovery/agents")?,
            )
            .await
    }

    /// Enabled skill definitions available to policy resolution.
    pub async fn skills(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/discovery/skills")?,
            )
            .await
    }

    /// MCP servers, MCP tools, and built-in tools.
    pub async fn tools(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/discovery/tools")?,
            )
            .await
    }

    /// Knowledge bases owned by UAR.
    pub async fn knowledge_bases(&self) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/knowledge-bases")?,
            )
            .await
    }

    /// Load a conversation policy. Missing policy resolves to JSON null.
    pub async fn conversation_policy(&self, conversation_id: &str) -> Result<Value> {
        let response = self
            .0
            .request(
                reqwest::Method::GET,
                &format!("/api/uar/conversations/{conversation_id}/policy"),
            )?
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Value::Null);
        }
        decode(response).await
    }

    /// Persist conversation-scoped intent in UAR storage.
    pub async fn save_conversation_policy(
        &self,
        conversation_id: &str,
        policy: &Value,
    ) -> Result<Value> {
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::PUT,
                        &format!("/api/uar/conversations/{conversation_id}/policy"),
                    )?
                    .json(policy),
            )
            .await
    }

    /// Delete conversation policy and return to inherited policy.
    pub async fn delete_conversation_policy(&self, conversation_id: &str) -> Result<()> {
        self.0
            .no_content(self.0.request(
                reqwest::Method::DELETE,
                &format!("/api/uar/conversations/{conversation_id}/policy"),
            )?)
            .await
    }

    /// Resolve the immutable effective policy used for the next turn.
    pub async fn effective_config(&self, conversation_id: &str) -> Result<Value> {
        self.0
            .json(self.0.request(
                reqwest::Method::GET,
                &format!("/api/uar/sessions/{conversation_id}/effective-config"),
            )?)
            .await
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T> {
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(api_error(response).await)
    }
}

async fn api_error(response: Response) -> Error {
    let status = response.status().as_u16();
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| "runtime returned an unreadable error".into());
    let envelope = serde_json::from_str::<UarError>(&text)
        .or_else(|_| {
            serde_json::from_str::<Value>(&text).map(|v| {
                let inner = v.get("error").unwrap_or(&v);
                UarError {
                    code: inner
                        .get("code")
                        .and_then(Value::as_str)
                        .unwrap_or("uar::http_error")
                        .into(),
                    message: inner
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or(&text)
                        .into(),
                    request_id: inner
                        .get("request_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                }
            })
        })
        .unwrap_or(UarError {
            code: "uar::http_error".into(),
            message: text,
            request_id: None,
        });
    Error::Api {
        status,
        error_code: envelope.code,
        message: envelope.message,
        request_id: envelope.request_id.or(request_id),
    }
}

fn parse_event(block: &str) -> Option<StreamEvent> {
    let mut id = None;
    let mut event = None;
    let mut data = Vec::new();
    for line in block.lines() {
        if line.starts_with(':') {
            continue;
        }
        let (field, value) = line
            .split_once(':')
            .map_or((line, ""), |(a, b)| (a, b.strip_prefix(' ').unwrap_or(b)));
        match field {
            "id" => id = Some(value.into()),
            "event" => event = Some(value.into()),
            "data" => data.push(value),
            _ => {}
        }
    }
    if id.is_none() && event.is_none() && data.is_empty() {
        return None;
    }
    let raw = data.join("\n");
    Some(StreamEvent {
        id,
        event,
        data: serde_json::from_str(&raw).unwrap_or(Value::String(raw)),
    })
}

/// Chat completion operations.
pub struct ChatApi<'a>(&'a Client);
impl ChatApi<'_> {
    /// Create a non-streaming completion.
    pub async fn complete(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        request.stream = false;
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/chat/completion")?
                    .json(&request),
            )
            .await
    }
    /// Stream completion chunks as typed SSE events.
    pub async fn stream(&self, mut request: ChatCompletionRequest) -> Result<EventStream> {
        request.stream = true;
        self.0
            .stream(
                self.0
                    .request(reqwest::Method::POST, "/api/chat/completion")?
                    .json(&request),
            )
            .await
    }
}

/// Provider catalog and configuration operations.
pub struct ProvidersApi<'a>(&'a Client);
impl ProvidersApi<'_> {
    /// Return the complete compile-time provider/model catalog with metadata.
    pub async fn catalog(&self) -> Result<Value> {
        self.0
            .json(self.0.request(reqwest::Method::GET, "/api/models")?)
            .await
    }

    /// Return all configured providers (credentials are never returned).
    pub async fn list(&self) -> Result<ProvidersResponse> {
        self.0
            .json(self.0.request(reqwest::Method::GET, "/api/uar/providers")?)
            .await
    }

    /// Return only enabled provider/model routes.
    pub async fn list_enabled(&self) -> Result<ProvidersResponse> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, "/api/uar/providers/enabled")?,
            )
            .await
    }

    /// Create or update a provider configuration in UAR-owned storage.
    pub async fn save(&self, config: SaveProviderConfig) -> Result<ProviderView> {
        let exists = self
            .list()
            .await?
            .providers
            .iter()
            .any(|item| item.id == config.id);
        let path = if exists {
            format!("/api/uar/providers/{}", config.id)
        } else {
            "/api/uar/providers".to_string()
        };
        let method = if exists {
            reqwest::Method::PUT
        } else {
            reqwest::Method::POST
        };
        self.0
            .json(self.0.request(method, &path)?.json(&config))
            .await
    }

    /// Delete a configured provider.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.0
            .no_content(
                self.0
                    .request(reqwest::Method::DELETE, &format!("/api/uar/providers/{id}"))?,
            )
            .await
    }

    /// Perform a minimal live request using the configured provider.
    pub async fn test(&self, id: &str, model: Option<&str>) -> Result<ProviderTestResponse> {
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::POST,
                        &format!("/api/uar/providers/{id}/test"),
                    )?
                    .json(&serde_json::json!({ "model": model })),
            )
            .await
    }

    /// Select the provider as UAR's default route.
    pub async fn set_default(&self, id: &str) -> Result<()> {
        self.0
            .no_content(self.0.request(
                reqwest::Method::POST,
                &format!("/api/uar/providers/{id}/default"),
            )?)
            .await
    }
}

/// Agent-run lifecycle operations.
pub struct RunsApi<'a>(&'a Client);
impl RunsApi<'_> {
    /// Create a run.
    pub async fn create(&self, request: CreateRunRequest) -> Result<RunResponse> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/uar/runs")?
                    .json(&request),
            )
            .await
    }
    /// Stream a run, optionally replaying after `last_event_id`.
    pub async fn stream(&self, run_id: &str, last_event_id: Option<u64>) -> Result<EventStream> {
        let mut request = self.0.request(
            reqwest::Method::GET,
            &format!("/api/uar/runs/{run_id}/stream"),
        )?;
        if let Some(id) = last_event_id {
            request = request
                .query(&[("last_event_id", id)])
                .header("last-event-id", id);
        }
        self.0.stream(request).await
    }
    /// Cancel an active run.
    pub async fn cancel(&self, run_id: &str) -> Result<CancelRunResponse> {
        self.0
            .json(self.0.request(
                reqwest::Method::POST,
                &format!("/api/uar/runs/{run_id}/cancel"),
            )?)
            .await
    }
    /// Cancel the current run associated with a stable conversation session.
    pub async fn cancel_session(&self, session_id: &str) -> Result<CancelRunResponse> {
        self.0
            .json(self.0.request(
                reqwest::Method::POST,
                &format!("/api/uar/sessions/{session_id}/cancel"),
            )?)
            .await
    }
    /// Submit a response to an A2UI input artifact and let the paused run continue.
    pub async fn submit_artifact_response(
        &self,
        run_id: &str,
        request: ArtifactResponseRequest,
    ) -> Result<ArtifactResponseAck> {
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::POST,
                        &format!("/api/uar/runs/{run_id}/artifact-response"),
                    )?
                    .json(&request),
            )
            .await
    }
    /// Submit an action from a rendered A2UI surface and continue the agent.
    pub async fn submit_a2ui_action(
        &self,
        run_id: &str,
        request: A2uiActionRequest,
    ) -> Result<A2uiActionAck> {
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::POST,
                        &format!("/api/uar/runs/{run_id}/a2ui/actions"),
                    )?
                    .json(&request),
            )
            .await
    }
    /// List persisted checkpoints.
    pub async fn checkpoints(&self, run_id: &str) -> Result<CheckpointListResponse> {
        self.0
            .json(self.0.request(
                reqwest::Method::GET,
                &format!("/api/uar/runs/{run_id}/checkpoints"),
            )?)
            .await
    }
    /// Resume from the latest checkpoint.
    pub async fn resume(&self, run_id: &str, request: ResumeRunRequest) -> Result<RunResponse> {
        self.resume_at(run_id, None, request).await
    }
    /// Resume from a selected checkpoint.
    pub async fn resume_at(
        &self,
        run_id: &str,
        checkpoint_id: Option<&str>,
        request: ResumeRunRequest,
    ) -> Result<RunResponse> {
        let path = checkpoint_id.map_or_else(
            || format!("/api/uar/runs/{run_id}/resume"),
            |id| format!("/api/uar/runs/{run_id}/resume/{id}"),
        );
        self.0
            .json(self.0.request(reqwest::Method::POST, &path)?.json(&request))
            .await
    }
}

/// Tool execution operations.
pub struct ToolsApi<'a>(&'a Client);
impl ToolsApi<'_> {
    /// Execute a named registered tool.
    pub async fn execute(&self, name: &str, arguments: Value) -> Result<ToolCallResponse> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, &format!("/api/tools/{name}/execute"))?
                    .json(&ToolCallRequest { arguments }),
            )
            .await
    }
}

/// Embedding operations.
pub struct EmbeddingsApi<'a>(&'a Client);
impl EmbeddingsApi<'_> {
    /// Generate embedding vectors through the OpenAI-compatible route.
    pub async fn create(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/v1/embeddings")?
                    .json(&request),
            )
            .await
    }
}

/// Knowledge-base and document operations.
pub struct KnowledgeApi<'a>(&'a Client);
impl KnowledgeApi<'_> {
    /// List knowledge bases.
    pub async fn list(&self) -> Result<Vec<KnowledgeBase>> {
        self.0
            .json(self.0.request(reqwest::Method::GET, "/api/knowledge/")?)
            .await
    }
    /// Create a knowledge base.
    pub async fn create(&self, request: CreateKnowledgeBaseRequest) -> Result<KnowledgeBase> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/knowledge/")?
                    .json(&request),
            )
            .await
    }
    /// Get a knowledge base.
    pub async fn get(&self, id: &str) -> Result<KnowledgeBase> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::GET, &format!("/api/knowledge/{id}"))?,
            )
            .await
    }
    /// Update a knowledge base.
    pub async fn update(
        &self,
        id: &str,
        request: UpdateKnowledgeBaseRequest,
    ) -> Result<KnowledgeBase> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::PUT, &format!("/api/knowledge/{id}"))?
                    .json(&request),
            )
            .await
    }
    /// Delete a knowledge base.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.0
            .no_content(
                self.0
                    .request(reqwest::Method::DELETE, &format!("/api/knowledge/{id}"))?,
            )
            .await
    }
    /// List documents.
    pub async fn list_documents(&self, id: &str) -> Result<Vec<Document>> {
        self.0
            .json(self.0.request(
                reqwest::Method::GET,
                &format!("/api/knowledge/{id}/documents"),
            )?)
            .await
    }
    /// Get a document.
    pub async fn get_document(&self, id: &str, document_id: &str) -> Result<Document> {
        self.0
            .json(self.0.request(
                reqwest::Method::GET,
                &format!("/api/knowledge/{id}/documents/{document_id}"),
            )?)
            .await
    }
    /// Upload document bytes.
    pub async fn upload_document(
        &self,
        id: &str,
        filename: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<Document> {
        let part = multipart::Part::bytes(bytes.into()).file_name(filename.into());
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::POST,
                        &format!("/api/knowledge/{id}/documents"),
                    )?
                    .multipart(multipart::Form::new().part("file", part)),
            )
            .await
    }
    /// Delete a document.
    pub async fn delete_document(&self, id: &str, document_id: &str) -> Result<()> {
        self.0
            .no_content(self.0.request(
                reqwest::Method::DELETE,
                &format!("/api/knowledge/{id}/documents/{document_id}"),
            )?)
            .await
    }
    /// Search a knowledge base.
    pub async fn search(&self, id: &str, query: impl Into<String>) -> Result<SearchResponse> {
        self.search_with_options(id, query, 5, 0.7).await
    }
    /// Search with explicit result controls.
    pub async fn search_with_options(
        &self,
        id: &str,
        query: impl Into<String>,
        limit: usize,
        min_score: f32,
    ) -> Result<SearchResponse> {
        self.0
            .json(
                self.0
                    .request(
                        reqwest::Method::POST,
                        &format!("/api/knowledge/{id}/search"),
                    )?
                    .json(&SearchRequest {
                        query: query.into(),
                        limit,
                        min_score,
                    }),
            )
            .await
    }
}

/// Generic ingestion operations.
pub struct IngestApi<'a>(&'a Client);
impl IngestApi<'_> {
    /// Ingest text content and optional metadata.
    pub async fn ingest(
        &self,
        content: impl Into<String>,
        metadata: Option<Value>,
    ) -> Result<IngestResponse> {
        self.0
            .json(
                self.0
                    .request(reqwest::Method::POST, "/api/ingest")?
                    .json(&IngestRequest {
                        content: content.into(),
                        metadata,
                    }),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_multiline_sse() {
        let event = parse_event("id: 42\nevent: delta\ndata: {\"x\":\ndata: 1}").unwrap();
        assert_eq!(event.id.as_deref(), Some("42"));
        assert_eq!(event.event.as_deref(), Some("delta"));
        assert_eq!(event.data["x"], 1);
    }
    #[test]
    fn chat_request_carries_tools_and_schema() {
        let request = ChatCompletionRequest {
            messages: vec![ChatMessage::text("user", "hi")],
            tools: vec![serde_json::json!({"type":"function"})],
            response_format: Some(serde_json::json!({"type":"json_object"})),
            ..Default::default()
        };
        let json = serde_json::to_value(request).unwrap();
        assert!(json["tools"].is_array());
        assert_eq!(json["response_format"]["type"], "json_object");
    }
}
