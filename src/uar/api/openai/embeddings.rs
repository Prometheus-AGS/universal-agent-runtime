//! OpenAI-compatible `POST /v1/embeddings`.
//!
//! The route embeds through the host's configured embedding backend — the same
//! [`SwappableEmbeddingBackend`](crate::uar::rag::embeddings::swappable::SwappableEmbeddingBackend)
//! that RAG ingest, memory and skill matching use — so every embedding a caller
//! receives comes from the provider UAR governs. There is no per-request
//! provider selection and no second provider call path: a request names the
//! served model, and a request for any other model is refused.
//!
//! Authentication, rate limiting and governance come from the router layers
//! shared by every `/v1/*` route.

use crate::AppState;
use crate::uar::rag::embeddings::{EmbeddingBackend, EmbeddingError};
use crate::uar::security::claims::UserContext;
use axum::{
    Json,
    extract::{Extension, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Upper bound on inputs per request, matching the `OpenAI` embeddings API.
/// Request bodies cross a trust boundary; this bounds the provider fan-out one
/// request can cause.
pub const MAX_EMBEDDING_INPUTS: usize = 2048;

/// `input` is one string or an array of strings. Token-id arrays are not
/// accepted: UAR backends embed text.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    One(String),
    Many(Vec<String>),
}

/// Request body for `POST /v1/embeddings`.
#[derive(Debug, Deserialize)]
pub struct EmbeddingRequest {
    /// Served embedding model: `<backend>/<model>` or the bare `<model>`.
    pub model: String,
    pub input: EmbeddingInput,
    /// Requested vector size. It must equal the served model's dimension; UAR
    /// does not truncate or project vectors.
    #[serde(default)]
    pub dimensions: Option<usize>,
    /// `float` (default) or `base64` (little-endian `f32`, as `OpenAI` encodes it).
    #[serde(default)]
    pub encoding_format: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum EmbeddingVector {
    Float(Vec<f32>),
    Base64(String),
}

#[derive(Debug, Serialize)]
struct EmbeddingRow {
    object: &'static str,
    index: usize,
    embedding: EmbeddingVector,
}

#[derive(Debug, Serialize)]
struct EmbeddingUsage {
    prompt_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Serialize)]
struct EmbeddingResponse {
    object: &'static str,
    data: Vec<EmbeddingRow>,
    model: String,
    /// UAR embedding backends do not report token counts; both fields are 0.
    usage: EmbeddingUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncodingFormat {
    Float,
    Base64,
}

fn error_response(status: StatusCode, message: &str, param: Option<&str>, code: &str) -> Response {
    let error_type = if status.is_server_error() {
        "server_error"
    } else {
        "invalid_request_error"
    };
    (
        status,
        Json(serde_json::json!({
            "error": {
                "message": message,
                "type": error_type,
                "param": param,
                "code": code,
            }
        })),
    )
        .into_response()
}

/// Canonical UAR address of the served model, `<backend>/<model>`.
fn served_model(backend: &dyn EmbeddingBackend) -> String {
    format!("{}/{}", backend.backend_name(), backend.model_id())
}

/// Whether `requested` names the served model, either as `<backend>/<model>`
/// or as the bare model id.
fn names_served_model(requested: &str, backend: &dyn EmbeddingBackend) -> bool {
    let requested = requested.trim();
    let model = backend.model_id();
    requested == model
        || requested
            .strip_prefix(backend.backend_name())
            .and_then(|rest| rest.strip_prefix('/'))
            == Some(model)
}

fn encode_base64(vector: &[f32]) -> String {
    let bytes: Vec<u8> = vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    BASE64.encode(bytes)
}

/// `POST /v1/embeddings`.
pub async fn create_embeddings(
    State(state): State<AppState>,
    // Present only after the auth middleware admitted the caller; a mount
    // without that layer fails closed on the missing extension.
    Extension(_user): Extension<UserContext>,
    body: Result<Json<EmbeddingRequest>, JsonRejection>,
) -> Response {
    let request = match body {
        Ok(Json(request)) => request,
        Err(rejection) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!(
                    "Request must be JSON with `model` and `input` (a string or an array of strings): {}",
                    rejection.body_text()
                ),
                None,
                "invalid_request",
            );
        }
    };
    // Pin one backend snapshot for the whole request so a concurrent settings
    // swap cannot mix two embedding spaces in one answer.
    embed_request(state.embedding_backend.pin(), request).await
}

/// Validate `request` against the pinned `backend`, embed, and render the
/// OpenAI-compatible answer.
async fn embed_request(backend: Arc<dyn EmbeddingBackend>, request: EmbeddingRequest) -> Response {
    let encoding = match request.encoding_format.as_deref() {
        None | Some("float") => EncodingFormat::Float,
        Some("base64") => EncodingFormat::Base64,
        Some(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "`encoding_format` must be `float` or `base64`",
                Some("encoding_format"),
                "invalid_encoding_format",
            );
        }
    };
    let inputs: Vec<String> = match request.input {
        EmbeddingInput::One(text) => vec![text],
        EmbeddingInput::Many(texts) => texts,
    };
    if inputs.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "`input` must contain at least one string",
            Some("input"),
            "invalid_input",
        );
    }
    if inputs.len() > MAX_EMBEDDING_INPUTS {
        return error_response(
            StatusCode::BAD_REQUEST,
            &format!("`input` may contain at most {MAX_EMBEDDING_INPUTS} strings"),
            Some("input"),
            "too_many_inputs",
        );
    }
    if inputs.iter().any(String::is_empty) {
        return error_response(
            StatusCode::BAD_REQUEST,
            "`input` strings must not be empty",
            Some("input"),
            "invalid_input",
        );
    }

    if backend.backend_name() == "unavailable" {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "No embedding backend is available on this runtime",
            None,
            "embedding_backend_unavailable",
        );
    }
    let served = served_model(backend.as_ref());
    if !names_served_model(&request.model, backend.as_ref()) {
        return error_response(
            StatusCode::NOT_FOUND,
            &format!(
                "Embedding model `{}` is not served by this runtime; it embeds with `{served}`",
                request.model.trim()
            ),
            Some("model"),
            "model_not_found",
        );
    }
    let dimension = backend.vector_dimension();
    match request.dimensions {
        Some(0) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "`dimensions` must be positive",
                Some("dimensions"),
                "invalid_dimensions",
            );
        }
        Some(requested) if requested != dimension => {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!(
                    "`{served}` produces {dimension}-dimensional vectors; `dimensions` {requested} is not supported"
                ),
                Some("dimensions"),
                "dimensions_mismatch",
            );
        }
        _ => {}
    }

    let texts: Vec<&str> = inputs.iter().map(String::as_str).collect();
    let vectors = match backend.embed(&texts).await {
        Ok(vectors) => vectors,
        Err(error) => {
            tracing::warn!(
                backend = %backend.backend_name(),
                model = %backend.model_id(),
                inputs = texts.len(),
                error = %error,
                "Embedding request failed"
            );
            return match error {
                EmbeddingError::BackendUnavailable(_) | EmbeddingError::MissingApiKey { .. } => {
                    error_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "The embedding backend is not available",
                        None,
                        "embedding_backend_unavailable",
                    )
                }
                EmbeddingError::InvalidInput(message) => error_response(
                    StatusCode::BAD_REQUEST,
                    &message,
                    Some("input"),
                    "invalid_input",
                ),
                EmbeddingError::RequestFailed(_)
                | EmbeddingError::EmptyResponse
                | EmbeddingError::DimensionMismatch { .. } => error_response(
                    StatusCode::BAD_GATEWAY,
                    "The embedding provider failed to embed the input",
                    None,
                    "embedding_provider_error",
                ),
            };
        }
    };
    // The provider response is external data: refuse an answer that does not
    // give exactly one vector of the served size per input.
    if vectors.len() != texts.len() || vectors.iter().any(|vector| vector.len() != dimension) {
        tracing::warn!(
            backend = %backend.backend_name(),
            model = %backend.model_id(),
            inputs = texts.len(),
            vectors = vectors.len(),
            "Embedding backend returned a malformed batch"
        );
        return error_response(
            StatusCode::BAD_GATEWAY,
            "The embedding provider returned a malformed answer",
            None,
            "embedding_provider_error",
        );
    }

    let data = vectors
        .into_iter()
        .enumerate()
        .map(|(index, vector)| EmbeddingRow {
            object: "embedding",
            index,
            embedding: match encoding {
                EncodingFormat::Float => EmbeddingVector::Float(vector),
                EncodingFormat::Base64 => EmbeddingVector::Base64(encode_base64(&vector)),
            },
        })
        .collect();
    Json(EmbeddingResponse {
        object: "list",
        data,
        model: served,
        usage: EmbeddingUsage {
            prompt_tokens: 0,
            total_tokens: 0,
        },
    })
    .into_response()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::uar::rag::embeddings::UnavailableEmbeddingBackend;
    use async_trait::async_trait;
    use serde_json::Value;

    /// Deterministic backend: vector `i` of a batch is `[len(text_i), i, 0, …]`.
    #[derive(Debug)]
    struct StubBackend {
        dimension: usize,
        fail: Option<fn() -> EmbeddingError>,
        drop_last: bool,
    }

    impl StubBackend {
        fn ok(dimension: usize) -> Arc<dyn EmbeddingBackend> {
            Arc::new(Self {
                dimension,
                fail: None,
                drop_last: false,
            })
        }
    }

    #[async_trait]
    impl EmbeddingBackend for StubBackend {
        fn backend_name(&self) -> &str {
            "openai"
        }

        fn vector_dimension(&self) -> usize {
            self.dimension
        }

        fn model_id(&self) -> &str {
            "text-embedding-3-small"
        }

        async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            if let Some(fail) = self.fail {
                return Err(fail());
            }
            let mut vectors: Vec<Vec<f32>> = texts
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    let mut vector = vec![0.0; self.dimension];
                    vector[0] = text.len() as f32;
                    vector[1] = index as f32;
                    vector
                })
                .collect();
            if self.drop_last {
                vectors.pop();
            }
            Ok(vectors)
        }
    }

    fn request(body: Value) -> EmbeddingRequest {
        serde_json::from_value(body).expect("request deserializes")
    }

    async fn call(backend: Arc<dyn EmbeddingBackend>, body: Value) -> (StatusCode, Value) {
        let response = embed_request(backend, request(body)).await;
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body reads");
        (
            status,
            serde_json::from_slice(&bytes).expect("body is JSON"),
        )
    }

    #[tokio::test]
    async fn embeds_an_array_in_input_order_with_the_canonical_model() {
        let (status, body) = call(
            StubBackend::ok(4),
            serde_json::json!({"model": "openai/text-embedding-3-small", "input": ["a", "bbb"]}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["object"], "list");
        assert_eq!(body["model"], "openai/text-embedding-3-small");
        assert_eq!(body["usage"]["prompt_tokens"], 0);
        let data = body["data"].as_array().unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0]["object"], "embedding");
        assert_eq!(data[0]["index"], 0);
        assert_eq!(
            data[0]["embedding"],
            serde_json::json!([1.0, 0.0, 0.0, 0.0])
        );
        assert_eq!(data[1]["index"], 1);
        assert_eq!(
            data[1]["embedding"],
            serde_json::json!([3.0, 1.0, 0.0, 0.0])
        );
    }

    #[tokio::test]
    async fn accepts_a_single_string_the_bare_model_id_and_the_served_dimension() {
        let (status, body) = call(
            StubBackend::ok(4),
            serde_json::json!({"model": "text-embedding-3-small", "input": "hello", "dimensions": 4}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn base64_encoding_is_little_endian_f32() {
        let (status, body) = call(
            StubBackend::ok(2),
            serde_json::json!({"model": "text-embedding-3-small", "input": "ab", "encoding_format": "base64"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let encoded = body["data"][0]["embedding"].as_str().unwrap();
        let bytes = BASE64.decode(encoded).unwrap();
        let decoded: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        assert_eq!(decoded, vec![2.0, 0.0]);
    }

    #[tokio::test]
    async fn refuses_a_model_the_runtime_does_not_serve() {
        let (status, body) = call(
            StubBackend::ok(4),
            serde_json::json!({"model": "voyage/text-embedding-3-small", "input": "x"}),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "model_not_found");
        assert_eq!(body["error"]["param"], "model");
    }

    #[tokio::test]
    async fn refuses_a_dimension_other_than_the_served_one() {
        let (status, body) = call(
            StubBackend::ok(4),
            serde_json::json!({"model": "text-embedding-3-small", "input": "x", "dimensions": 8}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "dimensions_mismatch");
        let (status, body) = call(
            StubBackend::ok(4),
            serde_json::json!({"model": "text-embedding-3-small", "input": "x", "dimensions": 0}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "invalid_dimensions");
    }

    #[tokio::test]
    async fn refuses_empty_oversized_and_blank_input() {
        for (input, code) in [
            (serde_json::json!([]), "invalid_input"),
            (serde_json::json!(["ok", ""]), "invalid_input"),
            (
                serde_json::json!(vec!["x"; MAX_EMBEDDING_INPUTS + 1]),
                "too_many_inputs",
            ),
        ] {
            let (status, body) = call(
                StubBackend::ok(4),
                serde_json::json!({"model": "text-embedding-3-small", "input": input}),
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(body["error"]["code"], code);
        }
    }

    #[tokio::test]
    async fn refuses_an_unknown_encoding_format() {
        let (status, body) = call(
            StubBackend::ok(4),
            serde_json::json!({"model": "text-embedding-3-small", "input": "x", "encoding_format": "int8"}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "invalid_encoding_format");
    }

    #[tokio::test]
    async fn unavailable_backend_answers_503() {
        let backend: Arc<dyn EmbeddingBackend> =
            Arc::new(UnavailableEmbeddingBackend::new(4, "not compiled"));
        let (status, body) = call(
            backend,
            serde_json::json!({"model": "text-embedding-3-small", "input": "x"}),
        )
        .await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], "embedding_backend_unavailable");
        assert_eq!(body["error"]["type"], "server_error");
    }

    #[tokio::test]
    async fn provider_failures_map_to_502_or_503_without_provider_detail() {
        let failing = |fail: fn() -> EmbeddingError| -> Arc<dyn EmbeddingBackend> {
            Arc::new(StubBackend {
                dimension: 4,
                fail: Some(fail),
                drop_last: false,
            })
        };
        let (status, body) = call(
            failing(|| EmbeddingError::RequestFailed("upstream secret detail".into())),
            serde_json::json!({"model": "text-embedding-3-small", "input": "x"}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["error"]["code"], "embedding_provider_error");
        assert!(!body.to_string().contains("upstream secret detail"));
        let (status, body) = call(
            failing(|| EmbeddingError::MissingApiKey {
                backend: "openai".into(),
            }),
            serde_json::json!({"model": "text-embedding-3-small", "input": "x"}),
        )
        .await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], "embedding_backend_unavailable");
    }

    #[tokio::test]
    async fn malformed_backend_batch_answers_502() {
        let backend: Arc<dyn EmbeddingBackend> = Arc::new(StubBackend {
            dimension: 4,
            fail: None,
            drop_last: true,
        });
        let (status, body) = call(
            backend,
            serde_json::json!({"model": "text-embedding-3-small", "input": ["a", "b"]}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["error"]["code"], "embedding_provider_error");
    }

    #[test]
    fn token_id_input_does_not_deserialize() {
        let parsed = serde_json::from_value::<EmbeddingRequest>(
            serde_json::json!({"model": "m", "input": [1, 2, 3]}),
        );
        assert!(parsed.is_err());
    }
}
