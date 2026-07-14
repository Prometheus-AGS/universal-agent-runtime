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
    /// Run API.
    #[must_use]
    pub fn runs(&self) -> RunsApi<'_> {
        RunsApi(self)
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
